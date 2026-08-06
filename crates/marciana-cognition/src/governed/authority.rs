//! Fresh LakeCat authority revalidation adapted to TypeSec's vault seam.

use std::sync::{Arc, Mutex};

use crate::CognitionBindingError;
use async_trait::async_trait;
use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use lakecat_core::governed_scan::{
    GovernedScanCatalogIdentity, GovernedScanProof, governed_evidence_digest,
    validate_governed_sha256_digest,
};
use serde_json::json;
use typesec_core::policy::RequestContext;
use typesec_memory::{
    CognitionAuthorityError, CognitionAuthorityEvidence, CognitionAuthorityVerifier,
    CognitionBinding,
};

use super::binding::BindingBasis;
use super::clock::CognitionClock;
use super::intent::{
    CLAIM_ALGORITHM, CLAIM_ALGORITHM_VERSION, CLAIM_JOB_ID, CONTEXT_REQUEST_DIGEST,
    CONTEXT_SUBJECT, ensure_active,
};

const CURRENT_POLICY_DECISION_DOMAIN: &str = "marciana.current-policy-decision.digest.v2";

/// Fresh result from a LakeCat service revalidation.
#[derive(Debug, Clone)]
pub struct FreshLakeCatAuthority {
    /// Catalog identity asserted by the service performing revalidation.
    pub catalog_identity: GovernedScanCatalogIdentity,
    /// Original persisted grant after LakeCat revalidated current state.
    pub proof: GovernedScanProof,
    /// Grant identity LakeCat currently recognizes for this authorization.
    pub current_grant_id: String,
    /// Current immutable catalog snapshot identity.
    pub current_snapshot_digest: String,
    /// Current policy-narrowed projection.
    pub current_effective_projection: Vec<String>,
    /// Fresh authorization digest; never substituted for the original receipt.
    pub fresh_authorization_digest: String,
    /// Fresh policy digest; combined with authorization into current evidence.
    pub fresh_policy_decision_digest: String,
    /// Authenticated time at which LakeCat completed this revalidation.
    pub revalidated_at: DateTime<Utc>,
}

/// Sanitized LakeCat authority failure with no adapter-controlled text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LakeCatAuthorityError {
    /// The durable grant is revoked, stale, or otherwise no longer authorized.
    #[error("LakeCat denied the governed scan grant")]
    Denied,
    /// LakeCat could not complete a fresh authority check.
    #[error("LakeCat authority is unavailable")]
    Unavailable,
}

/// Host port implemented by the LakeCat service boundary.
#[async_trait]
pub trait LakeCatCognitionAuthority: Send + Sync {
    /// Stable catalog identity owned by this LakeCat authority endpoint.
    fn catalog_identity(&self) -> &GovernedScanCatalogIdentity;

    /// Freshly revalidate a persisted governed scan grant.
    async fn revalidate(
        &self,
        presented: &GovernedScanProof,
    ) -> Result<FreshLakeCatAuthority, LakeCatAuthorityError>;
}

pub(crate) struct PrimedAuthorityVerifier {
    primed: Mutex<Option<PrimedAuthority>>,
    clock: Arc<dyn CognitionClock>,
}

struct PrimedAuthority {
    evidence: CognitionAuthorityEvidence,
    effective_expires_at: u64,
}

impl PrimedAuthorityVerifier {
    pub(crate) fn new(clock: Arc<dyn CognitionClock>) -> Self {
        Self {
            primed: Mutex::new(None),
            clock,
        }
    }

    pub(crate) fn prime(
        &self,
        evidence: CognitionAuthorityEvidence,
        effective_expires_at: u64,
    ) -> Result<(), CognitionAuthorityError> {
        let mut slot = self.primed.lock().map_err(lock_error)?;
        if slot.is_some() {
            return Err(CognitionAuthorityError::Unavailable);
        }
        *slot = Some(PrimedAuthority {
            evidence,
            effective_expires_at,
        });
        Ok(())
    }

    pub(crate) fn clear(&self) -> Result<(), CognitionAuthorityError> {
        self.primed.lock().map_err(lock_error)?.take();
        Ok(())
    }
}

impl CognitionAuthorityVerifier for PrimedAuthorityVerifier {
    fn revalidate(
        &self,
        binding: &CognitionBinding,
        context: &RequestContext,
    ) -> Result<CognitionAuthorityEvidence, CognitionAuthorityError> {
        let primed = self
            .primed
            .lock()
            .map_err(lock_error)?
            .take()
            .ok_or(CognitionAuthorityError::Unavailable)?;
        if ensure_active(primed.effective_expires_at, self.clock.now()).is_err() {
            return Err(CognitionAuthorityError::Unavailable);
        }
        let evidence = primed.evidence;
        if context.purpose.as_deref() != Some(evidence.purpose.as_str())
            || context.custom.get(CONTEXT_SUBJECT) != Some(&evidence.subject)
            || context.custom.get(CONTEXT_REQUEST_DIGEST) != Some(&evidence.typedid_request_digest)
            || context.custom.get(CLAIM_JOB_ID) != Some(&evidence.job_id)
            || context.custom.get(CLAIM_ALGORITHM) != Some(&evidence.algorithm)
            || context.custom.get(CLAIM_ALGORITHM_VERSION) != Some(&evidence.algorithm_version)
            || binding.governed_source_scope != evidence.governed_source_scope
            || binding.typedid_request_digest != evidence.typedid_request_digest
        {
            return Err(CognitionAuthorityError::Unavailable);
        }
        Ok(evidence)
    }
}

fn lock_error<T>(_error: std::sync::PoisonError<T>) -> CognitionAuthorityError {
    CognitionAuthorityError::Unavailable
}

pub(crate) fn validate_fresh_authority(
    basis: &BindingBasis,
    fresh: &FreshLakeCatAuthority,
    now: DateTime<Utc>,
    max_age: TimeDelta,
    future_skew: TimeDelta,
) -> Result<String, CognitionBindingError> {
    let oldest = now
        .checked_sub_signed(max_age)
        .ok_or(CognitionBindingError::InvalidAuthorityFreshness)?;
    let latest = now
        .checked_add_signed(future_skew)
        .ok_or(CognitionBindingError::InvalidAuthorityFreshness)?;
    if fresh.revalidated_at < oldest {
        return Err(CognitionBindingError::StaleAuthorityEvidence);
    }
    if fresh.revalidated_at > latest {
        return Err(CognitionBindingError::FutureAuthorityEvidence);
    }
    for (digest, label) in [
        (fresh.current_grant_id.as_str(), "current grant"),
        (fresh.current_snapshot_digest.as_str(), "current snapshot"),
        (
            fresh.fresh_authorization_digest.as_str(),
            "fresh authorization",
        ),
        (
            fresh.fresh_policy_decision_digest.as_str(),
            "fresh policy decision",
        ),
    ] {
        validate_governed_sha256_digest(digest, label)
            .map_err(|_| CognitionBindingError::InvalidAuthorityDigest)?;
    }
    basis.validate_fresh_proof(&fresh.catalog_identity, &fresh.proof)?;
    if fresh.current_grant_id != basis.proof.grant_id() {
        return Err(CognitionBindingError::GrantMismatch);
    }
    if fresh.current_snapshot_digest != basis.snapshot_identity() {
        return Err(CognitionBindingError::SnapshotMismatch);
    }
    basis.verify_current_projection(&fresh.current_effective_projection)?;
    current_policy_decision_id(
        &fresh.fresh_authorization_digest,
        &fresh.fresh_policy_decision_digest,
        &fresh.revalidated_at,
    )
}

pub(crate) fn current_policy_decision_id(
    fresh_authorization_digest: &str,
    fresh_policy_decision_digest: &str,
    provider_revalidated_at: &DateTime<Utc>,
) -> Result<String, CognitionBindingError> {
    for (digest, label) in [
        (fresh_authorization_digest, "fresh authorization"),
        (fresh_policy_decision_digest, "fresh policy decision"),
    ] {
        validate_governed_sha256_digest(digest, label)
            .map_err(|_| CognitionBindingError::InvalidAuthorityDigest)?;
    }
    governed_evidence_digest(
        CURRENT_POLICY_DECISION_DOMAIN,
        &json!({
            "version": CURRENT_POLICY_DECISION_DOMAIN,
            "freshAuthorizationDigest": fresh_authorization_digest,
            "freshPolicyDecisionDigest": fresh_policy_decision_digest,
            "providerRevalidatedAt": provider_revalidated_at
                .to_rfc3339_opts(SecondsFormat::Nanos, true),
        }),
    )
    .map_err(|_| CognitionBindingError::Digest)
}
