use marciana_cognition::{EncryptionBoundary, EncryptionBoundaryError};

#[test]
fn boundary_is_digest_only_and_matches_exact_scope() {
    let boundary = EncryptionBoundary::new("tenant:coffee".into(), "kms:coffee-memory".into(), 4)
        .expect("boundary");
    let digest = boundary.digest();
    assert!(digest.starts_with("sha256:"));
    assert!(!digest.contains("coffee-memory"));
    boundary
        .matches("tenant:coffee", "kms:coffee-memory", 4)
        .expect("exact boundary");
    for candidate in [
        ("tenant:other", "kms:coffee-memory", 4),
        ("tenant:coffee", "kms:other", 4),
        ("tenant:coffee", "kms:coffee-memory", 3),
    ] {
        assert_eq!(
            boundary.matches(candidate.0, candidate.1, candidate.2),
            Err(EncryptionBoundaryError::Mismatch)
        );
    }
}

#[test]
fn rotation_is_monotonic_and_changes_the_receipt_identity() {
    let boundary =
        EncryptionBoundary::new("tenant:coffee".into(), "kms:key-a".into(), 1).expect("boundary");
    let rotated = boundary.rotate("kms:key-b".into()).expect("rotation");
    assert_eq!(rotated.tenant_id(), "tenant:coffee");
    assert_eq!(rotated.key_revision(), 2);
    assert_ne!(boundary.digest(), rotated.digest());
    assert_eq!(
        boundary.matches("tenant:coffee", "kms:key-b", 2),
        Err(EncryptionBoundaryError::Mismatch)
    );
}

#[test]
fn boundary_rejects_invalid_identities_without_echoing_values() {
    assert_eq!(
        EncryptionBoundary::new("tenant coffee".into(), "kms:key".into(), 1).expect_err("tenant"),
        EncryptionBoundaryError::InvalidTenant
    );
    assert_eq!(
        EncryptionBoundary::new("tenant:coffee".into(), "kms/key with text".into(), 1)
            .expect_err("key"),
        EncryptionBoundaryError::InvalidKeyId
    );
    assert_eq!(
        EncryptionBoundary::new("tenant:coffee".into(), "kms:key".into(), 0).expect_err("revision"),
        EncryptionBoundaryError::InvalidRevision
    );
}
