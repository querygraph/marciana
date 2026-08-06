use chrono::{TimeZone, Utc};

use super::{BackupManifest, RestoreError};

#[test]
fn restore_requires_matching_content_free_schema_metadata() {
    let manifest = BackupManifest::new(
        "backup-1".into(),
        Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, 0).unwrap(),
        "querygraph-memory-v1".into(),
        vec![("typesec".into(), "14bd5427".into())],
    )
    .unwrap();
    assert!(manifest.validate_restore("querygraph-memory-v1").is_ok());
    assert_eq!(
        manifest.validate_restore("querygraph-memory-v2"),
        Err(RestoreError::IncompatibleSchema)
    );
}

#[test]
fn invalid_manifest_is_rejected_before_restore() {
    assert!(BackupManifest::new(String::new(), Utc::now(), "schema".into(), vec![]).is_err());
}
