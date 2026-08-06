//! Bounded, digest-only worker progress for durable cognition jobs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::CognitionStateError;
use super::graph::is_sha256;

/// Maximum progress units a worker may advertise for one job.
pub const MAX_COGNITION_PROGRESS_UNITS: u64 = 1_000_000;

/// Coarse phases are intentionally closed so operators can depend on stable semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitionProgressPhase {
    Queued,
    Authorizing,
    Scanning,
    Planning,
    Revalidating,
    Staging,
    Committing,
    Finalizing,
}

/// Safe operational progress. Detail is a digest, never worker text or model output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CognitionProgress {
    pub phase: CognitionProgressPhase,
    pub completed_units: u64,
    pub total_units: Option<u64>,
    pub detail_digest: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl CognitionProgress {
    pub(crate) fn queued(now: DateTime<Utc>) -> Self {
        Self {
            phase: CognitionProgressPhase::Queued,
            completed_units: 0,
            total_units: None,
            detail_digest: None,
            updated_at: now,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), CognitionStateError> {
        if self.completed_units > MAX_COGNITION_PROGRESS_UNITS {
            return Err(CognitionStateError::Invalid(
                "progress exceeds its fixed bound".into(),
            ));
        }
        if let Some(total) = self.total_units {
            if total == 0 || total > MAX_COGNITION_PROGRESS_UNITS || self.completed_units > total {
                return Err(CognitionStateError::Invalid(
                    "progress units violate bounds".into(),
                ));
            }
        }
        if self
            .detail_digest
            .as_deref()
            .is_some_and(|digest| !is_sha256(digest))
        {
            return Err(CognitionStateError::Invalid(
                "progress detail must be a canonical SHA-256 digest".into(),
            ));
        }
        Ok(())
    }
}
