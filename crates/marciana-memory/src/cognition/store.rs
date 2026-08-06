//! Durable leased scheduler and proposal persistence over guarded graph CAS.
//!
//! These are storage-level primitives for an authenticated scheduler and
//! trusted worker pool; canonical owner strings and scoped job keys are stable
//! identities, not credentials. A worker may intentionally differ from the
//! submitter so expired work can be transferred. After acquisition, the
//! unpersisted lease token is the sole bearer credential for worker state
//! changes. Marciana's composition boundary must authenticate access to these
//! methods and must never treat knowledge of a job key as authorization.

use chrono::{DateTime, Duration, Utc};
use grust_core::prelude::GraphCommitStore;
use typesec_memory::{CognitionIdempotencyKey, CognitionProposal};

use super::bounds::{is_bounded_failure, is_canonical_text};
use super::graph::{
    authority_scope_matches, error_digest, is_sha256, job_digest, owner_digest, token_digest,
    validate_idempotency_key,
};
use super::invariants::{advance_attempt, advance_job, validate_transition_time};
use super::lease::{
    cancel_exhausted, lease_expiry, new_lease_token, validate_bearer_token, validate_lease,
};
use super::{
    CognitionJob, CognitionJobClaim, CognitionJobClaimRequest, CognitionJobStatus, CognitionLease,
    CognitionLeaseState, CognitionStateError,
};
use crate::GraphStoreMemoryStore;

impl<G: GraphCommitStore> GraphStoreMemoryStore<G> {
    /// Submit a durable job for a verified `TypeDID` request envelope, or recover
    /// the identical existing submission.
    ///
    /// `owner` identifies the authenticated submitting scheduler. This storage
    /// method records its digest but does not authenticate the caller itself.
    ///
    /// # Errors
    ///
    /// Returns [`CognitionStateError`] when the job key or digest is invalid
    /// or the persisted job state fails validation.
    pub fn submit_cognition_job(
        &self,
        key: &CognitionIdempotencyKey,
        owner: &str,
        typedid_request_digest: &str,
        max_attempts: u32,
        now: DateTime<Utc>,
    ) -> Result<CognitionJob, CognitionStateError> {
        validate_submission(key, owner, typedid_request_digest, max_attempts)?;
        if let Some((_, existing)) = self.load_job_node(key)? {
            if existing.typedid_request_digest != typedid_request_digest
                || existing.owner_digest != owner_digest(owner)
                || existing.max_attempts != max_attempts
            {
                return Err(CognitionStateError::DigestCollision);
            }
            return Ok(existing);
        }

        let job = CognitionJob {
            schema_version: super::state::JOB_SCHEMA_VERSION,
            job_digest: job_digest(key),
            owner_digest: owner_digest(owner),
            typedid_request_digest: typedid_request_digest.to_owned(),
            status: CognitionJobStatus::Pending,
            attempts: 0,
            max_attempts,
            revision: 0,
            lease: None,
            proposal_digest: None,
            completion_digest: None,
            last_error_digest: None,
            created_at: now,
            transitioned_at: now,
            progress: super::CognitionProgress::queued(now),
        };
        match self.create_job(key, &job) {
            Ok(()) => Ok(job),
            Err(CognitionStateError::ConcurrentModification) => {
                self.recover_submission(key, owner, typedid_request_digest, max_attempts)
            }
            Err(error) => Err(error),
        }
    }

    /// Load digest-safe durable job state.
    ///
    /// The scoped key is an address, not read authorization; the enclosing
    /// scheduler must authenticate and authorize access to operational state.
    ///
    /// # Errors
    ///
    /// Returns [`CognitionStateError`] when the job is absent or its
    /// persisted state fails validation.
    pub fn cognition_job(
        &self,
        key: &CognitionIdempotencyKey,
    ) -> Result<Option<CognitionJob>, CognitionStateError> {
        validate_job_key(key)?;
        Ok(self.load_job_node(key)?.map(|(_, job)| job))
    }

    /// Submit or recover a job, then either return its exclusive worker lease
    /// or the only durable identity recovery may use.
    ///
    /// In particular, a staged proposal is never leased again. That prevents a
    /// later process from re-planning protected input after response loss; it
    /// must instead use the stored digest with `TypeSec`'s proposal-free commit
    /// recovery path.
    ///
    /// # Errors
    ///
    /// Returns [`CognitionStateError`] when the job cannot be submitted, is
    /// not claimable in its current state, or fails lease validation.
    pub fn claim_cognition_job(
        &self,
        request: CognitionJobClaimRequest<'_>,
    ) -> Result<CognitionJobClaim, CognitionStateError> {
        let job = self.submit_cognition_job(
            request.key,
            request.submitter,
            request.typedid_request_digest,
            request.max_attempts,
            request.now,
        )?;
        match job.status {
            CognitionJobStatus::ProposalReady => Ok(CognitionJobClaim::ProposalReady {
                proposal_digest: job.proposal_digest.ok_or_else(|| {
                    CognitionStateError::Backend("staged job has no proposal digest".into())
                })?,
            }),
            CognitionJobStatus::Completed => Ok(CognitionJobClaim::Completed {
                completion_digest: job.completion_digest.ok_or_else(|| {
                    CognitionStateError::Backend("completed job has no commit digest".into())
                })?,
            }),
            CognitionJobStatus::Cancelled => Err(CognitionStateError::Terminal),
            CognitionJobStatus::Pending
            | CognitionJobStatus::Leased
            | CognitionJobStatus::Failed => self
                .acquire_cognition_lease(
                    request.key,
                    request.worker,
                    request.now,
                    request.lease_ttl,
                )
                .map(CognitionJobClaim::Lease),
        }
    }

    fn recover_submission(
        &self,
        key: &CognitionIdempotencyKey,
        owner: &str,
        typedid_request_digest: &str,
        max_attempts: u32,
    ) -> Result<CognitionJob, CognitionStateError> {
        let (_, existing) = self.load_job_node(key)?.ok_or_else(|| {
            CognitionStateError::Backend("job CAS conflicted without a job".into())
        })?;
        if existing.typedid_request_digest == typedid_request_digest
            && existing.owner_digest == owner_digest(owner)
            && existing.max_attempts == max_attempts
        {
            Ok(existing)
        } else {
            Err(CognitionStateError::DigestCollision)
        }
    }

    /// Acquire an exclusive lease with a fresh, unpersisted bearer token.
    ///
    /// `owner` identifies the authenticated worker and may deliberately differ
    /// from the submitting scheduler so expired work can move between workers.
    /// The enclosing service must authorize acquisition; the scoped key is not
    /// a credential.
    ///
    /// A lease is capped at one hour; long-running work must renew it.
    ///
    /// # Errors
    ///
    /// Returns [`CognitionStateError`] when the job is not leasable in its
    /// current state or the persisted state fails validation.
    pub fn acquire_cognition_lease(
        &self,
        key: &CognitionIdempotencyKey,
        owner: &str,
        now: DateTime<Utc>,
        ttl: Duration,
    ) -> Result<CognitionLease, CognitionStateError> {
        validate_identity(key, owner)?;
        let expires_at = lease_expiry(now, ttl)?;
        let (node, mut job) = self
            .load_job_node(key)?
            .ok_or(CognitionStateError::NotFound)?;
        validate_transition_time(&job, now)?;
        if job.status.is_terminal() {
            return Err(CognitionStateError::Terminal);
        }
        if job.status == CognitionJobStatus::ProposalReady {
            return Err(CognitionStateError::InvalidTransition(
                "a staged proposal must use proposal-free recovery".into(),
            ));
        }
        if job
            .lease
            .as_ref()
            .is_some_and(|lease| lease.expires_at > now)
        {
            return Err(CognitionStateError::LeaseHeld);
        }
        if job.attempts >= job.max_attempts {
            cancel_exhausted(&mut job, now)?;
            self.transition_job(key, node, &job)?;
            return Err(CognitionStateError::AttemptsExhausted);
        }

        advance_attempt(&mut job)?;
        advance_job(&mut job, now)?;
        job.status = CognitionJobStatus::Leased;
        let token = new_lease_token();
        job.lease = Some(CognitionLeaseState {
            owner_digest: owner_digest(owner),
            token_digest: token_digest(&token),
            acquired_at: now,
            expires_at,
        });
        self.transition_job(key, node, &job)?;
        Ok(CognitionLease::new(
            job.job_digest.clone(),
            token,
            job.attempts,
            expires_at,
        ))
    }

    /// Extend an active lease without changing its bearer token, for at most
    /// one hour from `now`.
    ///
    /// Possession of the unexpired token is the worker credential; owner text
    /// is intentionally not accepted again or used as substitute authority.
    ///
    /// # Errors
    ///
    /// Returns [`CognitionStateError`] when the lease token is stale or the
    /// persisted job state fails validation.
    pub fn renew_cognition_lease(
        &self,
        key: &CognitionIdempotencyKey,
        token: &str,
        now: DateTime<Utc>,
        ttl: Duration,
    ) -> Result<CognitionLease, CognitionStateError> {
        validate_job_key(key)?;
        validate_bearer_token(token)?;
        let expires_at = lease_expiry(now, ttl)?;
        let (node, mut job) = self
            .load_job_node(key)?
            .ok_or(CognitionStateError::NotFound)?;
        validate_transition_time(&job, now)?;
        validate_lease(&job, token, now)?;
        let current_expiry = job
            .lease
            .as_ref()
            .ok_or(CognitionStateError::StaleLease)?
            .expires_at;
        if expires_at <= current_expiry {
            return Err(CognitionStateError::InvalidTransition(
                "lease renewal must extend the current expiry".into(),
            ));
        }
        let lease = job.lease.as_mut().ok_or(CognitionStateError::StaleLease)?;
        lease.expires_at = expires_at;
        advance_job(&mut job, now)?;
        self.transition_job(key, node, &job)?;
        Ok(CognitionLease::new(
            job.job_digest.clone(),
            token.to_owned(),
            job.attempts,
            expires_at,
        ))
    }

    /// Update digest-safe progress while holding the active lease.
    ///
    /// The lease is validated against the caller-independent `now`, never the
    /// worker-supplied progress timestamp, so an expired lease cannot keep
    /// writing progress by backdating `updated_at`.
    ///
    /// # Errors
    ///
    /// Returns [`CognitionStateError`] when the lease is stale against
    /// trusted time or the progress payload violates its bounds.
    pub fn update_cognition_progress(
        &self,
        key: &CognitionIdempotencyKey,
        token: &str,
        progress: super::CognitionProgress,
        now: DateTime<Utc>,
    ) -> Result<CognitionJob, CognitionStateError> {
        validate_job_key(key)?;
        validate_bearer_token(token)?;
        progress.validate()?;
        let (node, mut job) = self
            .load_job_node(key)?
            .ok_or(CognitionStateError::NotFound)?;
        validate_transition_time(&job, now)?;
        validate_lease(&job, token, now)?;
        if progress.updated_at < job.progress.updated_at {
            return Err(CognitionStateError::InvalidTransition(
                "progress timestamp regressed".into(),
            ));
        }
        job.progress = progress;
        advance_job(&mut job, now)?;
        self.transition_job(key, node, &job)?;
        Ok(job)
    }

    /// Persist only a proposal's canonical identity and mark its job ready.
    ///
    /// Proposal drafts and evidence stay transient; cognition metadata never
    /// serializes their potentially protected content. The lease governs this
    /// worker staging step. Once staged, expiry does not authorize or forbid a
    /// mutation: only a freshly vault-authorized opaque `TypeSec` prepared commit
    /// can apply the proposal.
    ///
    /// This transition is bearer-authorized by the active lease token.
    ///
    /// # Errors
    ///
    /// Returns [`CognitionStateError`] when the lease is stale, the proposal
    /// digest is invalid, or the state transition is not permitted.
    pub fn persist_cognition_proposal(
        &self,
        key: &CognitionIdempotencyKey,
        token: &str,
        proposal: &CognitionProposal,
        now: DateTime<Utc>,
    ) -> Result<String, CognitionStateError> {
        validate_job_key(key)?;
        validate_bearer_token(token)?;
        let Some(binding) = proposal.binding.as_ref() else {
            return Err(CognitionStateError::Invalid(
                "proposal identity does not match scheduler job".into(),
            ));
        };
        if proposal.job_id != key.job_id()
            || binding.space_id != key.space_id()
            || !authority_scope_matches(key, &binding.subject, &binding.purpose)
        {
            return Err(CognitionStateError::Invalid(
                "proposal identity does not match scheduler job".into(),
            ));
        }
        let digest = proposal
            .canonical_digest()
            .map_err(|_| CognitionStateError::Serialization("proposal digest failed".into()))?;
        let (job_node, mut job) = self
            .load_job_node(key)?
            .ok_or(CognitionStateError::NotFound)?;
        validate_transition_time(&job, now)?;
        validate_lease(&job, token, now)?;
        if job.typedid_request_digest != binding.typedid_request_digest {
            return Err(CognitionStateError::DigestCollision);
        }

        if let Some(stored_digest) = job.proposal_digest.as_deref() {
            if stored_digest != digest {
                return Err(CognitionStateError::DigestCollision);
            }
            if job.status == CognitionJobStatus::ProposalReady {
                return Ok(digest);
            }
        }

        job.status = CognitionJobStatus::ProposalReady;
        job.proposal_digest = Some(digest.clone());
        advance_job(&mut job, now)?;
        self.transition_job(key, job_node, &job)?;
        Ok(digest)
    }

    /// Record a worker failure without persisting its plaintext.
    ///
    /// This transition is bearer-authorized by the active lease token.
    ///
    /// # Errors
    ///
    /// Returns [`CognitionStateError`] when the failure transition is not
    /// permitted from the current persisted state.
    pub fn fail_cognition_job(
        &self,
        key: &CognitionIdempotencyKey,
        token: &str,
        error: &str,
        now: DateTime<Utc>,
    ) -> Result<CognitionJob, CognitionStateError> {
        validate_job_key(key)?;
        validate_bearer_token(token)?;
        if !is_bounded_failure(error) {
            return Err(CognitionStateError::Invalid(
                "failure input exceeds its fixed bound or is empty".into(),
            ));
        }
        let (node, mut job) = self
            .load_job_node(key)?
            .ok_or(CognitionStateError::NotFound)?;
        validate_transition_time(&job, now)?;
        validate_lease(&job, token, now)?;
        job.status = if job.attempts >= job.max_attempts {
            CognitionJobStatus::Cancelled
        } else {
            CognitionJobStatus::Failed
        };
        job.last_error_digest = Some(error_digest(error));
        job.lease = None;
        advance_job(&mut job, now)?;
        self.transition_job(key, node, &job)?;
        Ok(job)
    }

    /// Cancel a non-terminal job using the submitting owner's identity.
    ///
    /// Matching the stored digest prevents cross-owner mistakes but does not
    /// authenticate raw owner text; the enclosing scheduler must authenticate
    /// and authorize cancellation before calling this storage primitive.
    ///
    /// # Errors
    ///
    /// Returns [`CognitionStateError`] when the cancel transition is not
    /// permitted from the current persisted state.
    pub fn cancel_cognition_job(
        &self,
        key: &CognitionIdempotencyKey,
        owner: &str,
        now: DateTime<Utc>,
    ) -> Result<CognitionJob, CognitionStateError> {
        validate_identity(key, owner)?;
        let (node, mut job) = self
            .load_job_node(key)?
            .ok_or(CognitionStateError::NotFound)?;
        validate_transition_time(&job, now)?;
        if job.owner_digest != owner_digest(owner) {
            return Err(CognitionStateError::StaleLease);
        }
        if job.status.is_terminal() {
            return Err(CognitionStateError::Terminal);
        }
        job.status = CognitionJobStatus::Cancelled;
        job.lease = None;
        advance_job(&mut job, now)?;
        self.transition_job(key, node, &job)?;
        Ok(job)
    }
}

fn validate_identity(
    key: &CognitionIdempotencyKey,
    owner: &str,
) -> Result<(), CognitionStateError> {
    validate_job_key(key)?;
    if !is_canonical_text(owner) {
        return Err(CognitionStateError::Invalid(
            "owner must be canonical text".into(),
        ));
    }
    Ok(())
}

fn validate_job_key(key: &CognitionIdempotencyKey) -> Result<(), CognitionStateError> {
    validate_idempotency_key(key).map_err(|message| CognitionStateError::Invalid(message.into()))
}

fn validate_submission(
    key: &CognitionIdempotencyKey,
    owner: &str,
    typedid_request_digest: &str,
    max_attempts: u32,
) -> Result<(), CognitionStateError> {
    validate_identity(key, owner)?;
    if !is_sha256(typedid_request_digest) || max_attempts == 0 {
        return Err(CognitionStateError::Invalid(
            "TypeDID request digest must be canonical SHA-256 and max attempts must be positive"
                .into(),
        ));
    }
    Ok(())
}
