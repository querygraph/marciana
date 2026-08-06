//! `VectorIndex` — an embedding-backed [`SemanticIndex`] with the
//! embedding-privacy rule **enforced by construction**.
//!
//! typesec-memory's `SemanticIndex` documents a contract: content labeled
//! above `Internal` must never be sent to a *remote* embedder (vectors leak
//! content). Here that contract is not merely documented — it is a type-level
//! obligation on the [`Embedder`] and checked at index time:
//!
//! - An [`Embedder`] declares [`Embedder::is_local`]. Only a local embedder
//!   may receive `Sensitive`/`Secret` text.
//! - [`VectorIndex::index`] routes above-`Internal` records to the embedder
//!   only when it is local; otherwise it **declines to index** (returns `Ok`
//!   without storing a vector). The record stays fully recallable through
//!   ordinary `StoreQuery`s — it just does not participate in vector ranking
//!   through a remote embedder. Fail closed: the content never egresses.
//!
//! Search returns the cosine-nearest record ids; the vault still applies the
//! label gate, so ranking is never an authorization path. A hybrid graph
//! re-rank ([`VectorIndex::with_neighbors`]) can boost ids that co-mention an
//! entity, Zep-style — but that is a *reordering* of already-authorized
//! candidates, never a widening.

use std::collections::HashMap;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use typesec_memory::{IndexError, Label, MemoryId, SemanticIndex};

use crate::vector_manifest::{
    VectorIndexManifest, VectorManifestError, VectorRepairBatch, VectorRepairOperation,
};

const MAX_SCOPE_COMPONENT: usize = 256;

/// Turns text into a dense vector. Implementations declare whether they run
/// locally (on-box, no egress) so the index can honor the embedding-privacy
/// rule structurally.
pub trait Embedder: Send + Sync {
    /// Embed `text` into a fixed-width vector.
    fn embed(&self, text: &str) -> Result<Vec<f32>, IndexError>;

    /// Whether embedding happens entirely on-box (no network egress). A
    /// remote embedder (a hosted embedding API) returns `false` and will
    /// never be handed `Sensitive`/`Secret` content by [`VectorIndex`].
    fn is_local(&self) -> bool;
}

/// A cosine-similarity vector index over an [`Embedder`].
pub struct VectorIndex<E: Embedder> {
    embedder: E,
    embedding_space: String,
    vectors: RwLock<HashMap<MemoryId, Vec<f32>>>,
    // id -> entity names it mentions, for the optional hybrid graph re-rank.
    entities: RwLock<HashMap<MemoryId, Vec<String>>>,
}

/// Tenant and embedding-space identity that must accompany a vector index.
/// The digest is safe to persist as an index manifest; it contains no key or
/// memory content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorIndexScope {
    tenant_id: String,
    embedding_space: String,
    digest: String,
}

impl VectorIndexScope {
    /// Create a bounded scope for one tenant and one embedding space.
    ///
    /// # Errors
    /// Returns a fixed error for malformed tenant or embedding-space IDs.
    pub fn new(
        tenant_id: impl Into<String>,
        embedding_space: impl Into<String>,
    ) -> Result<Self, TenantIndexError> {
        let tenant_id = tenant_id.into();
        let embedding_space = embedding_space.into();
        if !valid_scope_component(&tenant_id) {
            return Err(TenantIndexError::InvalidTenant);
        }
        if !valid_scope_component(&embedding_space) {
            return Err(TenantIndexError::InvalidEmbeddingSpace);
        }
        let mut hasher = Sha256::new();
        hasher.update(b"querygraph.marciana.vector-scope.v1\0");
        hasher.update(tenant_id.as_bytes());
        hasher.update([0]);
        hasher.update(embedding_space.as_bytes());
        Ok(Self {
            tenant_id,
            embedding_space,
            digest: format!("sha256:{:x}", hasher.finalize()),
        })
    }

    /// Tenant identity bound to the index.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Embedding-space identity bound to the index.
    #[must_use]
    pub fn embedding_space(&self) -> &str {
        &self.embedding_space
    }

    /// Stable scope digest for durable manifests and repair work.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// Verify a decoded scope was created from canonical components.
    pub fn validate(&self) -> Result<(), TenantIndexError> {
        if !valid_scope_component(&self.tenant_id) {
            return Err(TenantIndexError::InvalidTenant);
        }
        if !valid_scope_component(&self.embedding_space) {
            return Err(TenantIndexError::InvalidEmbeddingSpace);
        }
        let expected = Self::new(&self.tenant_id, &self.embedding_space)
            .map_err(|_| TenantIndexError::InvalidEmbeddingSpace)?;
        if expected.digest != self.digest {
            return Err(TenantIndexError::TenantMismatch);
        }
        Ok(())
    }
}

/// Explicitly tenant-checked vector-index seam. It intentionally does not
/// implement `SemanticIndex`, because every operation must carry its tenant.
pub struct TenantVectorIndex<E: Embedder> {
    scope: VectorIndexScope,
    index: VectorIndex<E>,
    manifest: RwLock<VectorIndexManifest>,
}

impl<E: Embedder> TenantVectorIndex<E> {
    /// Build a tenant-scoped index with an explicit embedding-space identity.
    pub fn new(
        embedder: E,
        tenant_id: impl Into<String>,
        embedding_space: impl Into<String>,
    ) -> Result<Self, TenantIndexError> {
        let scope = VectorIndexScope::new(tenant_id, embedding_space)?;
        let index = VectorIndex::with_embedding_space(embedder, scope.embedding_space.clone())
            .map_err(TenantIndexError::Index)?;
        Ok(Self {
            manifest: RwLock::new(VectorIndexManifest::new(scope.clone())),
            scope,
            index,
        })
    }

    /// Borrow the persisted scope identity.
    #[must_use]
    pub fn scope(&self) -> &VectorIndexScope {
        &self.scope
    }

    /// Index one authorized record under the exact tenant scope.
    pub fn index_for(
        &self,
        tenant_id: &str,
        id: &MemoryId,
        label: Label,
        text: &str,
    ) -> Result<(), TenantIndexError> {
        self.check_tenant(tenant_id)?;
        if !self.index.may_embed(label) {
            return self
                .index
                .index(id, label, text)
                .map_err(TenantIndexError::Index);
        }
        let batch =
            VectorRepairBatch::new(&self.scope, vec![VectorRepairOperation::Index(id.clone())])
                .map_err(TenantIndexError::Manifest)?;
        self.manifest
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .apply(&batch)
            .map_err(TenantIndexError::Manifest)?;
        self.index.index(id, label, text).map_err(|error| {
            let rollback = VectorRepairBatch::new(
                &self.scope,
                vec![VectorRepairOperation::Remove(id.clone())],
            )
            .expect("canonical rollback batch");
            self.manifest
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .apply(&rollback)
                .expect("manifest rollback remains valid");
            TenantIndexError::Index(error)
        })
    }

    /// Remove one record under the exact tenant scope.
    pub fn remove_for(&self, tenant_id: &str, id: &MemoryId) -> Result<(), TenantIndexError> {
        self.check_tenant(tenant_id)?;
        let batch =
            VectorRepairBatch::new(&self.scope, vec![VectorRepairOperation::Remove(id.clone())])
                .map_err(TenantIndexError::Manifest)?;
        self.manifest
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .apply(&batch)
            .map_err(TenantIndexError::Manifest)?;
        self.index.remove(id).map_err(TenantIndexError::Index)
    }

    /// Search under the exact tenant scope.
    pub fn search_for(
        &self,
        tenant_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryId>, TenantIndexError> {
        self.check_tenant(tenant_id)?;
        self.index
            .search(query, limit)
            .map_err(TenantIndexError::Index)
    }

    /// Hybrid search under the exact tenant scope.
    pub fn search_hybrid_for(
        &self,
        tenant_id: &str,
        query: &str,
        limit: usize,
        co_entities: &[String],
    ) -> Result<Vec<MemoryId>, TenantIndexError> {
        self.check_tenant(tenant_id)?;
        self.index
            .search_hybrid(query, limit, co_entities)
            .map_err(TenantIndexError::Index)
    }

    /// Return the content-free membership manifest for host persistence.
    #[must_use]
    pub fn manifest(&self) -> VectorIndexManifest {
        self.manifest
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn check_tenant(&self, tenant_id: &str) -> Result<(), TenantIndexError> {
        if tenant_id == self.scope.tenant_id {
            Ok(())
        } else {
            Err(TenantIndexError::TenantMismatch)
        }
    }
}

/// Fixed tenant-index boundary failures.
#[derive(Debug, thiserror::Error)]
pub enum TenantIndexError {
    #[error("tenant index tenant identity is invalid")]
    InvalidTenant,
    #[error("tenant index embedding-space identity is invalid")]
    InvalidEmbeddingSpace,
    #[error("tenant index scope does not match")]
    TenantMismatch,
    #[error(transparent)]
    Index(#[from] IndexError),
    #[error(transparent)]
    Manifest(#[from] VectorManifestError),
}

fn valid_scope_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SCOPE_COMPONENT
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_:/.-".contains(&byte))
}

impl<E: Embedder> VectorIndex<E> {
    /// Build a vector index over `embedder`.
    pub fn new(embedder: E) -> Self {
        Self::with_embedding_space(embedder, "marciana-reference-embedding-v1")
            .expect("built-in embedding space is canonical")
    }

    /// Build an index with an explicit embedding-space identity.
    ///
    /// Reusing vectors across model, dimension, or preprocessing changes is
    /// unsafe. Callers therefore persist and compare this identity alongside
    /// the index rather than treating all vectors as interchangeable.
    pub fn with_embedding_space(
        embedder: E,
        embedding_space: impl Into<String>,
    ) -> Result<Self, IndexError> {
        let embedding_space = embedding_space.into();
        if embedding_space.is_empty()
            || embedding_space.len() > 128
            || !embedding_space
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
        {
            return Err(IndexError::Backend(
                "invalid embedding-space identity".into(),
            ));
        }
        Ok(Self {
            embedder,
            embedding_space,
            vectors: RwLock::new(HashMap::new()),
            entities: RwLock::new(HashMap::new()),
        })
    }

    /// Stable identity that must accompany persisted vectors and repair work.
    #[must_use]
    pub fn embedding_space(&self) -> &str {
        &self.embedding_space
    }

    /// Record which entities `id` mentions, enabling the hybrid graph re-rank
    /// in [`search_hybrid`](Self::search_hybrid). Optional; a caller wiring
    /// this from the graph store keeps ranking honest without changing what
    /// is *authorized*.
    pub fn note_entities(&self, id: &MemoryId, entities: impl IntoIterator<Item = String>) {
        self.entities
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(id.clone(), entities.into_iter().collect());
    }

    /// May this label's content be sent to the embedder? Above `Internal`
    /// requires a local embedder.
    fn may_embed(&self, label: Label) -> bool {
        label <= Label::Internal || self.embedder.is_local()
    }

    fn ranked(&self, query_vec: &[f32], limit: usize, boost: Option<&[String]>) -> Vec<MemoryId> {
        let vectors = self
            .vectors
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let entities = self
            .entities
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut scored: Vec<(f32, MemoryId)> = vectors
            .iter()
            .map(|(id, vec)| {
                let mut score = cosine(query_vec, vec);
                if let Some(boost) = boost
                    && let Some(mentions) = entities.get(id)
                    && mentions.iter().any(|e| boost.contains(e))
                {
                    // A small, bounded re-rank nudge — reordering, not gating.
                    score += 0.1;
                }
                (score, id.clone())
            })
            // Drop orthogonal (no-signal) records: a zero cosine is not a hit.
            .filter(|(score, _)| *score > 0.0)
            .collect();
        scored.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.cmp(&b.1))
        });
        scored.into_iter().take(limit).map(|(_, id)| id).collect()
    }

    /// Hybrid search: cosine ranking with a bounded boost for records that
    /// mention any of `co_entities` (call [`note_entities`](Self::note_entities)
    /// first). Purely a reordering of vector candidates.
    pub fn search_hybrid(
        &self,
        query: &str,
        limit: usize,
        co_entities: &[String],
    ) -> Result<Vec<MemoryId>, IndexError> {
        let query_vec = self.embedder.embed(query)?;
        Ok(self.ranked(&query_vec, limit, Some(co_entities)))
    }
}

impl<E: Embedder> SemanticIndex for VectorIndex<E> {
    fn index(&self, id: &MemoryId, label: Label, text: &str) -> Result<(), IndexError> {
        if !self.may_embed(label) {
            // Fail closed: decline to index rather than send hot content to a
            // remote embedder. The record stays recallable by ordinary query.
            tracing::debug!(
                %id, label = label.name(),
                "vector index: skipping remote-embed of above-Internal content"
            );
            return Ok(());
        }
        let vector = self.embedder.embed(text)?;
        self.vectors
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(id.clone(), vector);
        Ok(())
    }

    fn remove(&self, id: &MemoryId) -> Result<(), IndexError> {
        self.vectors
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(id);
        self.entities
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(id);
        Ok(())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryId>, IndexError> {
        let query_vec = self.embedder.embed(query)?;
        Ok(self.ranked(&query_vec, limit, None))
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

#[cfg(test)]
mod tests;
