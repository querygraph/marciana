//! Closed, versioned formation profiles for durable cognition jobs.

use std::str::FromStr;

use querygraph_memory::cognition::CognitionOperation;

/// A declarative profile that selects one conservative formation behavior.
/// Profiles are stable product identities, not model-provided configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormationProfile {
    /// Exact duplicate consolidation over a governed source selection.
    BackgroundDeduplicationV1,
    /// Contradiction reconciliation over a governed source selection.
    BackgroundReconciliationV1,
}

impl FormationProfile {
    /// Canonical profile identity for signed job bindings.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BackgroundDeduplicationV1 => "background-deduplication-v1",
            Self::BackgroundReconciliationV1 => "background-reconciliation-v1",
        }
    }

    /// The only native cognition operation this profile may invoke.
    #[must_use]
    pub const fn operation(self) -> CognitionOperation {
        match self {
            Self::BackgroundDeduplicationV1 => CognitionOperation::Deduplicate,
            Self::BackgroundReconciliationV1 => CognitionOperation::Reconcile,
        }
    }

    /// Stable schema version of the declarative profile contract.
    #[must_use]
    pub const fn schema_version(self) -> &'static str {
        "1"
    }
}

impl std::fmt::Display for FormationProfile {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for FormationProfile {
    type Err = ParseFormationProfileError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "background-deduplication-v1" => Ok(Self::BackgroundDeduplicationV1),
            "background-reconciliation-v1" => Ok(Self::BackgroundReconciliationV1),
            _ => Err(ParseFormationProfileError),
        }
    }
}

/// A profile identity was absent, unsupported, or non-canonical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unsupported formation profile")]
pub struct ParseFormationProfileError;
