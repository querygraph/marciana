use chrono::{TimeZone, Utc};
use marciana_cognition::{AuditExportError, AuditExportRecord};
use sha2::Digest;
use typesec_memory::{CognitionAuditEvidence, CognitionEffect, MemoryId};

fn digest(value: &str) -> String {
    format!("sha256:{:x}", sha2::Sha256::digest(value.as_bytes()))
}

fn audit() -> CognitionAuditEvidence {
    let at = Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, 0).unwrap();
    CognitionAuditEvidence {
        schema_version: CognitionAuditEvidence::SCHEMA_VERSION,
        effect: CognitionEffect::NoChange,
        operation_id: "job:coffee".into(),
        subject: "agent:coffee".into(),
        space_id: "memory/user:alice/semantic".into(),
        purpose: "research".into(),
        governed_source_scope: None,
        proposal_digest: digest("proposal"),
        binding_digest: digest("binding"),
        source_manifest_digest: digest("manifest"),
        typedid_request_digest: digest("typedid"),
        governed_scan_digest: digest("scan"),
        snapshot_digest: digest("snapshot"),
        authorization_receipt_digest: digest("receipt"),
        policy_decision_id: "policy:coffee".into(),
        evidence_digest: digest("evidence"),
        affected_ids: vec![MemoryId::from_string("memory-1")],
        authority_revalidated_at: at,
        prepared_at: at,
    }
}

#[test]
fn audit_export_is_redacted_and_order_stable() {
    let mut audit = audit();
    let export = AuditExportRecord::from_audit(&audit).expect("export");
    assert_eq!(export.schema_version, 1);
    assert_ne!(export.subject_digest, audit.subject);
    assert_ne!(export.purpose_digest, audit.purpose);
    assert_eq!(export.affected_id_count, 1);
    audit.affected_ids.reverse();
    let reversed = AuditExportRecord::from_audit(&audit).expect("reversed export");
    assert_eq!(export.affected_ids_digest, reversed.affected_ids_digest);
}

#[test]
fn audit_export_rejects_unknown_schema() {
    let mut audit = audit();
    audit.schema_version += 1;
    assert_eq!(
        AuditExportRecord::from_audit(&audit).expect_err("schema rejection"),
        AuditExportError::UnsupportedSchema
    );
}
