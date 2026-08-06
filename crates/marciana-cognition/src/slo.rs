//! Deterministic, content-free service-level objective evaluation.

use crate::{MetricsSnapshot, OperationKind};

const OPERATION_COUNT: usize = 5;
const BASIS_POINTS: u128 = 10_000;

/// Per-operation SLO limits. Denial limits are expressed in basis points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SloPolicy {
    pub max_latency_micros: [u64; OPERATION_COUNT],
    pub max_denial_basis_points: [u16; OPERATION_COUNT],
}

/// Content-free result of evaluating one metrics snapshot against a policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SloReport {
    pub observed: [bool; OPERATION_COUNT],
    pub compliant: [bool; OPERATION_COUNT],
    pub max_latency_micros: [u64; OPERATION_COUNT],
    pub denial_basis_points: [u16; OPERATION_COUNT],
}

impl SloPolicy {
    /// Create a policy with positive latency limits and valid percentages.
    ///
    /// # Errors
    /// Returns a fixed error when a target cannot be evaluated safely.
    pub fn new(
        max_latency_micros: [u64; OPERATION_COUNT],
        max_denial_basis_points: [u16; OPERATION_COUNT],
    ) -> Result<Self, SloError> {
        if max_latency_micros.contains(&0) {
            return Err(SloError::InvalidLatencyTarget);
        }
        if max_denial_basis_points.iter().any(|value| *value > 10_000) {
            return Err(SloError::InvalidDenialTarget);
        }
        Ok(Self {
            max_latency_micros,
            max_denial_basis_points,
        })
    }

    /// Evaluate bounded operation metrics without inspecting request data.
    #[must_use]
    pub fn evaluate(self, metrics: MetricsSnapshot) -> SloReport {
        let mut observed = [false; OPERATION_COUNT];
        let mut compliant = [true; OPERATION_COUNT];
        let mut denial_basis_points = [0; OPERATION_COUNT];
        for index in 0..OPERATION_COUNT {
            observed[index] = metrics.counts[index] != 0;
            denial_basis_points[index] =
                denial_rate_basis_points(metrics.denials[index], metrics.counts[index]);
            compliant[index] = !observed[index]
                || (metrics.max_latency_micros[index] <= self.max_latency_micros[index]
                    && denial_basis_points[index] <= self.max_denial_basis_points[index]);
        }
        SloReport {
            observed,
            compliant,
            max_latency_micros: metrics.max_latency_micros,
            denial_basis_points,
        }
    }
}

impl SloReport {
    /// Whether every operation with traffic met its configured SLO.
    #[must_use]
    pub fn is_compliant(self) -> bool {
        self.compliant.iter().all(|value| *value)
    }

    /// Return the result for one lifecycle operation.
    #[must_use]
    pub fn operation(self, operation: OperationKind) -> (bool, bool, u64, u16) {
        let index = operation.index();
        (
            self.observed[index],
            self.compliant[index],
            self.max_latency_micros[index],
            self.denial_basis_points[index],
        )
    }
}

/// Fixed SLO policy validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SloError {
    #[error("SLO latency target is invalid")]
    InvalidLatencyTarget,
    #[error("SLO denial target is invalid")]
    InvalidDenialTarget,
}

fn denial_rate_basis_points(denials: u64, count: u64) -> u16 {
    if count == 0 {
        return 0;
    }
    let numerator = u128::from(denials).saturating_mul(BASIS_POINTS);
    let rate = numerator.saturating_add(u128::from(count).saturating_sub(1)) / u128::from(count);
    u16::try_from(rate.min(BASIS_POINTS)).unwrap_or(10_000)
}
