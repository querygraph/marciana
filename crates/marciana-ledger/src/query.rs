use chrono::{DateTime, Utc};

use crate::{Assertion, AssertionState, LedgerError};

/// A deterministic, content-free assertion candidate query. The query never
/// performs authorization or materializes protected assertion content.
pub struct AssertionQuery {
    as_of: DateTime<Utc>,
    states: u8,
    require_validity: bool,
}

impl AssertionQuery {
    /// Selects current assertions whose validity contains `as_of`.
    #[must_use]
    pub fn current_at(as_of: DateTime<Utc>) -> Self {
        Self {
            as_of,
            states: state_bit(AssertionState::Current),
            require_validity: true,
        }
    }

    /// Selects assertions in any requested lifecycle state at `as_of`.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerError::InvalidQuery`] when no lifecycle state is
    /// requested. Historical-state queries intentionally do not require the
    /// assertion's validity interval to contain `as_of`.
    pub fn states_at(
        as_of: DateTime<Utc>,
        states: impl IntoIterator<Item = AssertionState>,
    ) -> Result<Self, LedgerError> {
        let states = states
            .into_iter()
            .fold(0, |selected, state| selected | state_bit(state));
        (states != 0)
            .then_some(Self {
                as_of,
                states,
                require_validity: false,
            })
            .ok_or(LedgerError::InvalidQuery)
    }

    /// Returns deterministic candidate references, newest validity first and
    /// then by stable assertion identity.
    #[must_use]
    pub fn select<'a>(&self, assertions: &'a [Assertion]) -> Vec<&'a Assertion> {
        let mut selected = assertions
            .iter()
            .filter(|assertion| {
                self.states & state_bit(assertion.state_at(self.as_of)) != 0
                    && (!self.require_validity || assertion.validity().contains(self.as_of))
            })
            .collect::<Vec<_>>();
        selected.sort_by(|left, right| {
            right
                .validity()
                .valid_from()
                .cmp(&left.validity().valid_from())
                .then_with(|| left.id().cmp(right.id()))
        });
        selected
    }
}

const fn state_bit(state: AssertionState) -> u8 {
    match state {
        AssertionState::Proposed => 1 << 0,
        AssertionState::Current => 1 << 1,
        AssertionState::Disputed => 1 << 2,
        AssertionState::Negated => 1 << 3,
        AssertionState::Superseded => 1 << 4,
        AssertionState::Retracted => 1 << 5,
        AssertionState::Forgotten => 1 << 6,
    }
}
