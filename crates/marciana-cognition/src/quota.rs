//! Fixed-array tenant quotas for content-free operation accounting.

use chrono::{DateTime, TimeDelta, Utc};

use crate::OperationKind;

const OPERATION_COUNT: usize = 5;
const MAX_TENANT_ID: usize = 256;

/// Per-window limits in the same order as [`OperationKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuotaLimits {
    pub operations: [u64; OPERATION_COUNT],
}

/// Content-free quota state owned by a deployment host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantQuota {
    pub tenant_id: String,
    pub window_started_at: DateTime<Utc>,
    pub window: TimeDelta,
    pub limits: QuotaLimits,
    usage: [u64; OPERATION_COUNT],
}

/// Exportable quota state without request or memory content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaSnapshot {
    pub tenant_id: String,
    pub window_started_at: DateTime<Utc>,
    pub window: TimeDelta,
    pub limits: QuotaLimits,
    pub usage: [u64; OPERATION_COUNT],
}

impl TenantQuota {
    /// Create a quota window for one canonical tenant identity.
    ///
    /// # Errors
    /// Returns a fixed error when the tenant identity or window is invalid.
    pub fn new(
        tenant_id: String,
        window_started_at: DateTime<Utc>,
        window: TimeDelta,
        limits: QuotaLimits,
    ) -> Result<Self, QuotaError> {
        validate_tenant(&tenant_id)?;
        if window <= TimeDelta::zero() {
            return Err(QuotaError::InvalidWindow);
        }
        Ok(Self {
            tenant_id,
            window_started_at,
            window,
            limits,
            usage: [0; OPERATION_COUNT],
        })
    }

    /// Consume quota, resetting the window only after its boundary.
    ///
    /// # Errors
    /// Returns a fixed error for clock regression, zero amount, or exhaustion.
    pub fn try_consume(
        &mut self,
        operation: OperationKind,
        amount: u64,
        now: DateTime<Utc>,
    ) -> Result<(), QuotaError> {
        if amount == 0 {
            return Err(QuotaError::InvalidAmount);
        }
        if now < self.window_started_at {
            return Err(QuotaError::ClockRegression);
        }
        if now >= self.window_started_at + self.window {
            self.window_started_at = now;
            self.usage = [0; OPERATION_COUNT];
        }
        let index = operation.index();
        let next = self.usage[index].saturating_add(amount);
        if next > self.limits.operations[index] {
            return Err(QuotaError::Exceeded);
        }
        self.usage[index] = next;
        Ok(())
    }

    /// Return the remaining allowance for one operation in this window.
    #[must_use]
    pub fn remaining(&self, operation: OperationKind) -> u64 {
        self.limits.operations[operation.index()].saturating_sub(self.usage[operation.index()])
    }

    /// Return content-free state suitable for host export.
    #[must_use]
    pub fn snapshot(&self) -> QuotaSnapshot {
        QuotaSnapshot {
            tenant_id: self.tenant_id.clone(),
            window_started_at: self.window_started_at,
            window: self.window,
            limits: self.limits,
            usage: self.usage,
        }
    }
}

/// Fixed tenant-quota failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum QuotaError {
    #[error("tenant identity is invalid")]
    InvalidTenant,
    #[error("quota window is invalid")]
    InvalidWindow,
    #[error("quota amount is invalid")]
    InvalidAmount,
    #[error("quota clock moved backwards")]
    ClockRegression,
    #[error("tenant operation quota is exhausted")]
    Exceeded,
}

fn validate_tenant(value: &str) -> Result<(), QuotaError> {
    if value.is_empty()
        || value.len() > MAX_TENANT_ID
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_:/.-".contains(&byte))
    {
        Err(QuotaError::InvalidTenant)
    } else {
        Ok(())
    }
}
