#![cfg(feature = "turso")]

use querygraph_memory::turso::TursoConfig;
use querygraph_memory::{
    TursoMemoryStore, VectorIndexManifest, VectorIndexScope, VectorRepairBatch,
    VectorRepairOperation,
};
use tempfile::TempDir;
use typesec_memory::MemoryId;

fn config(dir: &TempDir) -> TursoConfig {
    TursoConfig {
        path: dir
            .path()
            .join("vector-manifest.db")
            .to_string_lossy()
            .into_owned(),
        table_prefix: "vector_manifest".into(),
        ..TursoConfig::default()
    }
}

#[test]
fn manifest_persists_and_recovers_through_guarded_graph_storage() {
    let dir = tempfile::tempdir().unwrap();
    let scope = VectorIndexScope::new("tenant-a", "embed-v1").unwrap();
    let mut manifest = VectorIndexManifest::new(scope.clone());
    manifest
        .apply(
            &VectorRepairBatch::new(
                &scope,
                vec![VectorRepairOperation::Index(MemoryId::from_string("mem-a"))],
            )
            .unwrap(),
        )
        .unwrap();
    {
        let store = TursoMemoryStore::open_with_config(config(&dir)).unwrap();
        store.persist_vector_manifest(&manifest).unwrap();
        store.persist_vector_manifest(&manifest).unwrap();
    }
    let reopened = TursoMemoryStore::open_with_config(config(&dir)).unwrap();
    let recovered = reopened.load_vector_manifest(&scope).unwrap().unwrap();
    assert_eq!(recovered, manifest);
}

#[test]
fn missing_manifest_is_distinct_from_corrupt_scope() {
    let dir = tempfile::tempdir().unwrap();
    let store = TursoMemoryStore::open_with_config(config(&dir)).unwrap();
    let scope = VectorIndexScope::new("tenant-a", "embed-v1").unwrap();
    assert!(store.load_vector_manifest(&scope).unwrap().is_none());
}
