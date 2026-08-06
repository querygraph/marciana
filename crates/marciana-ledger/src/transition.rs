use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{AssertionId, LedgerError};

/// A belief state is separate from its structural subject-predicate-object
/// projection. History is retained through state transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssertionState {
    Proposed,
    Current,
    Disputed,
    Negated,
    Superseded,
    Retracted,
    Forgotten,
}

impl AssertionState {
    pub(crate) fn may_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (
                Self::Proposed,
                Self::Current | Self::Disputed | Self::Negated | Self::Retracted | Self::Forgotten
            ) | (
                Self::Current,
                Self::Disputed
                    | Self::Negated
                    | Self::Superseded
                    | Self::Retracted
                    | Self::Forgotten
            ) | (
                Self::Disputed,
                Self::Current
                    | Self::Negated
                    | Self::Superseded
                    | Self::Retracted
                    | Self::Forgotten
            ) | (
                Self::Negated,
                Self::Current | Self::Superseded | Self::Retracted | Self::Forgotten
            ) | (Self::Superseded, Self::Retracted | Self::Forgotten)
                | (Self::Retracted, Self::Forgotten)
        )
    }
}

/// Identifiers and opaque evidence digests that justify a lifecycle change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransitionEvidence {
    causing_assertions: Vec<AssertionId>,
    evidence_digests: Vec<String>,
}

impl TransitionEvidence {
    /// Creates canonical, duplicate-free transition evidence.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerError::InvalidTransitionEvidence`] unless both causal
    /// assertions and lowercase SHA-256 evidence digests are present and
    /// duplicate-free.
    pub fn new(
        mut causing_assertions: Vec<AssertionId>,
        mut evidence_digests: Vec<String>,
    ) -> Result<Self, LedgerError> {
        causing_assertions.sort();
        evidence_digests.sort();
        if causing_assertions.is_empty()
            || evidence_digests.is_empty()
            || causing_assertions.windows(2).any(|pair| pair[0] == pair[1])
            || evidence_digests.windows(2).any(|pair| pair[0] == pair[1])
            || evidence_digests.iter().any(|digest| !is_digest(digest))
        {
            return Err(LedgerError::InvalidTransitionEvidence);
        }
        Ok(Self {
            causing_assertions,
            evidence_digests,
        })
    }

    /// Canonically ordered causal assertion identifiers.
    #[must_use]
    pub fn causing_assertions(&self) -> &[AssertionId] {
        &self.causing_assertions
    }

    /// Canonically ordered opaque evidence digests.
    #[must_use]
    pub fn evidence_digests(&self) -> &[String] {
        &self.evidence_digests
    }
}

/// A recorded lifecycle state change. The guarded durable ledger applies it
/// only if `from` matches the current persisted state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssertionTransition {
    from: AssertionState,
    to: AssertionState,
    at: DateTime<Utc>,
    evidence: TransitionEvidence,
}

impl AssertionTransition {
    /// Validates the allowed lifecycle edge and its causal evidence.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerError::InvalidTransition`] when the lifecycle edge is
    /// not an approved transition.
    pub fn new(
        from: AssertionState,
        to: AssertionState,
        at: DateTime<Utc>,
        evidence: TransitionEvidence,
    ) -> Result<Self, LedgerError> {
        if !from.may_transition_to(to) {
            return Err(LedgerError::InvalidTransition);
        }
        Ok(Self {
            from,
            to,
            at,
            evidence,
        })
    }

    #[must_use]
    pub fn from(&self) -> AssertionState {
        self.from
    }
    #[must_use]
    pub fn to(&self) -> AssertionState {
        self.to
    }
    #[must_use]
    pub fn at(&self) -> DateTime<Utc> {
        self.at
    }
    #[must_use]
    pub fn evidence(&self) -> &TransitionEvidence {
        &self.evidence
    }
}

fn is_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}
