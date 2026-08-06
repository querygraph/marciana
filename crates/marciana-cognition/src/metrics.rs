//! Bounded, content-free operation metrics for dashboards and SLO checks.

/// The authoritative memory lifecycle verbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationKind {
    Remember,
    Recall,
    Improve,
    Forget,
}

impl OperationKind {
    const fn index(self) -> usize {
        match self {
            Self::Remember => 0,
            Self::Recall => 1,
            Self::Improve => 2,
            Self::Forget => 3,
        }
    }
}

/// One bounded operation observation; it contains no request or memory data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationSample {
    pub operation: OperationKind,
    pub allowed: bool,
    pub latency_micros: u64,
}

/// Aggregated counters safe for operational export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricsSnapshot {
    pub counts: [u64; 4],
    pub denials: [u64; 4],
    pub total_latency_micros: [u128; 4],
    pub max_latency_micros: [u64; 4],
}

/// In-process metrics collector. Persistence/export belongs to the host.
#[derive(Debug, Clone, Copy, Default)]
pub struct OperationMetrics {
    counts: [u64; 4],
    denials: [u64; 4],
    total_latency_micros: [u128; 4],
    max_latency_micros: [u64; 4],
}

impl OperationMetrics {
    /// Record one operation observation.
    pub fn record(&mut self, sample: OperationSample) {
        let index = sample.operation.index();
        self.counts[index] = self.counts[index].saturating_add(1);
        if !sample.allowed {
            self.denials[index] = self.denials[index].saturating_add(1);
        }
        self.total_latency_micros[index] =
            self.total_latency_micros[index].saturating_add(u128::from(sample.latency_micros));
        self.max_latency_micros[index] = self.max_latency_micros[index].max(sample.latency_micros);
    }

    /// Return a copy suitable for serialization or export.
    #[must_use]
    pub const fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            counts: self.counts,
            denials: self.denials,
            total_latency_micros: self.total_latency_micros,
            max_latency_micros: self.max_latency_micros,
        }
    }
}

#[cfg(test)]
mod tests;
