//! Proposal-only cohort rollout and trajectory-retention policy.

use crate::learning::{Procedure, ProcedureStatus};
use sha2::{Digest, Sha256};

const MAX_COHORT_ID: usize = 256;
const MAX_RETENTION_DAYS: u16 = 3_650;

/// Lifecycle for a bounded procedure cohort rollout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcedureRolloutStatus {
    Proposed,
    Approved,
    Active,
    RolledBack,
}

/// Content-free deployment policy for one evaluated procedure and cohort.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureRollout {
    pub rollout_digest: String,
    pub procedure_digest: String,
    pub evaluation_digest: String,
    pub cohort_digest: String,
    pub traffic_basis_points: u16,
    pub trajectory_retention_days: u16,
    pub status: ProcedureRolloutStatus,
}

impl ProcedureRollout {
    /// Propose a rollout bound to an approved procedure and exact evaluation.
    ///
    /// # Errors
    /// Returns a fixed error for invalid identities, bounds, or procedure
    /// lifecycle state.
    pub fn propose(
        procedure: &Procedure,
        cohort_digest: String,
        traffic_basis_points: u16,
        trajectory_retention_days: u16,
    ) -> Result<Self, ProcedureRolloutError> {
        if !matches!(
            procedure.status,
            ProcedureStatus::Approved | ProcedureStatus::Active
        ) || procedure.evaluation_digest.is_none()
        {
            return Err(ProcedureRolloutError::ProcedureState);
        }
        validate_fields(
            &procedure.procedure_digest,
            procedure.evaluation_digest.as_deref().unwrap_or_default(),
            &cohort_digest,
            traffic_basis_points,
            trajectory_retention_days,
        )?;
        let rollout_digest = rollout_digest(
            &procedure.procedure_digest,
            procedure.evaluation_digest.as_deref().unwrap_or_default(),
            &cohort_digest,
            traffic_basis_points,
            trajectory_retention_days,
        );
        Ok(Self {
            rollout_digest,
            procedure_digest: procedure.procedure_digest.clone(),
            evaluation_digest: procedure.evaluation_digest.clone().unwrap_or_default(),
            cohort_digest,
            traffic_basis_points,
            trajectory_retention_days,
            status: ProcedureRolloutStatus::Proposed,
        })
    }

    /// Validate immutable rollout identity and bounds.
    ///
    /// # Errors
    /// Returns [`ProcedureRolloutError::Digest`] when metadata was modified.
    pub fn validate(&self) -> Result<(), ProcedureRolloutError> {
        validate_fields(
            &self.procedure_digest,
            &self.evaluation_digest,
            &self.cohort_digest,
            self.traffic_basis_points,
            self.trajectory_retention_days,
        )?;
        if rollout_digest(
            &self.procedure_digest,
            &self.evaluation_digest,
            &self.cohort_digest,
            self.traffic_basis_points,
            self.trajectory_retention_days,
        ) != self.rollout_digest
        {
            return Err(ProcedureRolloutError::Digest);
        }
        Ok(())
    }

    /// Approve a proposed rollout without activating it.
    ///
    /// # Errors
    /// Returns [`ProcedureRolloutError`] when identity is invalid or the
    /// rollout is not proposed.
    pub fn approve(&mut self) -> Result<(), ProcedureRolloutError> {
        self.validate()?;
        if self.status != ProcedureRolloutStatus::Proposed {
            return Err(ProcedureRolloutError::Transition);
        }
        self.status = ProcedureRolloutStatus::Approved;
        Ok(())
    }

    /// Activate only when the exact evaluated procedure is already active.
    ///
    /// # Errors
    /// Returns [`ProcedureRolloutError`] when the procedure identity or
    /// lifecycle state does not match this rollout.
    pub fn activate(&mut self, procedure: &Procedure) -> Result<(), ProcedureRolloutError> {
        self.validate()?;
        if self.status != ProcedureRolloutStatus::Approved
            || procedure.status != ProcedureStatus::Active
            || procedure.procedure_digest != self.procedure_digest
            || procedure.evaluation_digest.as_deref() != Some(&self.evaluation_digest)
        {
            return Err(ProcedureRolloutError::ProcedureState);
        }
        self.status = ProcedureRolloutStatus::Active;
        Ok(())
    }

    /// Roll back an active cohort rollout.
    ///
    /// # Errors
    /// Returns [`ProcedureRolloutError`] when identity is invalid or the
    /// rollout is not active.
    pub fn rollback(&mut self) -> Result<(), ProcedureRolloutError> {
        self.validate()?;
        if self.status != ProcedureRolloutStatus::Active {
            return Err(ProcedureRolloutError::Transition);
        }
        self.status = ProcedureRolloutStatus::RolledBack;
        Ok(())
    }
}

/// Fixed procedure-rollout validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ProcedureRolloutError {
    #[error("procedure rollout identity is invalid")]
    Identity,
    #[error("procedure rollout bounds are invalid")]
    Bounds,
    #[error("procedure rollout digest is invalid")]
    Digest,
    #[error("procedure rollout procedure state is invalid")]
    ProcedureState,
    #[error("procedure rollout transition is not permitted")]
    Transition,
}

fn validate_fields(
    procedure_digest: &str,
    evaluation_digest: &str,
    cohort_digest: &str,
    traffic_basis_points: u16,
    trajectory_retention_days: u16,
) -> Result<(), ProcedureRolloutError> {
    if !is_digest(procedure_digest) || !is_digest(evaluation_digest) || !is_digest(cohort_digest) {
        return Err(ProcedureRolloutError::Identity);
    }
    if traffic_basis_points == 0
        || traffic_basis_points > 10_000
        || trajectory_retention_days == 0
        || trajectory_retention_days > MAX_RETENTION_DAYS
        || cohort_digest.len() > MAX_COHORT_ID
    {
        return Err(ProcedureRolloutError::Bounds);
    }
    Ok(())
}

fn rollout_digest(
    procedure_digest: &str,
    evaluation_digest: &str,
    cohort_digest: &str,
    traffic_basis_points: u16,
    trajectory_retention_days: u16,
) -> String {
    let value = format!(
        "procedure-rollout-v1|{procedure_digest}|{evaluation_digest}|{cohort_digest}|{traffic_basis_points}|{trajectory_retention_days}"
    );
    format!("sha256:{:x}", Sha256::digest(value.as_bytes()))
}

fn is_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}
