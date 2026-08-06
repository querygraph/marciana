use thiserror::Error;

/// Fixed validation failures for ledger values.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LedgerError {
    #[error("assertion identifier is invalid")]
    InvalidAssertionId,
    #[error("assertion term is invalid")]
    InvalidTerm,
    #[error("assertion confidence is invalid")]
    InvalidConfidence,
    #[error("temporal interval is invalid")]
    InvalidTemporalInterval,
    #[error("assertion lineage is invalid")]
    InvalidLineage,
    #[error("assertion transition is invalid")]
    InvalidTransition,
    #[error("assertion transition evidence is invalid")]
    InvalidTransitionEvidence,
}
