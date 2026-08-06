use std::collections::BTreeSet;

use chrono::{DateTime, Utc};

use crate::{Assertion, AssertionState, LedgerError};

/// A deterministic, content-free assertion candidate query. The query never
/// performs authorization or materializes protected assertion content.
pub struct AssertionQuery {
    as_of: DateTime<Utc>,
    states: BTreeSet<AssertionState>,
    require_validity: bool,
}

impl AssertionQuery {
    /// Selects current assertions whose validity contains `as_of`.
    #[must_use]
    pub fn current_at(as_of: DateTime<Utc>) -> Self {
        Self {
            as_of,
            states: BTreeSet::from([AssertionState::Current]),
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
        let states = states.into_iter().collect::<BTreeSet<_>>();
        (!states.is_empty())
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
                self.states.contains(&assertion.state_at(self.as_of))
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
