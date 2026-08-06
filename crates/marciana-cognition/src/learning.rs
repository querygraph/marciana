//! Governed learning artifacts. These are proposal metadata, never direct memory writes.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

const MAX_EVIDENCE: usize = 256;
const MAX_FEEDBACK: usize = 10_000;

/// Lifecycle for an evidence-backed derived observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationStatus {
    Proposed,
    Accepted,
    Rejected,
    Expired,
}

/// A recurring pattern whose support is explicit and content-free.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    pub observation_digest: String,
    pub evidence_digests: Vec<String>,
    pub support_count: u32,
    pub confidence_basis_points: u16,
    pub status: ObservationStatus,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
}

impl Observation {
    pub fn propose(
        evidence_digests: Vec<String>,
        support_count: u32,
        confidence_basis_points: u16,
        valid_from: DateTime<Utc>,
    ) -> Result<Self, LearningError> {
        if evidence_digests.is_empty() || evidence_digests.len() > MAX_EVIDENCE {
            return Err(LearningError::Bounds);
        }
        if evidence_digests.iter().any(|digest| !is_digest(digest))
            || support_count == 0
            || usize::try_from(support_count).map_err(|_| LearningError::Bounds)?
                > evidence_digests.len()
            || confidence_basis_points > 10_000
        {
            return Err(LearningError::InvalidEvidence);
        }
        let mut canonical = evidence_digests.clone();
        canonical.sort();
        let observation_digest = digest(&format!(
            "observation-v1|{canonical:?}|{support_count}|{confidence_basis_points}|{valid_from}"
        ));
        Ok(Self {
            observation_digest,
            evidence_digests: canonical,
            support_count,
            confidence_basis_points,
            status: ObservationStatus::Proposed,
            valid_from,
            valid_to: None,
        })
    }

    pub fn transition(
        &mut self,
        next: ObservationStatus,
        at: DateTime<Utc>,
    ) -> Result<(), LearningError> {
        if at < self.valid_from || self.valid_to.is_some() {
            return Err(LearningError::InvalidTransition);
        }
        let allowed = matches!(
            (self.status, next),
            (ObservationStatus::Proposed, ObservationStatus::Accepted)
                | (ObservationStatus::Proposed, ObservationStatus::Rejected)
                | (ObservationStatus::Accepted, ObservationStatus::Expired)
        );
        if !allowed {
            return Err(LearningError::InvalidTransition);
        }
        self.status = next;
        if next == ObservationStatus::Expired {
            self.valid_to = Some(at);
        }
        Ok(())
    }
}

/// One outcome row in an offline feedback corpus; only stable digests persist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackRecord {
    pub trajectory_digest: String,
    pub outcome_basis_points: u16,
    pub recorded_at: DateTime<Utc>,
}

/// Bounded, immutable dataset identity for offline evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackDataset {
    pub dataset_digest: String,
    pub records: Vec<FeedbackRecord>,
}

/// A thresholded offline evaluation bound to one procedure and dataset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationReport {
    pub report_digest: String,
    pub procedure_digest: String,
    pub dataset_digest: String,
    pub score_basis_points: u16,
    pub passed: bool,
}

impl EvaluationReport {
    pub fn new(
        procedure_digest: String,
        dataset_digest: String,
        score_basis_points: u16,
    ) -> Result<Self, LearningError> {
        if !is_digest(&procedure_digest)
            || !is_digest(&dataset_digest)
            || score_basis_points > 10_000
        {
            return Err(LearningError::InvalidFeedback);
        }
        let passed = score_basis_points >= 7_000;
        let report_digest = digest(&format!(
            "evaluation-v1|{procedure_digest}|{dataset_digest}|{score_basis_points}|{passed}"
        ));
        Ok(Self {
            report_digest,
            procedure_digest,
            dataset_digest,
            score_basis_points,
            passed,
        })
    }
}

impl FeedbackDataset {
    pub fn new(mut records: Vec<FeedbackRecord>) -> Result<Self, LearningError> {
        if records.is_empty() || records.len() > MAX_FEEDBACK {
            return Err(LearningError::Bounds);
        }
        if records.iter().any(|record| {
            !is_digest(&record.trajectory_digest) || record.outcome_basis_points > 10_000
        }) {
            return Err(LearningError::InvalidFeedback);
        }
        records.sort_by(|left, right| {
            left.trajectory_digest
                .cmp(&right.trajectory_digest)
                .then_with(|| left.recorded_at.cmp(&right.recorded_at))
        });
        let dataset_digest = digest(&format!("feedback-v1|{records:?}"));
        Ok(Self {
            dataset_digest,
            records,
        })
    }
}

/// Activation state for a versioned procedure artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcedureStatus {
    Proposed,
    Evaluated,
    Approved,
    Active,
    RolledBack,
}

/// Prompt/instruction/tool policy identity without persisting its plaintext.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Procedure {
    pub procedure_digest: String,
    pub schema_version: &'static str,
    pub evaluation_digest: Option<String>,
    pub status: ProcedureStatus,
}

impl Procedure {
    pub fn propose(procedure_digest: String) -> Result<Self, LearningError> {
        if !is_digest(&procedure_digest) {
            return Err(LearningError::InvalidProcedure);
        }
        Ok(Self {
            procedure_digest,
            schema_version: "1",
            evaluation_digest: None,
            status: ProcedureStatus::Proposed,
        })
    }

    pub fn record_evaluation(&mut self, report: &EvaluationReport) -> Result<(), LearningError> {
        if self.status != ProcedureStatus::Proposed
            || report.procedure_digest != self.procedure_digest
            || !report.passed
        {
            return Err(LearningError::InvalidTransition);
        }
        self.evaluation_digest = Some(report.report_digest.clone());
        self.status = ProcedureStatus::Evaluated;
        Ok(())
    }

    pub fn approve(&mut self) -> Result<(), LearningError> {
        if self.status != ProcedureStatus::Evaluated {
            return Err(LearningError::InvalidTransition);
        }
        self.status = ProcedureStatus::Approved;
        Ok(())
    }

    pub fn activate(&mut self) -> Result<(), LearningError> {
        if self.status != ProcedureStatus::Approved || self.evaluation_digest.is_none() {
            return Err(LearningError::InvalidTransition);
        }
        self.status = ProcedureStatus::Active;
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), LearningError> {
        if self.status != ProcedureStatus::Active {
            return Err(LearningError::InvalidTransition);
        }
        self.status = ProcedureStatus::RolledBack;
        Ok(())
    }
}

/// Fixed learning-domain errors that do not disclose caller values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LearningError {
    #[error("learning input exceeds its fixed bound")]
    Bounds,
    #[error("learning evidence is invalid")]
    InvalidEvidence,
    #[error("feedback record is invalid")]
    InvalidFeedback,
    #[error("procedure identity is invalid")]
    InvalidProcedure,
    #[error("learning transition is not permitted")]
    InvalidTransition,
}

fn is_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn digest(value: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(value.as_bytes()))
}
