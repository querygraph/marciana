//! Marciana-owned cognition composition primitives.

mod binding_error;
mod engine_binding;
mod governed;
mod memory_error;

pub use binding_error::CognitionBindingError;
pub use engine_binding::CognitionEngineBinding;
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use governed::CognitionClock;
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use governed::intent_claim_limits_for_test;
pub use governed::{
    CLAIM_ALGORITHM, CLAIM_ALGORITHM_VERSION, CLAIM_CATALOG_IDENTITY, CLAIM_FIELD_MAPPING_DIGEST,
    CLAIM_GRANT_ID, CLAIM_INTENT_VERSION, CLAIM_JOB_ID, CLAIM_OPERATION,
    CLAIM_SOURCE_SELECTION_DIGEST, COGNITION_ACTION, COGNITION_INTENT_VERSION,
    CognitionApplicationError, FreshLakeCatAuthority, GovernedCognitionApplication,
    GovernedCognitionConfig, GovernedCognitionResult, LakeCatAuthorityError,
    LakeCatCognitionAuthority, cognition_field_mapping_digest, cognition_source_selection_digest,
};
pub use memory_error::CognitionMemoryError;

#[cfg(test)]
mod binding_error_tests;
#[cfg(test)]
mod engine_binding_tests;
#[cfg(test)]
mod memory_error_tests;
