use chrono::Utc;
use marciana_cognition::{BackupManifest, RestoreError, SchemaWindow, SchemaWindowError};

#[test]
fn schema_window_accepts_only_canonical_versions_inclusive() {
    let window = SchemaWindow::new("querygraph-memory".into(), 1, 2).expect("window");
    assert_eq!(window.family(), "querygraph-memory");
    assert_eq!(window.minimum(), 1);
    assert_eq!(window.maximum(), 2);
    assert!(window.accepts("querygraph-memory-v1"));
    assert!(window.accepts("querygraph-memory-v2"));
    assert!(!window.accepts("querygraph-memory-v3"));
    assert!(!window.accepts("querygraph-memory-v1-extra"));
    assert!(!window.accepts("querygraph-memory-v01"));
    assert!(!window.accepts("querygraph-memory-v+1"));
    assert!(!window.accepts("other-v1"));
}

#[test]
fn schema_window_rejects_invalid_ranges_without_echoing_values() {
    assert_eq!(
        SchemaWindow::new("querygraph-memory".into(), 0, 1).expect_err("range"),
        SchemaWindowError::InvalidRange
    );
    assert_eq!(
        SchemaWindow::new("querygraph memory".into(), 1, 2).expect_err("family"),
        SchemaWindowError::InvalidFamily
    );
}

#[test]
fn backup_restore_can_use_a_supported_schema_window() {
    let manifest = BackupManifest::new(
        "backup-1".into(),
        Utc::now(),
        "querygraph-memory-v1".into(),
        vec![("marciana".into(), "revision".into())],
    )
    .expect("manifest");
    let window = SchemaWindow::new("querygraph-memory".into(), 1, 2).expect("window");
    assert!(manifest.validate_restore_window(&window).is_ok());
    let unsupported = SchemaWindow::new("querygraph-memory".into(), 2, 3).expect("window");
    assert_eq!(
        manifest.validate_restore_window(&unsupported),
        Err(RestoreError::IncompatibleSchema)
    );
}
