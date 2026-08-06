use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, de};
use uuid::Uuid;

use crate::{AssertionState, AssertionTransition, LedgerError, TemporalInterval};

const MAX_TERM_BYTES: usize = 512;
const MAX_LINEAGE_BYTES: usize = 512;

/// A collision-resistant, opaque assertion identity. It is deliberately not a
/// hash of a structural triplet, so identical claims may retain distinct
/// provenance and temporal history.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct AssertionId(String);

impl AssertionId {
    /// Generates a random UUID v4 identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Parses a canonical UUID assertion identity.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerError::InvalidAssertionId`] when `value` is not a
    /// canonical lowercase UUID.
    pub fn parse(value: impl AsRef<str>) -> Result<Self, LedgerError> {
        let parsed =
            Uuid::parse_str(value.as_ref()).map_err(|_| LedgerError::InvalidAssertionId)?;
        if parsed.to_string() != value.as_ref() {
            return Err(LedgerError::InvalidAssertionId);
        }
        Ok(Self(parsed.to_string()))
    }

    pub(crate) fn from_legacy_key(key: &str) -> Self {
        let name = format!("querygraph.marciana.legacy-relation.v1\0{key}");
        Self(Uuid::new_v5(&Uuid::NAMESPACE_OID, name.as_bytes()).to_string())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for AssertionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AssertionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<'de> Deserialize<'de> for AssertionId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::parse(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// A bounded confidence represented exactly in basis points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct Confidence(u16);

impl Confidence {
    pub const MAX: u16 = 10_000;

    /// # Errors
    ///
    /// Returns [`LedgerError::InvalidConfidence`] above 10,000 basis points.
    pub fn from_basis_points(value: u16) -> Result<Self, LedgerError> {
        (value <= Self::MAX)
            .then_some(Self(value))
            .ok_or(LedgerError::InvalidConfidence)
    }

    #[must_use]
    pub fn basis_points(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for Confidence {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_basis_points(u16::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// Exact source-record and formation provenance for an assertion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssertionLineage {
    source_episode_id: String,
    source_record_id: String,
    formation_profile_version: String,
    schema_version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AssertionLineageWire {
    source_episode_id: String,
    source_record_id: String,
    formation_profile_version: String,
    schema_version: String,
}

impl<'de> Deserialize<'de> for AssertionLineage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = AssertionLineageWire::deserialize(deserializer)?;
        Self::new(
            wire.source_episode_id,
            wire.source_record_id,
            wire.formation_profile_version,
            wire.schema_version,
        )
        .map_err(de::Error::custom)
    }
}

impl AssertionLineage {
    /// # Errors
    ///
    /// Returns [`LedgerError::InvalidLineage`] when a lineage field is empty,
    /// non-canonical, or exceeds its bounded representation.
    pub fn new(
        source_episode_id: impl Into<String>,
        source_record_id: impl Into<String>,
        formation_profile_version: impl Into<String>,
        schema_version: impl Into<String>,
    ) -> Result<Self, LedgerError> {
        let lineage = Self {
            source_episode_id: source_episode_id.into(),
            source_record_id: source_record_id.into(),
            formation_profile_version: formation_profile_version.into(),
            schema_version: schema_version.into(),
        };
        [
            &lineage.source_episode_id,
            &lineage.source_record_id,
            &lineage.formation_profile_version,
            &lineage.schema_version,
        ]
        .iter()
        .all(|value| is_canonical(value, MAX_LINEAGE_BYTES))
        .then_some(lineage)
        .ok_or(LedgerError::InvalidLineage)
    }

    #[must_use]
    pub fn source_episode_id(&self) -> &str {
        &self.source_episode_id
    }
    #[must_use]
    pub fn source_record_id(&self) -> &str {
        &self.source_record_id
    }
    #[must_use]
    pub fn formation_profile_version(&self) -> &str {
        &self.formation_profile_version
    }
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

/// An atomic, provenance-bearing belief that has a stable identity independent
/// of its graph projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Assertion {
    id: AssertionId,
    subject: String,
    predicate: String,
    object: String,
    confidence: Confidence,
    observed_at: DateTime<Utc>,
    ingested_at: DateTime<Utc>,
    validity: TemporalInterval,
    lineage: AssertionLineage,
    state: AssertionState,
    transitions: Vec<AssertionTransition>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AssertionWire {
    id: AssertionId,
    subject: String,
    predicate: String,
    object: String,
    confidence: Confidence,
    observed_at: DateTime<Utc>,
    ingested_at: DateTime<Utc>,
    validity: TemporalInterval,
    lineage: AssertionLineage,
    state: AssertionState,
    transitions: Vec<AssertionTransition>,
}

impl<'de> Deserialize<'de> for Assertion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = AssertionWire::deserialize(deserializer)?;
        let mut assertion = Self::new(
            wire.id,
            wire.subject,
            wire.predicate,
            wire.object,
            wire.confidence,
            wire.observed_at,
            wire.ingested_at,
            wire.validity,
            wire.lineage,
        )
        .map_err(de::Error::custom)?;
        for transition in wire.transitions {
            assertion
                .apply_transition(transition)
                .map_err(de::Error::custom)?;
        }
        if assertion.state != wire.state {
            return Err(de::Error::custom(LedgerError::InvalidTransition));
        }
        Ok(assertion)
    }
}

impl Assertion {
    #[allow(clippy::too_many_arguments)]
    /// # Errors
    ///
    /// Returns [`LedgerError::InvalidTerm`] when a term is non-canonical or
    /// the observation is later than ingestion.
    pub fn new(
        id: AssertionId,
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: impl Into<String>,
        confidence: Confidence,
        observed_at: DateTime<Utc>,
        ingested_at: DateTime<Utc>,
        validity: TemporalInterval,
        lineage: AssertionLineage,
    ) -> Result<Self, LedgerError> {
        let assertion = Self {
            id,
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
            confidence,
            observed_at,
            ingested_at,
            validity,
            lineage,
            state: AssertionState::Proposed,
            transitions: Vec::new(),
        };
        if [
            assertion.subject(),
            assertion.predicate(),
            assertion.object(),
        ]
        .iter()
        .all(|term| is_canonical(term, MAX_TERM_BYTES))
            && assertion.observed_at <= assertion.ingested_at
        {
            Ok(assertion)
        } else {
            Err(LedgerError::InvalidTerm)
        }
    }

    /// Applies a validated transition only when it starts at the current state
    /// and does not move time backwards.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerError::InvalidTransition`] if its source state differs
    /// from the current state, precedes ingestion, or regresses history.
    pub fn apply_transition(&mut self, transition: AssertionTransition) -> Result<(), LedgerError> {
        if transition.from() != self.state
            || transition.at() < self.ingested_at
            || self
                .transitions
                .last()
                .is_some_and(|prior| transition.at() < prior.at())
        {
            return Err(LedgerError::InvalidTransition);
        }
        self.state = transition.to();
        self.transitions.push(transition);
        Ok(())
    }

    #[must_use]
    pub fn id(&self) -> &AssertionId {
        &self.id
    }
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }
    #[must_use]
    pub fn predicate(&self) -> &str {
        &self.predicate
    }
    #[must_use]
    pub fn object(&self) -> &str {
        &self.object
    }
    #[must_use]
    pub fn confidence(&self) -> Confidence {
        self.confidence
    }
    #[must_use]
    pub fn observed_at(&self) -> DateTime<Utc> {
        self.observed_at
    }
    #[must_use]
    pub fn ingested_at(&self) -> DateTime<Utc> {
        self.ingested_at
    }
    #[must_use]
    pub fn validity(&self) -> &TemporalInterval {
        &self.validity
    }
    #[must_use]
    pub fn lineage(&self) -> &AssertionLineage {
        &self.lineage
    }
    #[must_use]
    pub fn state(&self) -> AssertionState {
        self.state
    }
    /// Belief state at an instant, derived from the immutable transition log.
    #[must_use]
    pub fn state_at(&self, at: DateTime<Utc>) -> AssertionState {
        self.transitions
            .iter()
            .take_while(|transition| transition.at() <= at)
            .last()
            .map_or(AssertionState::Proposed, AssertionTransition::to)
    }

    /// Whether this assertion was both valid and current at an instant.
    #[must_use]
    pub fn is_current_at(&self, at: DateTime<Utc>) -> bool {
        self.validity.contains(at) && self.state_at(at) == AssertionState::Current
    }

    #[must_use]
    pub fn transitions(&self) -> &[AssertionTransition] {
        &self.transitions
    }
}

fn is_canonical(value: &str, limit: usize) -> bool {
    !value.is_empty()
        && value.len() <= limit
        && value.trim() == value
        && !value.chars().any(char::is_control)
}
