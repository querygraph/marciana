//! Content-free health data for operational integrations.

use chrono::{DateTime, Utc};

const MAX_COMPONENTS: usize = 32;

/// Availability state of one QueryGraph-stack component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentState {
    Ready,
    Degraded,
    Unavailable,
}

/// Bounded component identity and readiness; no endpoint or memory content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentHealth {
    pub name: String,
    pub revision: String,
    pub state: ComponentState,
}

/// Point-in-time operational snapshot safe to expose to an operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthSnapshot {
    pub schema_version: &'static str,
    pub checked_at: DateTime<Utc>,
    pub components: Vec<ComponentHealth>,
}

/// Health metadata exceeded its fixed component bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("health component bound exceeded")]
pub struct HealthError;

impl HealthSnapshot {
    /// Build a snapshot from trusted deployment metadata.
    ///
    /// # Errors
    /// Returns [`HealthError`] when too many components are declared.
    pub fn new(
        checked_at: DateTime<Utc>,
        components: Vec<ComponentHealth>,
    ) -> Result<Self, HealthError> {
        if components.len() > MAX_COMPONENTS {
            return Err(HealthError);
        }
        Ok(Self {
            schema_version: "marciana-health-v1",
            checked_at,
            components,
        })
    }

    /// Whether every declared component is ready.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        !self.components.is_empty()
            && self
                .components
                .iter()
                .all(|component| component.state == ComponentState::Ready)
    }
}

#[cfg(test)]
mod tests;
