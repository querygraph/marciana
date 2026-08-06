use chrono::{DateTime, Utc};

use crate::{
    Assertion, AssertionId, AssertionLineage, AssertionState, AssertionTransition, Confidence,
    LedgerError, TemporalInterval, TransitionEvidence,
};

const MAX_LEGACY_KEY_BYTES: usize = 512;

/// A structural relation from the baseline graph, plus the record-derived
/// facts needed to turn it into one explicit assertion. Its key must identify
/// one legacy edge deterministically across retries.
pub struct LegacyRelation {
    key: String,
    subject: String,
    predicate: String,
    object: String,
    confidence: Confidence,
    observed_at: DateTime<Utc>,
    ingested_at: DateTime<Utc>,
    validity: TemporalInterval,
    lineage: AssertionLineage,
    import_evidence: TransitionEvidence,
}

impl LegacyRelation {
    #[allow(clippy::too_many_arguments)]
    /// Creates a retry-stable migration input.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerError::InvalidAssertionId`] when `key` cannot safely
    /// serve as a bounded deterministic migration key. Other invalid fields
    /// are rejected when the resulting assertion is constructed.
    pub fn new(
        key: impl Into<String>,
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: impl Into<String>,
        confidence: Confidence,
        observed_at: DateTime<Utc>,
        ingested_at: DateTime<Utc>,
        validity: TemporalInterval,
        lineage: AssertionLineage,
        import_evidence: TransitionEvidence,
    ) -> Result<Self, LedgerError> {
        let key = key.into();
        if !is_key(&key) {
            return Err(LedgerError::InvalidAssertionId);
        }
        if !import_evidence.causing_assertions().is_empty() {
            return Err(LedgerError::InvalidTransitionEvidence);
        }
        Ok(Self {
            key,
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
            confidence,
            observed_at,
            ingested_at,
            validity,
            lineage,
            import_evidence,
        })
    }

    /// Converts the relation to a deterministic assertion and records its
    /// verified import evidence as the initial proposed-to-current transition.
    ///
    /// # Errors
    ///
    /// Propagates assertion or transition validation failures.
    pub fn migrate(self) -> Result<Assertion, LedgerError> {
        let id = AssertionId::from_legacy_key(&self.key);
        let mut assertion = Assertion::new(
            id,
            self.subject,
            self.predicate,
            self.object,
            self.confidence,
            self.observed_at,
            self.ingested_at,
            self.validity,
            self.lineage,
        )?;
        let transition = AssertionTransition::new(
            AssertionState::Proposed,
            AssertionState::Current,
            assertion.ingested_at(),
            self.import_evidence,
        )?;
        assertion.apply_transition(transition)?;
        Ok(assertion)
    }
}

fn is_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_LEGACY_KEY_BYTES
        && value.trim() == value
        && !value.chars().any(char::is_control)
}
