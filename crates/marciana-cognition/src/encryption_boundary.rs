//! Content-free tenant and encryption-key boundary identities.

use sha2::{Digest, Sha256};

use crate::tenant::{is_valid_component, is_valid_tenant_id};

const MAX_KEY_ID: usize = 256;

/// Deployment-owned key scope. It identifies a cryptographic boundary but
/// never contains key material; encryption and key custody remain host-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionBoundary {
    tenant_id: String,
    key_id: String,
    key_revision: u64,
}

impl EncryptionBoundary {
    /// Bind one tenant to one non-secret key identity and revision.
    ///
    /// # Errors
    /// Returns a fixed error for malformed identities or a zero revision.
    pub fn new(
        tenant_id: String,
        key_id: String,
        key_revision: u64,
    ) -> Result<Self, EncryptionBoundaryError> {
        if !is_valid_tenant_id(&tenant_id) {
            return Err(EncryptionBoundaryError::InvalidTenant);
        }
        if !is_valid_component(&key_id, MAX_KEY_ID) {
            return Err(EncryptionBoundaryError::InvalidKeyId);
        }
        if key_revision == 0 {
            return Err(EncryptionBoundaryError::InvalidRevision);
        }
        Ok(Self {
            tenant_id,
            key_id,
            key_revision,
        })
    }

    /// Tenant identity bound to this boundary.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Non-secret deployment key identity.
    #[must_use]
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    /// Monotonic deployment key revision.
    #[must_use]
    pub const fn key_revision(&self) -> u64 {
        self.key_revision
    }

    /// Stable digest for receipts and persistence metadata.
    #[must_use]
    pub fn digest(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"querygraph.marciana.encryption-boundary.v1\0");
        hasher.update(self.tenant_id.as_bytes());
        hasher.update([0]);
        hasher.update(self.key_id.as_bytes());
        hasher.update([0]);
        hasher.update(self.key_revision.to_be_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }

    /// Verify the exact boundary before reading or writing host-managed data.
    ///
    /// # Errors
    /// Returns [`EncryptionBoundaryError::Mismatch`] for any substituted
    /// tenant, key identity, or revision.
    pub fn matches(
        &self,
        tenant_id: &str,
        key_id: &str,
        key_revision: u64,
    ) -> Result<(), EncryptionBoundaryError> {
        if self.tenant_id == tenant_id && self.key_id == key_id && self.key_revision == key_revision
        {
            Ok(())
        } else {
            Err(EncryptionBoundaryError::Mismatch)
        }
    }

    /// Advance to a new key revision without exposing key material.
    ///
    /// # Errors
    /// Returns a fixed error if the new key identity is invalid or the
    /// revision would overflow.
    pub fn rotate(&self, key_id: String) -> Result<Self, EncryptionBoundaryError> {
        let revision = self
            .key_revision
            .checked_add(1)
            .ok_or(EncryptionBoundaryError::InvalidRevision)?;
        Self::new(self.tenant_id.clone(), key_id, revision)
    }
}

/// Fixed encryption-boundary failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum EncryptionBoundaryError {
    #[error("encryption boundary tenant is invalid")]
    InvalidTenant,
    #[error("encryption boundary key identity is invalid")]
    InvalidKeyId,
    #[error("encryption boundary revision is invalid")]
    InvalidRevision,
    #[error("encryption boundary does not match")]
    Mismatch,
}
