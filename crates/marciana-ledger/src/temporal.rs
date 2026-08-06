use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, de};

use crate::LedgerError;

/// The period in which an assertion is believed to hold. Intervals are
/// half-open (`[valid_from, valid_to)`); a missing end is open-ended rather
/// than a sentinel timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemporalInterval {
    valid_from: DateTime<Utc>,
    valid_to: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TemporalIntervalWire {
    valid_from: DateTime<Utc>,
    valid_to: Option<DateTime<Utc>>,
}

impl<'de> Deserialize<'de> for TemporalInterval {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = TemporalIntervalWire::deserialize(deserializer)?;
        Self::new(wire.valid_from, wire.valid_to).map_err(de::Error::custom)
    }
}

impl TemporalInterval {
    /// Builds a closed or open-ended validity interval.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerError::InvalidTemporalInterval`] when the end precedes
    /// the start.
    pub fn new(
        valid_from: DateTime<Utc>,
        valid_to: Option<DateTime<Utc>>,
    ) -> Result<Self, LedgerError> {
        if valid_to.is_some_and(|end| end < valid_from) {
            return Err(LedgerError::InvalidTemporalInterval);
        }
        Ok(Self {
            valid_from,
            valid_to,
        })
    }

    /// Start of the interval, inclusive.
    #[must_use]
    pub fn valid_from(&self) -> DateTime<Utc> {
        self.valid_from
    }

    /// End of the interval, exclusive when present.
    #[must_use]
    pub fn valid_to(&self) -> Option<DateTime<Utc>> {
        self.valid_to
    }

    /// Whether a point in time is in the interval.
    #[must_use]
    pub fn contains(&self, at: DateTime<Utc>) -> bool {
        at >= self.valid_from && self.valid_to.is_none_or(|end| at < end)
    }
}
