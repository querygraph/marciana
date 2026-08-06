//! Small injectable clock seam for request and authority freshness checks.

use chrono::{DateTime, Utc};

#[doc(hidden)]
pub trait CognitionClock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Default)]
pub(crate) struct SystemCognitionClock;

impl CognitionClock for SystemCognitionClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
