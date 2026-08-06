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
    /// Conversation turns normalized into duplicate-safe memory candidates.
    ConversationDeduplicationV1,
    /// Document records normalized into duplicate-safe memory candidates.
    DocumentDeduplicationV1,
    /// JSON events normalized into contradiction-safe memory candidates.
    JsonEventReconciliationV1,
    /// Raw records admitted only to duplicate-safe consolidation.
    RawDeduplicationV1,
}

/// Trusted executor family selected by deployment composition, never by a job payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormationProvider {
    ReferenceV1,
    SailV1,
}

/// Fully resolved, bounded formation contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormationBinding {
    pub profile: FormationProfile,
    pub provider: FormationProvider,
    pub operation: CognitionOperation,
    pub input_schema_version: &'static str,
    pub output_schema_version: &'static str,
    pub max_source_records: u32,
    pub max_output_records: u32,
}

impl FormationProfile {
    /// Resolve one closed profile against one trusted provider family.
    #[must_use]
    pub const fn bind(self, provider: FormationProvider) -> FormationBinding {
        FormationBinding {
            profile: self,
            provider,
            operation: self.operation(),
            input_schema_version: "1",
            output_schema_version: "1",
            max_source_records: 10_000,
            max_output_records: 10_000,
        }
    }
    /// Canonical profile identity for signed job bindings.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BackgroundDeduplicationV1 => "background-deduplication-v1",
            Self::BackgroundReconciliationV1 => "background-reconciliation-v1",
            Self::ConversationDeduplicationV1 => "conversation-deduplication-v1",
            Self::DocumentDeduplicationV1 => "document-deduplication-v1",
            Self::JsonEventReconciliationV1 => "json-event-reconciliation-v1",
            Self::RawDeduplicationV1 => "raw-deduplication-v1",
        }
    }

    /// The only native cognition operation this profile may invoke.
    #[must_use]
    pub const fn operation(self) -> CognitionOperation {
        match self {
            Self::BackgroundDeduplicationV1
            | Self::ConversationDeduplicationV1
            | Self::DocumentDeduplicationV1
            | Self::RawDeduplicationV1 => CognitionOperation::Deduplicate,
            Self::BackgroundReconciliationV1 | Self::JsonEventReconciliationV1 => {
                CognitionOperation::Reconcile
            }
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
            "conversation-deduplication-v1" => Ok(Self::ConversationDeduplicationV1),
            "document-deduplication-v1" => Ok(Self::DocumentDeduplicationV1),
            "json-event-reconciliation-v1" => Ok(Self::JsonEventReconciliationV1),
            "raw-deduplication-v1" => Ok(Self::RawDeduplicationV1),
            _ => Err(ParseFormationProfileError),
        }
    }
}

/// A profile identity was absent, unsupported, or non-canonical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unsupported formation profile")]
pub struct ParseFormationProfileError;
