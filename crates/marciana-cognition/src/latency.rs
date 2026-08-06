//! Bounded content-free latency percentiles for reproducible evaluation.

use sha2::{Digest, Sha256};

const MAX_SAMPLES: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatencyPercentiles {
    pub count: usize,
    pub p50_micros: u64,
    pub p95_micros: u64,
    pub p99_micros: u64,
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LatencyError {
    #[error("latency sample capacity exceeded")]
    Capacity,
}

#[derive(Debug, Clone, Default)]
pub struct LatencySamples {
    samples: Vec<u64>,
}

impl LatencySamples {
    /// Record one bounded latency observation.
    ///
    /// # Errors
    ///
    /// Returns [`LatencyError::Capacity`] after the fixed sample bound.
    pub fn record(&mut self, latency_micros: u64) -> Result<(), LatencyError> {
        if self.samples.len() >= MAX_SAMPLES {
            return Err(LatencyError::Capacity);
        }
        self.samples.push(latency_micros);
        Ok(())
    }

    #[must_use]
    pub fn snapshot(&self) -> LatencyPercentiles {
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        LatencyPercentiles {
            count: sorted.len(),
            p50_micros: percentile(&sorted, 50),
            p95_micros: percentile(&sorted, 95),
            p99_micros: percentile(&sorted, 99),
            digest: digest(&sorted),
        }
    }
}

fn percentile(sorted: &[u64], percentage: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = (percentage * sorted.len()).div_ceil(100).max(1);
    sorted[rank - 1]
}

fn digest(sorted: &[u64]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.latency-percentiles.v1\0");
    for sample in sorted {
        hasher.update(sample.to_be_bytes());
    }
    format!("sha256:{:x}", hasher.finalize())
}
