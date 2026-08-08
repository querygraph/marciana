use querygraph_memory::{
    VectorIndexManifest, VectorIndexScope, VectorManifestError, VectorRepairBatch,
    VectorRepairOperation,
};
use typesec_memory::MemoryId;

fn id(value: &str) -> MemoryId {
    MemoryId::from_string(value)
}

#[test]
fn manifest_and_repair_are_scope_bound_and_deterministic() {
    let scope = VectorIndexScope::new("tenant-a", "embed-v1").unwrap();
    let mut manifest = VectorIndexManifest::new(scope.clone());
    let batch = VectorRepairBatch::new(
        &scope,
        vec![
            VectorRepairOperation::Index(id("mem-b")),
            VectorRepairOperation::Index(id("mem-a")),
        ],
    )
    .unwrap();
    manifest.apply(&batch).unwrap();
    assert_eq!(
        manifest.indexed_ids().collect::<Vec<_>>(),
        ["mem-a", "mem-b"]
    );
    let first_digest = manifest.digest().to_owned();
    assert_eq!(
        first_digest,
        "sha256:2bfaed6638c828cfc93648e6bb7d54128891b5ab62db32e9ba7bf1e3143c7cbb"
    );
    manifest
        .apply(
            &VectorRepairBatch::new(&scope, vec![VectorRepairOperation::Remove(id("mem-b"))])
                .unwrap(),
        )
        .unwrap();
    assert_ne!(first_digest, manifest.digest());
}

#[test]
fn failed_repair_does_not_partially_change_manifest() {
    let scope = VectorIndexScope::new("tenant-a", "embed-v1").unwrap();
    let other = VectorIndexScope::new("tenant-b", "embed-v1").unwrap();
    let mut manifest = VectorIndexManifest::new(scope.clone());
    let before = manifest.digest().to_owned();
    let batch =
        VectorRepairBatch::new(&other, vec![VectorRepairOperation::Index(id("mem-a"))]).unwrap();
    assert!(matches!(
        manifest.apply(&batch),
        Err(VectorManifestError::ScopeMismatch)
    ));
    assert_eq!(manifest.digest(), before);
}

#[test]
fn decoded_scope_identity_must_retain_constructor_invariants() {
    let scope = VectorIndexScope::new("tenant-a", "embed-v1").unwrap();
    let mut encoded = serde_json::to_value(&scope).unwrap();
    encoded["tenant_id"] = serde_json::Value::String("not canonical space".into());
    let decoded: VectorIndexScope = serde_json::from_value(encoded).unwrap();
    assert!(decoded.validate().is_err());
}
