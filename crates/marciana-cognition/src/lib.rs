//! Marciana-owned cognition composition primitives.

mod binding_error;
mod engine_binding;
mod formation_profile;
mod governed;
mod health;
mod learning;
mod memory_error;
mod metrics;

pub use binding_error::CognitionBindingError;
pub use engine_binding::CognitionEngineBinding;
pub use formation_profile::{FormationBinding, FormationProfile, FormationProvider};
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use governed::CognitionClock;
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use governed::intent_claim_limits_for_test;
pub use governed::{
    CLAIM_ALGORITHM, CLAIM_ALGORITHM_VERSION, CLAIM_CATALOG_IDENTITY, CLAIM_FIELD_MAPPING_DIGEST,
    CLAIM_FORMATION_PROFILE, CLAIM_GRANT_ID, CLAIM_INTENT_VERSION, CLAIM_JOB_ID, CLAIM_OPERATION,
    CLAIM_SOURCE_SELECTION_DIGEST, COGNITION_ACTION, COGNITION_INTENT_VERSION,
    CognitionApplicationError, FreshLakeCatAuthority, GovernedCognitionApplication,
    GovernedCognitionConfig, GovernedCognitionResult, LakeCatAuthorityError,
    LakeCatCognitionAuthority, cognition_field_mapping_digest, cognition_source_selection_digest,
};
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use governed::{
    CONTEXT_REQUEST_DIGEST, CONTEXT_SUBJECT, PrimedAuthorityVerifier, current_policy_decision_id,
};
pub use health::{ComponentHealth, ComponentState, HealthError, HealthSnapshot};
pub use learning::{
    EvaluationReport, FeedbackDataset, FeedbackRecord, LearningError, Observation,
    ObservationStatus, Procedure, ProcedureStatus,
};
pub use memory_error::CognitionMemoryError;
pub use metrics::{MetricsSnapshot, OperationKind, OperationMetrics, OperationSample};

#[cfg(test)]
mod binding_error_tests;
#[cfg(test)]
mod engine_binding_tests;
#[cfg(test)]
mod formation_profile_tests;
#[cfg(test)]
mod memory_error_tests;
