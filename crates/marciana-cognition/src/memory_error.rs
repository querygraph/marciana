//! Fixed public categories for protected-memory failures.

use typesec_memory::{CognitionApplyError, CognitionCommitError, MemoryError};

/// Sanitized failure from the `TypeSec` memory boundary.
///
/// `TypeSec` and backend implementations retain detailed causes for protected
/// diagnostics. Marciana never forwards policy rationale, record IDs, or
/// backend-controlled strings through its public cognition error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CognitionMemoryError {
    /// The capability, policy, space, or clearance did not authorize access.
    #[error("cognition memory access was denied")]
    AccessDenied,
    /// A selected source was unavailable under the authorized view.
    #[error("cognition source is unavailable")]
    SourceUnavailable,
    /// `TypeSec` rejected the proposal or its current authority binding.
    #[error("cognition proposal was rejected")]
    ProposalRejected,
    /// A source revision changed before the atomic commit.
    #[error("cognition source changed before commit")]
    StaleSource,
    /// The scoped job identity already belongs to another proposal.
    #[error("cognition job identity conflicts with an existing proposal")]
    IdempotencyConflict,
    /// The backend returned evidence inconsistent with the prepared commit.
    #[error("cognition backend returned invalid commit evidence")]
    InvalidCommitOutcome,
    /// Protected storage or durable commit state could not be used safely.
    #[error("cognition memory backend is unavailable")]
    BackendUnavailable,
    /// Historical commit evidence could not be disclosed safely.
    #[error("completed cognition outcome is unavailable")]
    RecoveryUnavailable,
}

impl From<MemoryError> for CognitionMemoryError {
    fn from(error: MemoryError) -> Self {
        match error {
            MemoryError::SpaceMismatch { .. }
            | MemoryError::Capability(_)
            | MemoryError::PolicyDenied { .. }
            | MemoryError::AboveCeiling { .. } => Self::AccessDenied,
            MemoryError::NotFound(_)
            | MemoryError::GovernedSourceScopeMismatch
            | MemoryError::Cognition(CognitionApplyError::SourceScopeMismatch) => {
                Self::SourceUnavailable
            }
            MemoryError::GovernedSourceVerification(_) | MemoryError::Store(_) => {
                Self::BackendUnavailable
            }
            MemoryError::Cognition(_) => Self::ProposalRejected,
            MemoryError::CognitionCommit(error) => match error {
                CognitionCommitError::StaleSource(_) => Self::StaleSource,
                CognitionCommitError::IdempotencyConflict => Self::IdempotencyConflict,
                CognitionCommitError::InvalidOutcome => Self::InvalidCommitOutcome,
                CognitionCommitError::Store(_) => Self::BackendUnavailable,
            },
            MemoryError::CognitionRecovery(_) => Self::RecoveryUnavailable,
        }
    }
}
