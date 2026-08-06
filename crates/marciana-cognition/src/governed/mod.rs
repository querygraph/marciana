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

pub use application::{
    CognitionApplicationError, GovernedCognitionApplication, GovernedCognitionConfig,
    GovernedCognitionResult,
};
pub use authority::{FreshLakeCatAuthority, LakeCatAuthorityError, LakeCatCognitionAuthority};

#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use clock::CognitionClock;

#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use intent::intent_claim_limits_for_test;
