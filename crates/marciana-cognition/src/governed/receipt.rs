//! Deterministic signed receipts built from committed TypeSec evidence.

use std::sync::OnceLock;

use crate::CognitionBindingError;
use chrono::{DateTime, TimeDelta, Utc};
use lakecat_core::governed_scan::governed_evidence_digest;
use serde_json::json;
use typesec_integrations::{
    CognitionCommitReceipt, CognitionCommitReceiptClaims, ReceiptError, ReceiptIssuer,
};
use typesec_memory::{
    CognitionAuthorityEvidence, CognitionCommitOutcome, CognitionCommitStatus, CognitionEffect,
    CognitionProposal,
};

use super::binding::BindingBasis;
use super::proposal::ValidatedProposalIdentity;

const POLICY_DECISION_DOMAIN: &str = "marciana.policy-decision.digest.v1";

pub(crate) struct ReceiptBasis {
    effect: CognitionEffect,
    proposal_digest: String,
    binding_digest: String,
    source_manifest_digest: String,
    job_id: String,
    subject: String,
    space_id: String,
    purpose: String,
    request_digest: String,
    governed_source_scope: String,
    governed_scan_digest: String,
    snapshot_digest: String,
    authorization_receipt_digest: String,
    current_policy_decision_id: String,
}

impl ReceiptBasis {
    pub(crate) fn new(
        proposal: &CognitionProposal,
        basis: &BindingBasis,
        proposal_identity: &ValidatedProposalIdentity,
        authority: &CognitionAuthorityEvidence,
    ) -> Result<Self, CognitionBindingError> {
        let binding = proposal
            .binding
            .as_ref()
            .ok_or(CognitionBindingError::MissingProposalBinding)?;
        basis.verify_binding(binding)?;
        let governed_source_scope = binding
            .governed_source_scope
            .as_ref()
            .ok_or(CognitionBindingError::ProposalIntentMismatch)?;
        if authority.space_id != binding.space_id
            || authority.subject != binding.subject
            || authority.purpose != binding.purpose
            || authority.governed_source_scope.as_ref() != Some(governed_source_scope)
            || authority.job_id != proposal.job_id
            || authority.algorithm != proposal.algorithm
            || authority.algorithm_version != proposal.algorithm_version
            || authority.governed_scan_digest != binding.governed_scan_digest
            || authority.snapshot_digest != binding.snapshot_digest
            || authority.plan_task_digest != binding.plan_task_digest
            || authority.authorization_receipt_digest != binding.authorization_receipt_digest
            || authority.effective_projection != binding.effective_projection
            || authority.typedid_request_digest != binding.typedid_request_digest
            || authority.policy_decision_id.trim().is_empty()
        {
            return Err(CognitionBindingError::FreshProofMismatch);
        }
        Ok(Self {
            effect: proposal.effect,
            proposal_digest: proposal_identity.proposal_digest().to_owned(),
            binding_digest: binding
                .canonical_digest()
                .map_err(|_| CognitionBindingError::Digest)?,
            source_manifest_digest: binding.source_manifest_digest.clone(),
            job_id: basis.intent.job_id.clone(),
            subject: basis.intent.subject.clone(),
            space_id: basis.space_id.clone(),
            purpose: basis.intent.purpose.clone(),
            request_digest: basis.intent.request_digest.clone(),
            governed_source_scope: governed_source_scope.as_str().to_owned(),
            governed_scan_digest: binding.governed_scan_digest.clone(),
            snapshot_digest: binding.snapshot_digest.clone(),
            authorization_receipt_digest: binding.authorization_receipt_digest.clone(),
            current_policy_decision_id: authority.policy_decision_id.clone(),
        })
    }
}

/// Request-bound, process-local projection of the first issued receipt.
///
/// The standalone Marciana service must replace this slot with an idempotent,
/// guarded issuance record. This transitional QueryGraph boundary guarantees
/// byte-identical replay only while this application instance remains alive.
pub(crate) struct CognitionReceiptSigner {
    issuer: ReceiptIssuer,
    ttl: TimeDelta,
    issued: OnceLock<CognitionCommitReceipt>,
}

impl CognitionReceiptSigner {
    pub(crate) fn new(issuer: ReceiptIssuer, ttl: TimeDelta) -> Self {
        Self {
            issuer,
            ttl,
            issued: OnceLock::new(),
        }
    }

    pub(crate) fn sign(
        &self,
        outcome: &CognitionCommitOutcome,
        basis: &ReceiptBasis,
        now: DateTime<Utc>,
    ) -> Result<(CognitionCommitReceipt, String), ReceiptError> {
        validate_commit_evidence(outcome, basis)?;
        if let Some(stored) = self.issued.get() {
            return self.sign_stored(stored, outcome, basis, now);
        }

        self.precheck_first_issuance(outcome, now)?;
        let receipt = self.project(outcome, basis, now)?;
        let token = self.issuer.issue_cognition(&receipt, now)?;
        if let Ok(()) = self.issued.set(receipt.clone()) {
            Ok((receipt, token))
        } else {
            let stored = self.issued.get().ok_or_else(|| {
                ReceiptError::InvalidClaims(
                    "cognition receipt issuance projection is unavailable".into(),
                )
            })?;
            self.sign_stored(stored, outcome, basis, now)
        }
    }

    fn sign_stored(
        &self,
        stored: &CognitionCommitReceipt,
        outcome: &CognitionCommitOutcome,
        basis: &ReceiptBasis,
        now: DateTime<Utc>,
    ) -> Result<(CognitionCommitReceipt, String), ReceiptError> {
        let rebuilt = self.project(outcome, basis, stored.issued_at())?;
        if &rebuilt != stored {
            return invalid("stored cognition receipt differs from current immutable evidence");
        }
        let token = self.issuer.issue_cognition(stored, now)?;
        Ok((stored.clone(), token))
    }

    fn precheck_first_issuance(
        &self,
        outcome: &CognitionCommitOutcome,
        now: DateTime<Utc>,
    ) -> Result<(), ReceiptError> {
        if outcome.committed_at > now {
            return Err(ReceiptError::NotYetValid {
                issued_at: outcome.committed_at,
                now,
            });
        }
        let expires_at = outcome
            .audit
            .prepared_at
            .checked_add_signed(self.ttl)
            .ok_or_else(|| {
                ReceiptError::InvalidClaims("cognition receipt expiry overflow".into())
            })?;
        if now >= expires_at {
            return Err(ReceiptError::Expired { expires_at, now });
        }
        Ok(())
    }

    fn project(
        &self,
        outcome: &CognitionCommitOutcome,
        _basis: &ReceiptBasis,
        issued_at: DateTime<Utc>,
    ) -> Result<CognitionCommitReceipt, ReceiptError> {
        let audit = &outcome.audit;
        CognitionCommitReceipt::new(
            CognitionCommitReceiptClaims {
                effect: outcome.effect,
                subject: audit.subject.clone(),
                resource: audit.space_id.clone(),
                job_id: audit.operation_id.clone(),
                governed_source_scope: audit
                    .governed_source_scope
                    .as_ref()
                    .map(|scope| scope.as_str().to_owned()),
                typedid_request_digest: audit.typedid_request_digest.clone(),
                proposal_digest: audit.proposal_digest.clone(),
                governed_scan_digest: audit.governed_scan_digest.clone(),
                input_snapshot_digest: audit.snapshot_digest.clone(),
                policy_decision_digest: policy_decision_digest(&audit.policy_decision_id)?,
                authorization_receipt_digest: audit.authorization_receipt_digest.clone(),
                prior_version: outcome.prior_version.clone(),
                resulting_version: outcome.resulting_version.clone(),
                affected_ids: outcome
                    .affected_ids
                    .iter()
                    .map(|id| id.as_str().to_owned())
                    .collect(),
                backend_commit_id: outcome.backend_commit_hash.clone(),
                authority_revalidated_at: audit.authority_revalidated_at,
                prepared_at: audit.prepared_at,
                committed_at: outcome.committed_at,
                issued_at,
            },
            self.ttl,
        )
    }
}

fn validate_commit_evidence(
    outcome: &CognitionCommitOutcome,
    basis: &ReceiptBasis,
) -> Result<(), ReceiptError> {
    let audit = &outcome.audit;
    if outcome.effect != basis.effect
        || audit.effect != basis.effect
        || audit.proposal_digest != basis.proposal_digest
        || audit.binding_digest != basis.binding_digest
        || audit.source_manifest_digest != basis.source_manifest_digest
        || audit.operation_id != basis.job_id
        || audit.subject != basis.subject
        || audit.space_id != basis.space_id
        || audit.purpose != basis.purpose
        || audit
            .governed_source_scope
            .as_ref()
            .map(typesec_memory::GovernedSourceScope::as_str)
            != Some(basis.governed_source_scope.as_str())
        || audit.typedid_request_digest != basis.request_digest
        || audit.governed_scan_digest != basis.governed_scan_digest
        || audit.snapshot_digest != basis.snapshot_digest
        || audit.authorization_receipt_digest != basis.authorization_receipt_digest
        // Fresh authority gates every call, but an idempotent replay must
        // preserve the policy decision atomically committed by the first call.
        || (outcome.status == CognitionCommitStatus::Applied
            && audit.policy_decision_id != basis.current_policy_decision_id)
    {
        return invalid("committed audit evidence differs from the application basis");
    }
    Ok(())
}

fn policy_decision_digest(decision_id: &str) -> Result<String, ReceiptError> {
    if decision_id.trim().is_empty() {
        return invalid("committed policy decision id is empty");
    }
    governed_evidence_digest(
        POLICY_DECISION_DOMAIN,
        &json!({ "version": POLICY_DECISION_DOMAIN, "decisionId": decision_id }),
    )
    .map_err(|_| ReceiptError::InvalidClaims("policy decision digest failed".into()))
}

fn invalid<T>(message: impl Into<String>) -> Result<T, ReceiptError> {
    Err(ReceiptError::InvalidClaims(message.into()))
}
