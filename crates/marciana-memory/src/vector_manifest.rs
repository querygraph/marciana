//! Content-free durable vector-index manifests and atomic ID-only repairs.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use typesec_memory::MemoryId;

use crate::vector::VectorIndexScope;

const MAX_MANIFEST_IDS: usize = 100_000;

/// A host-persistable identity and membership manifest for one tenant index.
/// It contains no embeddings or source content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorIndexManifest {
    scope: VectorIndexScope,
    indexed_ids: BTreeSet<String>,
    digest: String,
}

/// One ID-only repair operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VectorRepairOperation {
    Index(MemoryId),
    Remove(MemoryId),
}

/// A bounded repair batch tied to one exact tenant and embedding space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorRepairBatch {
    scope_digest: String,
    operations: Vec<VectorRepairOperation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum VectorManifestError {
    #[error("vector manifest is full")]
    Capacity,
    #[error("vector repair scope does not match")]
    ScopeMismatch,
    #[error("vector repair batch is invalid")]
    InvalidBatch,
}

impl VectorIndexManifest {
    #[must_use]
    pub fn new(scope: VectorIndexScope) -> Self {
        let mut manifest = Self {
            scope,
            indexed_ids: BTreeSet::new(),
            digest: String::new(),
        };
        manifest.refresh_digest();
        manifest
    }

    #[must_use]
    pub fn scope(&self) -> &VectorIndexScope {
        &self.scope
    }

    pub fn indexed_ids(&self) -> impl Iterator<Item = &str> {
        self.indexed_ids.iter().map(String::as_str)
    }

    /// Whether `id` is currently a manifest member.
    #[must_use]
    pub fn contains(&self, id: &MemoryId) -> bool {
        self.indexed_ids.contains(id.as_str())
    }

    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// Verify the persisted membership and digest identity.
    ///
    /// # Errors
    ///
    /// Returns [`VectorManifestError`] when a scope component or the
    /// membership digest is invalid.
    pub fn validate(&self) -> Result<(), VectorManifestError> {
        self.scope
            .validate()
            .map_err(|_| VectorManifestError::InvalidBatch)?;
        if self.indexed_ids.len() > MAX_MANIFEST_IDS
            || self.indexed_ids.iter().any(|id| {
                id.is_empty()
                    || id.len() > 256
                    || !id
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"_:/.-".contains(&byte))
            })
        {
            return Err(VectorManifestError::InvalidBatch);
        }
        if manifest_digest(&self.scope, &self.indexed_ids) != self.digest {
            return Err(VectorManifestError::InvalidBatch);
        }
        Ok(())
    }

    /// Apply a repair batch atomically in memory. Hosts can persist the
    /// resulting manifest under their own transaction after this succeeds.
    ///
    /// # Errors
    ///
    /// Returns [`VectorManifestError`] when the batch scope does not match or
    /// an operation violates the manifest bounds.
    pub fn apply(&mut self, batch: &VectorRepairBatch) -> Result<(), VectorManifestError> {
        if batch.scope_digest != self.scope.digest() {
            return Err(VectorManifestError::ScopeMismatch);
        }
        let mut changes = BTreeMap::new();
        for operation in &batch.operations {
            let id = operation.id();
            if id.as_str().is_empty() || id.as_str().len() > 256 {
                return Err(VectorManifestError::InvalidBatch);
            }
            changes.insert(id.as_str(), operation.indexes());
        }
        let mut next_len = self.indexed_ids.len();
        for (id, indexes) in &changes {
            match (*indexes, self.indexed_ids.contains(*id)) {
                (true, false) => {
                    next_len = next_len
                        .checked_add(1)
                        .ok_or(VectorManifestError::Capacity)?;
                }
                (false, true) => next_len -= 1,
                _ => {}
            }
        }
        if next_len > MAX_MANIFEST_IDS {
            return Err(VectorManifestError::Capacity);
        }
        for (id, indexes) in changes {
            if indexes {
                self.indexed_ids.insert(id.to_owned());
            } else {
                self.indexed_ids.remove(id);
            }
        }
        self.refresh_digest();
        Ok(())
    }

    fn refresh_digest(&mut self) {
        self.digest = manifest_digest(&self.scope, &self.indexed_ids);
    }
}

impl VectorRepairOperation {
    fn id(&self) -> &MemoryId {
        match self {
            Self::Index(id) | Self::Remove(id) => id,
        }
    }

    fn indexes(&self) -> bool {
        matches!(self, Self::Index(_))
    }
}

impl VectorRepairBatch {
    ///
    /// # Errors
    ///
    /// Returns [`VectorManifestError`] when a scope component or repair
    /// operation is invalid.
    pub fn new(
        scope: &VectorIndexScope,
        operations: Vec<VectorRepairOperation>,
    ) -> Result<Self, VectorManifestError> {
        if operations.len() > MAX_MANIFEST_IDS {
            return Err(VectorManifestError::Capacity);
        }
        if operations.iter().any(|operation| {
            let id = operation.id();
            id.as_str().is_empty()
                || id.as_str().len() > 256
                || !id
                    .as_str()
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_:/.-".contains(&byte))
        }) {
            return Err(VectorManifestError::InvalidBatch);
        }
        Ok(Self {
            scope_digest: scope.digest().to_owned(),
            operations,
        })
    }

    #[must_use]
    pub fn scope_digest(&self) -> &str {
        &self.scope_digest
    }

    #[must_use]
    pub fn operations(&self) -> &[VectorRepairOperation] {
        &self.operations
    }
}

fn manifest_digest(scope: &VectorIndexScope, indexed_ids: &BTreeSet<String>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.vector-manifest.v1\0");
    hasher.update(scope.digest().as_bytes());
    for id in indexed_ids {
        hasher.update(id.as_bytes());
        hasher.update([0]);
    }
    format!("sha256:{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: impl Into<String>) -> MemoryId {
        MemoryId::from_string(value.into())
    }

    #[test]
    fn repair_delta_preserves_sequential_last_operation_semantics() {
        let scope = VectorIndexScope::new("tenant-a", "embed-v1").expect("valid scope");
        let mut manifest = VectorIndexManifest::new(scope.clone());
        manifest
            .apply(
                &VectorRepairBatch::new(
                    &scope,
                    vec![
                        VectorRepairOperation::Index(id("memory-a")),
                        VectorRepairOperation::Remove(id("memory-a")),
                        VectorRepairOperation::Index(id("memory-a")),
                        VectorRepairOperation::Index(id("memory-b")),
                        VectorRepairOperation::Remove(id("memory-b")),
                    ],
                )
                .expect("valid repair"),
            )
            .expect("apply repair");

        assert!(manifest.contains(&id("memory-a")));
        assert!(!manifest.contains(&id("memory-b")));
    }

    #[test]
    fn capacity_rejection_leaves_a_full_manifest_unchanged() {
        let scope = VectorIndexScope::new("tenant-a", "embed-v1").expect("valid scope");
        let mut manifest = VectorIndexManifest::new(scope.clone());
        let seed = VectorRepairBatch::new(
            &scope,
            (0..MAX_MANIFEST_IDS)
                .map(|index| VectorRepairOperation::Index(id(format!("memory-{index:08}"))))
                .collect(),
        )
        .expect("capacity-sized repair");
        manifest.apply(&seed).expect("fill manifest");
        let before = manifest.digest().to_owned();
        let overflow = VectorRepairBatch::new(
            &scope,
            vec![VectorRepairOperation::Index(id("memory-overflow"))],
        )
        .expect("bounded overflow repair");

        assert_eq!(
            manifest.apply(&overflow),
            Err(VectorManifestError::Capacity)
        );
        assert_eq!(manifest.digest(), before);
        assert!(!manifest.contains(&id("memory-overflow")));
    }
}
