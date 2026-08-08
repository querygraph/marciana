//! Canonical signed TypeDID cognition intent.

use std::collections::BTreeMap;

use crate::{CognitionBindingError, FormationProfile};
use chrono::{DateTime, Utc};
use lakecat_core::governed_scan::{GovernedScanProof, governed_evidence_digest};
use querygraph_memory::cognition::{
    CognitionFieldMapping, CognitionOperation, MAX_COGNITION_IDENTITY_BYTES,
};
use serde_json::json;
use typesec_core::Resource;
use typesec_core::policy::RequestContext;
use typesec_integrations::VerifiedTypeDidContext;
use typesec_memory::{
    CognitionSourceBudget, Label, MAX_COGNITION_SOURCE_COUNT, MemoryId, MemorySpace,
};

/// Exact signed action accepted by the Marciana cognition boundary.
pub const COGNITION_ACTION: &str = "memory:improve";
/// Canonical version value for the signed claim set.
pub const COGNITION_INTENT_VERSION: &str = "marciana.cognition-intent.v3";
/// Required signed intent-version claim.
pub const CLAIM_INTENT_VERSION: &str = "marciana.intent.version";
/// Required signed durable job identity.
pub const CLAIM_JOB_ID: &str = "marciana.job.id";
/// Required signed native operation identity.
pub const CLAIM_OPERATION: &str = "marciana.operation";
/// Required signed versioned formation-profile identity.
pub const CLAIM_FORMATION_PROFILE: &str = "marciana.formation-profile";
/// Required signed native cognition algorithm identity.
pub const CLAIM_ALGORITHM: &str = "marciana.algorithm";
/// Required signed native cognition algorithm version.
pub const CLAIM_ALGORITHM_VERSION: &str = "marciana.algorithm.version";
/// Required signed digest of the canonical source selection.
pub const CLAIM_SOURCE_SELECTION_DIGEST: &str = "marciana.source-selection.digest";
/// Required signed LakeCat catalog identity.
pub const CLAIM_CATALOG_IDENTITY: &str = "marciana.catalog.identity";
/// Required signed LakeCat governed grant identity.
pub const CLAIM_GRANT_ID: &str = "marciana.grant.id";
/// Required signed digest of the exact field mapping.
pub const CLAIM_FIELD_MAPPING_DIGEST: &str = "marciana.field-mapping.digest";

const SOURCE_SELECTION_DOMAIN: &str = "marciana.source-selection.digest.v1";
const FIELD_MAPPING_DOMAIN: &str = "marciana.field-mapping.digest.v1";
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub const CONTEXT_SUBJECT: &str = "marciana.typedid.subject";
#[cfg(not(feature = "test-support"))]
pub(crate) const CONTEXT_SUBJECT: &str = "marciana.typedid.subject";
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub const CONTEXT_REQUEST_DIGEST: &str = "marciana.typedid.request-digest";
#[cfg(not(feature = "test-support"))]
pub(crate) const CONTEXT_REQUEST_DIGEST: &str = "marciana.typedid.request-digest";
pub(crate) const CONTEXT_REQUESTED_PRIVACY: &str = "marciana.requested-privacy";

const MAX_COGNITION_INTENT_CLAIMS: usize = 64;
const MAX_COGNITION_INTENT_CLAIM_BYTES: usize = 64 * 1024;
pub(super) const POLICY_INTENT_CLAIMS: [&str; 10] = [
    CLAIM_INTENT_VERSION,
    CLAIM_JOB_ID,
    CLAIM_OPERATION,
    CLAIM_FORMATION_PROFILE,
    CLAIM_ALGORITHM,
    CLAIM_ALGORITHM_VERSION,
    CLAIM_SOURCE_SELECTION_DIGEST,
    CLAIM_CATALOG_IDENTITY,
    CLAIM_GRANT_ID,
    CLAIM_FIELD_MAPPING_DIGEST,
];

#[derive(Debug, Clone)]
pub(crate) struct CognitionIntent {
    pub(crate) subject: String,
    pub(crate) purpose: String,
    pub(crate) request_digest: String,
    pub(crate) effective_expires_at: u64,
    pub(crate) job_id: String,
    pub(crate) operation: CognitionOperation,
    pub(crate) algorithm: String,
    pub(crate) algorithm_version: String,
    pub(crate) requested_clearance: Label,
    policy_claims: BTreeMap<String, String>,
}

#[derive(Clone, Copy)]
pub(crate) struct IntentInputs<'a> {
    pub(crate) space: &'a MemorySpace,
    pub(crate) catalog: &'a str,
    pub(crate) proof: &'a GovernedScanProof,
    pub(crate) source_ids: &'a [MemoryId],
    pub(crate) field_mapping: &'a CognitionFieldMapping,
    pub(crate) formation_profile: FormationProfile,
}

impl CognitionIntent {
    pub(crate) fn from_verified(
        verified: VerifiedTypeDidContext<'_>,
        inputs: IntentInputs<'_>,
        now: DateTime<Utc>,
    ) -> Result<Self, CognitionBindingError> {
        if verified.action() != COGNITION_ACTION {
            return Err(CognitionBindingError::ActionMismatch);
        }
        if verified.resource() != inputs.space.resource_id() {
            return Err(CognitionBindingError::ResourceMismatch);
        }
        let clearance = Label::from_name(verified.privacy());
        if clearance.name() != verified.privacy() {
            return Err(CognitionBindingError::InvalidPrivacy);
        }
        let purpose = verified
            .purpose()
            .filter(|purpose| !purpose.trim().is_empty())
            .ok_or(CognitionBindingError::MissingPurpose)?;
        if !is_canonical_identity(verified.subject().as_str())
            || !is_canonical_identity(inputs.space.resource_id())
            || !is_canonical_identity(purpose)
        {
            return Err(CognitionBindingError::InvalidIdentity);
        }
        ensure_active(verified.effective_expires_at(), now)?;

        let claims = verified.claims();
        validate_claims(claims)?;
        require_claim(claims, CLAIM_INTENT_VERSION, COGNITION_INTENT_VERSION)?;
        let job_id = required_nonblank_claim(claims, CLAIM_JOB_ID)?;
        if !is_canonical_identity(job_id) {
            return Err(CognitionBindingError::IntentClaimMismatch(CLAIM_JOB_ID));
        }
        let operation: CognitionOperation = required_nonblank_claim(claims, CLAIM_OPERATION)?
            .parse()
            .map_err(|_| CognitionBindingError::InvalidOperation)?;
        let formation_profile: FormationProfile =
            required_nonblank_claim(claims, CLAIM_FORMATION_PROFILE)?
                .parse()
                .map_err(|_| CognitionBindingError::IntentClaimMismatch(CLAIM_FORMATION_PROFILE))?;
        if formation_profile != inputs.formation_profile
            || formation_profile.operation() != operation
        {
            return Err(CognitionBindingError::IntentClaimMismatch(
                CLAIM_FORMATION_PROFILE,
            ));
        }
        let algorithm = required_nonblank_claim(claims, CLAIM_ALGORITHM)?;
        let algorithm_version = required_nonblank_claim(claims, CLAIM_ALGORITHM_VERSION)?;
        if !operation.is_native_algorithm(algorithm, algorithm_version) {
            return Err(CognitionBindingError::InvalidAlgorithm);
        }
        require_claim(
            claims,
            CLAIM_SOURCE_SELECTION_DIGEST,
            &cognition_source_selection_digest(inputs.source_ids)?,
        )?;
        require_claim(claims, CLAIM_CATALOG_IDENTITY, inputs.catalog)?;
        require_claim(claims, CLAIM_GRANT_ID, inputs.proof.grant_id())?;
        require_claim(
            claims,
            CLAIM_FIELD_MAPPING_DIGEST,
            &cognition_field_mapping_digest(inputs.field_mapping)?,
        )?;

        Ok(Self {
            subject: verified.subject().to_string(),
            purpose: purpose.to_owned(),
            request_digest: verified.request_digest().to_owned(),
            effective_expires_at: verified.effective_expires_at(),
            job_id: job_id.to_owned(),
            operation,
            algorithm: algorithm.to_owned(),
            algorithm_version: algorithm_version.to_owned(),
            requested_clearance: clearance,
            policy_claims: policy_claims(claims),
        })
    }

    pub(crate) fn ensure_active(&self, now: DateTime<Utc>) -> Result<(), CognitionBindingError> {
        ensure_active(self.effective_expires_at, now)
    }

    pub(crate) fn policy_context(&self) -> RequestContext {
        let mut context = RequestContext::new().with_purpose(self.purpose.clone());
        for (claim, value) in &self.policy_claims {
            context = context.with(claim.clone(), value.clone());
        }
        context
            .with(CONTEXT_SUBJECT, self.subject.clone())
            .with(CONTEXT_REQUEST_DIGEST, self.request_digest.clone())
            .with(CONTEXT_REQUESTED_PRIVACY, self.requested_clearance.name())
    }
}

pub(crate) fn canonical_source_ids(
    source_ids: &[MemoryId],
) -> Result<Vec<MemoryId>, CognitionBindingError> {
    if source_ids.is_empty() || source_ids.len() > MAX_COGNITION_SOURCE_COUNT {
        return Err(CognitionBindingError::InvalidSourceSelection);
    }
    let mut budget = CognitionSourceBudget::new();
    if source_ids
        .iter()
        .any(|id| !is_canonical_identity(id.as_str()) || budget.try_add(id.as_str(), "").is_err())
    {
        return Err(CognitionBindingError::InvalidSourceSelection);
    }
    let mut canonical = source_ids.to_vec();
    canonical.sort();
    let original_len = canonical.len();
    canonical.dedup();
    if canonical.len() != original_len {
        return Err(CognitionBindingError::InvalidSourceSelection);
    }
    Ok(canonical)
}

/// Hash the sorted, unique, non-empty source selection used by Marciana.
pub fn cognition_source_selection_digest(
    source_ids: &[MemoryId],
) -> Result<String, CognitionBindingError> {
    let canonical = canonical_source_ids(source_ids)?;
    let values: Vec<_> = canonical.iter().map(MemoryId::as_str).collect();
    governed_evidence_digest(
        SOURCE_SELECTION_DOMAIN,
        &json!({ "version": SOURCE_SELECTION_DOMAIN, "sourceIds": values }),
    )
    .map_err(|_| CognitionBindingError::Digest)
}

/// Hash the exact LakeCat-to-cognition field mapping used by Marciana.
pub fn cognition_field_mapping_digest(
    mapping: &CognitionFieldMapping,
) -> Result<String, CognitionBindingError> {
    let fields = [&mapping.id, &mapping.text, &mapping.valid_from];
    if fields.iter().any(|field| !is_canonical_identity(field))
        || fields
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != 3
    {
        return Err(CognitionBindingError::InvalidProjection);
    }
    governed_evidence_digest(
        FIELD_MAPPING_DOMAIN,
        &json!({
            "version": FIELD_MAPPING_DOMAIN,
            "id": mapping.id,
            "text": mapping.text,
            "validFrom": mapping.valid_from,
        }),
    )
    .map_err(|_| CognitionBindingError::Digest)
}

pub(crate) fn is_canonical_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_COGNITION_IDENTITY_BYTES
        && value == value.trim()
        && !value.chars().any(char::is_control)
}

fn policy_claims(claims: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    // Sender authentication proves who asserted a claim, not that arbitrary
    // authorization attributes such as roles or organizations are true. Only
    // intent values independently validated by this boundary reach policy.
    POLICY_INTENT_CLAIMS
        .into_iter()
        .filter_map(|name| {
            claims
                .get(name)
                .map(|value| (name.to_owned(), value.clone()))
        })
        .collect()
}

pub(crate) fn validate_claims(
    claims: &BTreeMap<String, String>,
) -> Result<(), CognitionBindingError> {
    if claims.len() > MAX_COGNITION_INTENT_CLAIMS {
        return Err(CognitionBindingError::InvalidClaims);
    }
    let bytes = claims.iter().try_fold(0usize, |total, (name, value)| {
        if !is_canonical_identity(name) || !is_canonical_identity(value) {
            return None;
        }
        total.checked_add(name.len())?.checked_add(value.len())
    });
    if bytes.is_none_or(|bytes| bytes > MAX_COGNITION_INTENT_CLAIM_BYTES) {
        return Err(CognitionBindingError::InvalidClaims);
    }
    Ok(())
}

#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
#[must_use]
#[allow(
    dead_code,
    reason = "the library test target cannot observe integration-test consumers"
)]
pub const fn intent_claim_limits_for_test() -> (usize, usize) {
    (
        MAX_COGNITION_INTENT_CLAIMS,
        MAX_COGNITION_INTENT_CLAIM_BYTES,
    )
}

fn require_claim(
    claims: &BTreeMap<String, String>,
    name: &'static str,
    expected: &str,
) -> Result<(), CognitionBindingError> {
    if claims.get(name).map(String::as_str) != Some(expected) {
        return Err(CognitionBindingError::IntentClaimMismatch(name));
    }
    Ok(())
}

fn required_nonblank_claim<'a>(
    claims: &'a BTreeMap<String, String>,
    name: &'static str,
) -> Result<&'a str, CognitionBindingError> {
    claims
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or(CognitionBindingError::IntentClaimMismatch(name))
}

pub(crate) fn ensure_active(expiry: u64, now: DateTime<Utc>) -> Result<(), CognitionBindingError> {
    let timestamp =
        u64::try_from(now.timestamp()).map_err(|_| CognitionBindingError::RequestExpired)?;
    if timestamp >= expiry {
        return Err(CognitionBindingError::RequestExpired);
    }
    Ok(())
}
