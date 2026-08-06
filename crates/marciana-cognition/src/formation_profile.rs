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

/// Closed scheduling modes for formation jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormationRunMode {
    /// Durable background work with normal lease/retry semantics.
    Background,
    /// Opt-in proposal generation on an agent hot path; it never authorizes
    /// direct memory mutation.
    HotPathProposal,
}

/// Native operation capability exposed by a trusted formation provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormationCapability {
    /// Exact duplicate consolidation.
    Deduplicate,
    /// Contradiction reconciliation.
    Reconcile,
}

/// Bounded resources a provider may consume for one formation job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormationResourceBudget {
    /// Maximum authorized source records.
    pub max_source_records: u32,
    /// Maximum proposed output records.
    pub max_output_records: u32,
}

impl FormationResourceBudget {
    const DEFAULT: Self = Self {
        max_source_records: 10_000,
        max_output_records: 10_000,
    };

    /// Check an authorized source selection before materialization.
    ///
    /// # Errors
    ///
    /// Returns [`FormationBudgetError::SourceRecords`] when `count` exceeds
    /// the provider ceiling.
    pub fn check_source_records(self, count: usize) -> Result<(), FormationBudgetError> {
        if count > self.max_source_records as usize {
            return Err(FormationBudgetError::SourceRecords {
                limit: self.max_source_records,
                actual: count,
            });
        }
        Ok(())
    }

    /// Check an inert proposal before authoritative application.
    ///
    /// # Errors
    ///
    /// Returns [`FormationBudgetError::OutputRecords`] when `count` exceeds
    /// the provider ceiling.
    pub fn check_output_records(self, count: usize) -> Result<(), FormationBudgetError> {
        if count > self.max_output_records as usize {
            return Err(FormationBudgetError::OutputRecords {
                limit: self.max_output_records,
                actual: count,
            });
        }
        Ok(())
    }
}

/// Error returned when a formation resource ceiling is exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FormationBudgetError {
    #[error("formation source-record budget exceeded: limit {limit}, actual {actual}")]
    SourceRecords { limit: u32, actual: usize },
    #[error("formation output-record budget exceeded: limit {limit}, actual {actual}")]
    OutputRecords { limit: u32, actual: usize },
}

/// Trusted provider/profile resolver. It contains no model- or payload-defined
/// registration path; deployments compose only the closed native providers.
#[derive(Debug, Clone, Copy, Default)]
pub struct FormationRegistry;

impl FormationRegistry {
    /// Resolve a closed profile to a provider capability and fixed schemas.
    ///
    /// # Errors
    ///
    /// Returns [`FormationRegistryError::UnsupportedCapability`] if the
    /// trusted provider does not expose the profile's native operation.
    pub fn resolve(
        self,
        profile: FormationProfile,
        provider: FormationProvider,
    ) -> Result<FormationBinding, FormationRegistryError> {
        self.resolve_for_mode(profile, provider, FormationRunMode::Background)
    }

    /// Resolve a profile for an explicit closed scheduling mode.
    ///
    /// # Errors
    /// Returns [`FormationRegistryError::ModeNotAllowed`] when a background
    /// profile is asked to run on the hot path.
    pub fn resolve_for_mode(
        self,
        profile: FormationProfile,
        provider: FormationProvider,
        mode: FormationRunMode,
    ) -> Result<FormationBinding, FormationRegistryError> {
        let capability = profile.capability();
        if !provider.supports(capability) {
            return Err(FormationRegistryError::UnsupportedCapability {
                provider,
                capability,
            });
        }
        if !profile.supports_mode(mode) {
            return Err(FormationRegistryError::ModeNotAllowed { profile, mode });
        }
        Ok(profile.bind_for_mode(provider, mode))
    }
}

/// Registry resolution failed closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FormationRegistryError {
    #[error("formation provider does not support the selected capability")]
    UnsupportedCapability {
        provider: FormationProvider,
        capability: FormationCapability,
    },
    #[error("formation profile is not allowed in the selected run mode")]
    ModeNotAllowed {
        profile: FormationProfile,
        mode: FormationRunMode,
    },
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
    /// Native operation capability selected by the profile.
    pub capability: FormationCapability,
    /// Explicit provider resource ceiling.
    pub budget: FormationResourceBudget,
    /// Closed scheduler mode bound into this provider/profile contract.
    pub run_mode: FormationRunMode,
}

impl FormationProfile {
    /// Resolve one closed profile against one trusted provider family.
    #[must_use]
    pub const fn bind(self, provider: FormationProvider) -> FormationBinding {
        self.bind_for_mode(provider, FormationRunMode::Background)
    }

    /// Bind one profile to a provider and already-validated run mode.
    #[must_use]
    pub const fn bind_for_mode(
        self,
        provider: FormationProvider,
        run_mode: FormationRunMode,
    ) -> FormationBinding {
        FormationBinding {
            profile: self,
            provider,
            operation: self.operation(),
            input_schema_version: "1",
            output_schema_version: "1",
            max_source_records: FormationResourceBudget::DEFAULT.max_source_records,
            max_output_records: FormationResourceBudget::DEFAULT.max_output_records,
            capability: self.capability(),
            budget: FormationResourceBudget::DEFAULT,
            run_mode,
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

    /// Native capability selected by this profile.
    #[must_use]
    pub const fn capability(self) -> FormationCapability {
        match self.operation() {
            CognitionOperation::Deduplicate => FormationCapability::Deduplicate,
            CognitionOperation::Reconcile => FormationCapability::Reconcile,
        }
    }

    /// Stable schema version of the declarative profile contract.
    #[must_use]
    pub const fn schema_version(self) -> &'static str {
        "1"
    }

    /// Whether this profile may run in the selected closed scheduler mode.
    #[must_use]
    pub const fn supports_mode(self, mode: FormationRunMode) -> bool {
        match mode {
            FormationRunMode::Background => true,
            FormationRunMode::HotPathProposal => !matches!(
                self,
                Self::BackgroundDeduplicationV1 | Self::BackgroundReconciliationV1
            ),
        }
    }
}

impl FormationProvider {
    /// Stable provider identity used by deployment composition.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReferenceV1 => "reference-v1",
            Self::SailV1 => "sail-v1",
        }
    }

    /// Whether this trusted provider supports a native capability.
    #[must_use]
    pub const fn supports(self, _capability: FormationCapability) -> bool {
        match self {
            Self::ReferenceV1 | Self::SailV1 => true,
        }
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
