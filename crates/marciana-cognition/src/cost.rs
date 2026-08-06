//! Bounded, tenant-scoped resource accounting without request or memory data.

use crate::OperationKind;
use crate::tenant::is_valid_tenant_id;

const OPERATION_COUNT: usize = 5;

/// Deployment-selected price per unit, expressed in integer microcredits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostRates {
    pub source_record: u64,
    pub output_record: u64,
    pub input_byte: u64,
    pub output_byte: u64,
    pub compute_microsecond: u64,
}

/// One content-free resource observation for a lifecycle operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostSample {
    pub operation: OperationKind,
    pub source_records: u32,
    pub output_records: u32,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub compute_microseconds: u64,
}

/// Aggregated resource usage and deterministic microcredit estimate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostSnapshot {
    pub tenant_id: String,
    pub rates: CostRates,
    pub samples: [u64; OPERATION_COUNT],
    pub source_records: [u64; OPERATION_COUNT],
    pub output_records: [u64; OPERATION_COUNT],
    pub input_bytes: [u128; OPERATION_COUNT],
    pub output_bytes: [u128; OPERATION_COUNT],
    pub compute_microseconds: [u128; OPERATION_COUNT],
    pub microcredits: [u128; OPERATION_COUNT],
}

/// Host-owned accumulator. Instantiate one meter per tenant and persistence
/// window; the meter never stores source, prompt, model, or memory values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantCostAccounting {
    tenant_id: String,
    rates: CostRates,
    samples: [u64; OPERATION_COUNT],
    source_records: [u64; OPERATION_COUNT],
    output_records: [u64; OPERATION_COUNT],
    input_bytes: [u128; OPERATION_COUNT],
    output_bytes: [u128; OPERATION_COUNT],
    compute_microseconds: [u128; OPERATION_COUNT],
    microcredits: [u128; OPERATION_COUNT],
}

impl TenantCostAccounting {
    /// Create a meter for one canonical tenant identity.
    ///
    /// # Errors
    /// Returns [`CostError::InvalidTenant`] for an unbounded or malformed ID.
    pub fn new(tenant_id: String, rates: CostRates) -> Result<Self, CostError> {
        if !is_valid_tenant_id(&tenant_id) {
            return Err(CostError::InvalidTenant);
        }
        Ok(Self {
            tenant_id,
            rates,
            samples: [0; OPERATION_COUNT],
            source_records: [0; OPERATION_COUNT],
            output_records: [0; OPERATION_COUNT],
            input_bytes: [0; OPERATION_COUNT],
            output_bytes: [0; OPERATION_COUNT],
            compute_microseconds: [0; OPERATION_COUNT],
            microcredits: [0; OPERATION_COUNT],
        })
    }

    /// Record one bounded resource observation using saturating arithmetic.
    pub fn record(&mut self, sample: CostSample) {
        let index = sample.operation.index();
        self.samples[index] = self.samples[index].saturating_add(1);
        self.source_records[index] =
            self.source_records[index].saturating_add(u64::from(sample.source_records));
        self.output_records[index] =
            self.output_records[index].saturating_add(u64::from(sample.output_records));
        self.input_bytes[index] =
            self.input_bytes[index].saturating_add(u128::from(sample.input_bytes));
        self.output_bytes[index] =
            self.output_bytes[index].saturating_add(u128::from(sample.output_bytes));
        self.compute_microseconds[index] = self.compute_microseconds[index]
            .saturating_add(u128::from(sample.compute_microseconds));
        self.microcredits[index] =
            self.microcredits[index].saturating_add(cost_of(sample, self.rates));
    }

    /// Return a content-free copy suitable for host persistence or export.
    #[must_use]
    pub fn snapshot(&self) -> CostSnapshot {
        CostSnapshot {
            tenant_id: self.tenant_id.clone(),
            rates: self.rates,
            samples: self.samples,
            source_records: self.source_records,
            output_records: self.output_records,
            input_bytes: self.input_bytes,
            output_bytes: self.output_bytes,
            compute_microseconds: self.compute_microseconds,
            microcredits: self.microcredits,
        }
    }
}

/// Fixed cost-accounting failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CostError {
    #[error("tenant identity is invalid")]
    InvalidTenant,
}

fn cost_of(sample: CostSample, rates: CostRates) -> u128 {
    u128::from(sample.source_records)
        .saturating_mul(u128::from(rates.source_record))
        .saturating_add(
            u128::from(sample.output_records).saturating_mul(u128::from(rates.output_record)),
        )
        .saturating_add(u128::from(sample.input_bytes).saturating_mul(u128::from(rates.input_byte)))
        .saturating_add(
            u128::from(sample.output_bytes).saturating_mul(u128::from(rates.output_byte)),
        )
        .saturating_add(
            u128::from(sample.compute_microseconds)
                .saturating_mul(u128::from(rates.compute_microsecond)),
        )
}
