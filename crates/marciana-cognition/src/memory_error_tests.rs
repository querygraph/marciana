use typesec_core::CapabilityUseError;
use typesec_memory::{
    CognitionApplyError, CognitionCommitError, CognitionRecoveryError,
    GovernedSourceVerificationError, MemoryError, MemoryId, StoreError,
};

use super::CognitionMemoryError;

#[test]
fn memory_errors_map_to_fixed_cognition_categories() {
    let cases = [
        (
            MemoryError::SpaceMismatch {
                capability_space: "memory/a".into(),
                target_space: "memory/b".into(),
            },
            CognitionMemoryError::AccessDenied,
        ),
        (
            MemoryError::Capability(CapabilityUseError::Revoked {
                minted_epoch: 1,
                current_epoch: 2,
            }),
            CognitionMemoryError::AccessDenied,
        ),
        (
            MemoryError::PolicyDenied {
                action: "memory:read",
                detail: "protected rationale".into(),
            },
            CognitionMemoryError::AccessDenied,
        ),
        (
            MemoryError::AboveCeiling {
                id: "protected-id".into(),
                label: "secret",
                ceiling: "public",
            },
            CognitionMemoryError::AccessDenied,
        ),
        (
            MemoryError::NotFound("protected-id".into()),
            CognitionMemoryError::SourceUnavailable,
        ),
        (
            MemoryError::GovernedSourceScopeMismatch,
            CognitionMemoryError::SourceUnavailable,
        ),
        (
            MemoryError::Cognition(CognitionApplyError::SourceScopeMismatch),
            CognitionMemoryError::SourceUnavailable,
        ),
        (
            MemoryError::GovernedSourceVerification(GovernedSourceVerificationError::Unavailable),
            CognitionMemoryError::BackendUnavailable,
        ),
        (
            MemoryError::Store(StoreError::Backend("protected backend detail".into())),
            CognitionMemoryError::BackendUnavailable,
        ),
        (
            MemoryError::Cognition(CognitionApplyError::Authority),
            CognitionMemoryError::ProposalRejected,
        ),
        (
            MemoryError::CognitionCommit(CognitionCommitError::StaleSource(MemoryId::from_string(
                "mem-stale",
            ))),
            CognitionMemoryError::StaleSource,
        ),
        (
            MemoryError::CognitionCommit(CognitionCommitError::IdempotencyConflict),
            CognitionMemoryError::IdempotencyConflict,
        ),
        (
            MemoryError::CognitionCommit(CognitionCommitError::InvalidOutcome),
            CognitionMemoryError::InvalidCommitOutcome,
        ),
        (
            MemoryError::CognitionCommit(CognitionCommitError::Store(StoreError::Unsupported)),
            CognitionMemoryError::BackendUnavailable,
        ),
        (
            MemoryError::CognitionRecovery(CognitionRecoveryError::Unavailable),
            CognitionMemoryError::RecoveryUnavailable,
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(CognitionMemoryError::from(error), expected);
    }
}
