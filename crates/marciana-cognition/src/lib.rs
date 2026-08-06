//! Marciana-owned cognition composition primitives.

mod binding_error;
mod engine_binding;
mod governed;
mod memory_error;

pub use binding_error::CognitionBindingError;
pub use engine_binding::CognitionEngineBinding;
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use governed::intent_claim_limits_for_test;
pub use governed::{
    CognitionApplicationError, FreshLakeCatAuthority, GovernedCognitionApplication,
    GovernedCognitionConfig, GovernedCognitionResult, LakeCatAuthorityError,
    LakeCatCognitionAuthority,
};
pub use memory_error::CognitionMemoryError;

#[cfg(test)]
mod binding_error_tests;
#[cfg(test)]
mod engine_binding_tests;
#[cfg(test)]
mod memory_error_tests;
