use chrono::{TimeZone, Utc};
use marciana_cognition::{AuditExportRecord, LineageInspection, LineageNodeKind};
use typesec_memory::CognitionEffect;

fn export() -> AuditExportRecord {
    let at = Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, 0).unwrap();
    let digest = |name: &str| format!("sha256:{name}");
    AuditExportRecord {
        schema_version: 1,
        operation_id: "job:coffee".into(),
        space_id: "memory/user:alice/semantic".into(),
        effect: CognitionEffect::NoChange,
        subject_digest: digest("subject"),
        purpose_digest: digest("purpose"),
        governed_source_scope_digest: None,
        proposal_digest: digest("proposal"),
        binding_digest: digest("binding"),
        source_manifest_digest: digest("manifest"),
        typedid_request_digest: digest("typedid"),
        governed_scan_digest: digest("scan"),
        snapshot_digest: digest("snapshot"),
        authorization_receipt_digest: digest("receipt"),
        policy_decision_digest: digest("policy"),
        evidence_digest: digest("evidence"),
        affected_id_count: 2,
        affected_ids_digest: digest("affected"),
        authority_revalidated_at: at,
        prepared_at: at,
    }
}

#[test]
fn lineage_projection_has_fixed_order_and_no_plaintext_payload() {
    let inspection = LineageInspection::from_export(&export()).expect("lineage");
    assert_eq!(inspection.schema_version, "marciana-lineage-v1");
    assert_eq!(inspection.nodes.len(), 9);
    assert_eq!(inspection.edges.len(), 8);
    assert_eq!(inspection.nodes[0].kind, LineageNodeKind::Proposal);
    assert_eq!(inspection.nodes[8].kind, LineageNodeKind::Evidence);
    assert!(
        inspection
            .nodes
            .iter()
            .all(|node| node.digest.starts_with("sha256:"))
    );
    assert!(
        !inspection
            .nodes
            .iter()
            .any(|node| node.digest.contains("coffee"))
    );
}
