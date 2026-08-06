//! Content-free backup/restore compatibility contracts.

use chrono::{DateTime, Utc};

const MAX_COMPONENTS: usize = 32;

/// Immutable metadata describing one exported deployment snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupManifest {
    pub schema_version: &'static str,
    pub backup_id: String,
    pub created_at: DateTime<Utc>,
    pub database_schema: String,
    pub component_revisions: Vec<(String, String)>,
}

/// Fixed restore validation failures; no storage values are echoed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RestoreError {
    #[error("backup manifest is invalid")]
    InvalidManifest,
    #[error("backup schema is incompatible")]
    IncompatibleSchema,
}

impl BackupManifest {
    /// Create a manifest from trusted deployment metadata.
    ///
    /// # Errors
    /// Returns [`RestoreError::InvalidManifest`] for empty or oversized metadata.
    pub fn new(
        backup_id: String,
        created_at: DateTime<Utc>,
        database_schema: String,
        component_revisions: Vec<(String, String)>,
    ) -> Result<Self, RestoreError> {
        if backup_id.is_empty()
            || backup_id.len() > 256
            || database_schema.is_empty()
            || database_schema.len() > 128
            || component_revisions.is_empty()
            || component_revisions.len() > MAX_COMPONENTS
            || component_revisions
                .iter()
                .any(|(name, revision)| name.is_empty() || revision.is_empty())
        {
            return Err(RestoreError::InvalidManifest);
        }
        Ok(Self {
            schema_version: "marciana-backup-v1",
            backup_id,
            created_at,
            database_schema,
            component_revisions,
        })
    }

    /// Verify that a restore target supports this manifest schema.
    ///
    /// # Errors
    /// Returns [`RestoreError::IncompatibleSchema`] when the target differs.
    pub fn validate_restore(&self, supported_schema: &str) -> Result<(), RestoreError> {
        if self.schema_version != "marciana-backup-v1" {
            return Err(RestoreError::IncompatibleSchema);
        }
        if supported_schema != self.database_schema {
            return Err(RestoreError::IncompatibleSchema);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
