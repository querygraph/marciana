//! Complete governed cognition composition.
#![allow(clippy::doc_markdown, clippy::missing_errors_doc)]

mod application;
mod authority;
mod binding;
mod clock;
mod governed_proof;
mod intent;
mod projection;
mod proposal;
mod receipt;

#[cfg(test)]
mod intent_tests;

pub use application::{
    CognitionApplicationError, GovernedCognitionApplication, GovernedCognitionConfig,
    GovernedCognitionResult,
};
pub use authority::{FreshLakeCatAuthority, LakeCatAuthorityError, LakeCatCognitionAuthority};
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use authority::{PrimedAuthorityVerifier, current_policy_decision_id};
pub use intent::{
    CLAIM_ALGORITHM, CLAIM_ALGORITHM_VERSION, CLAIM_CATALOG_IDENTITY, CLAIM_FIELD_MAPPING_DIGEST,
    CLAIM_GRANT_ID, CLAIM_INTENT_VERSION, CLAIM_JOB_ID, CLAIM_OPERATION,
    CLAIM_SOURCE_SELECTION_DIGEST, COGNITION_ACTION, COGNITION_INTENT_VERSION,
    cognition_field_mapping_digest, cognition_source_selection_digest,
};
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use intent::{CONTEXT_REQUEST_DIGEST, CONTEXT_SUBJECT};

#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use clock::CognitionClock;

#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use intent::intent_claim_limits_for_test;
