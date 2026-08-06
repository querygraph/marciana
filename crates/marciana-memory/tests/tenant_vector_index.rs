use std::sync::atomic::{AtomicUsize, Ordering};

use querygraph_memory::{Embedder, TenantIndexError, TenantVectorIndex};
use typesec_memory::{IndexError, Label, MemoryId};

struct EmbedderStub {
    calls: AtomicUsize,
}

impl Embedder for EmbedderStub {
    fn embed(&self, text: &str) -> Result<Vec<f32>, IndexError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if text.contains("poison") {
            return Err(IndexError::Backend("embedder rejected content".into()));
        }
        Ok(vec![if text.contains("coffee") { 1.0 } else { 0.0 }])
    }

    fn is_local(&self) -> bool {
        true
    }
}

fn id(value: &str) -> MemoryId {
    MemoryId::from_string(value)
}

#[test]
fn scoped_index_binds_tenant_and_embedding_space() {
    let index = TenantVectorIndex::new(
        EmbedderStub {
            calls: AtomicUsize::new(0),
        },
        "tenant:coffee",
        "model-v1:384",
    )
    .expect("scoped index");
    assert_eq!(index.scope().tenant_id(), "tenant:coffee");
    assert_eq!(index.scope().embedding_space(), "model-v1:384");
    assert!(index.scope().digest().starts_with("sha256:"));
    index
        .index_for("tenant:coffee", &id("m1"), Label::Public, "coffee price")
        .expect("index");
    assert_eq!(index.manifest().indexed_ids().collect::<Vec<_>>(), ["m1"]);
    assert_eq!(
        index.search_for("tenant:coffee", "coffee", 1).unwrap(),
        vec![id("m1")]
    );
    index.remove_for("tenant:coffee", &id("m1")).unwrap();
    assert!(index.manifest().indexed_ids().next().is_none());
}

#[test]
fn failed_reindex_preserves_prior_manifest_membership() {
    let index = TenantVectorIndex::new(
        EmbedderStub {
            calls: AtomicUsize::new(0),
        },
        "tenant:coffee",
        "model-v1:384",
    )
    .expect("scoped index");
    index
        .index_for("tenant:coffee", &id("m1"), Label::Public, "coffee price")
        .expect("index");
    assert!(matches!(
        index.index_for("tenant:coffee", &id("m1"), Label::Public, "poison"),
        Err(TenantIndexError::Index(_))
    ));
    // The old vector is still searchable, so membership must survive the
    // failed re-index; only a newly inserted id is rolled back.
    assert_eq!(index.manifest().indexed_ids().collect::<Vec<_>>(), ["m1"]);
    assert_eq!(
        index.search_for("tenant:coffee", "coffee", 1).unwrap(),
        vec![id("m1")]
    );
    assert!(matches!(
        index.index_for("tenant:coffee", &id("m2"), Label::Public, "poison"),
        Err(TenantIndexError::Index(_))
    ));
    assert_eq!(index.manifest().indexed_ids().collect::<Vec<_>>(), ["m1"]);
}

#[test]
fn scoped_index_rejects_cross_tenant_operations() {
    let index = TenantVectorIndex::new(
        EmbedderStub {
            calls: AtomicUsize::new(0),
        },
        "tenant:coffee",
        "model-v1:384",
    )
    .expect("scoped index");
    assert!(matches!(
        index.search_for("tenant:other", "coffee", 1),
        Err(TenantIndexError::TenantMismatch)
    ));
    assert!(matches!(
        index.remove_for("tenant:other", &id("m1")),
        Err(TenantIndexError::TenantMismatch)
    ));
}

#[test]
fn scoped_index_rejects_invalid_scope_without_echoing_values() {
    assert!(matches!(
        TenantVectorIndex::new(
            EmbedderStub {
                calls: AtomicUsize::new(0),
            },
            "tenant with space",
            "model-v1"
        ),
        Err(TenantIndexError::InvalidTenant)
    ));
    assert!(matches!(
        TenantVectorIndex::new(
            EmbedderStub {
                calls: AtomicUsize::new(0),
            },
            "tenant:coffee",
            "model v1"
        ),
        Err(TenantIndexError::InvalidEmbeddingSpace)
    ));
}
