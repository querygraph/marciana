use lakecat_core::governed_scan::{
    GovernedScanCatalogIdentity, GovernedScanProof, GovernedScanProofEvidence,
    governed_authorization_digest, governed_evidence_digest, governed_plan_digest,
    governed_policy_digest, governed_scan_digests,
};
use lakecat_core::{Namespace, TableIdent, TableName, WarehouseName};
use marciana_catalog::{
    LakeCatCognitionSourceError, governed_cognition_source, validate_governed_cognition_proof,
};
use serde_json::json;

#[test]
fn translation_preserves_only_the_canonical_cognition_identity() {
    let proof = proof();
    let digests = governed_scan_digests(&proof).expect("canonical LakeCat digests");
    let projection = vec![
        "memory_id".into(),
        "memory_text".into(),
        "valid_from".into(),
    ];

    let source = governed_cognition_source(&proof, &digests, &projection)
        .expect("translate a verified governed proof");

    assert_eq!(source.snapshot_digest, digests.snapshot_digest());
    assert_eq!(source.governed_scan_digest, proof.grant_id());
    assert_eq!(source.catalog, proof.catalog_identity().as_str());
    assert_eq!(source.namespace, proof.table().namespace.to_string());
    assert_eq!(source.table, proof.table().name.to_string());
    assert_eq!(source.snapshot_id, proof.snapshot_id());
    assert_eq!(source.effective_projection, projection);
    assert!(source.digest().is_ok());
}

#[test]
fn validation_rejects_a_proof_from_another_catalog() {
    let proof = proof();
    let configured = GovernedScanCatalogIdentity::new("lakecat://another-catalog")
        .expect("configured catalog identity");

    assert!(matches!(
        validate_governed_cognition_proof(&configured, &proof),
        Err(LakeCatCognitionSourceError::CatalogMismatch)
    ));
}

fn proof() -> GovernedScanProof {
    let subject = "did:key:marciana-catalog-test";
    GovernedScanProof::issue(GovernedScanProofEvidence {
        catalog_identity: GovernedScanCatalogIdentity::new("lakecat://qglake")
            .expect("catalog identity"),
        table: TableIdent::new(
            WarehouseName::new("local").expect("warehouse"),
            "research".parse::<Namespace>().expect("namespace"),
            TableName::new("findings").expect("table"),
        ),
        table_version: 7,
        snapshot_id: 42,
        plan_task_digest: governed_plan_digest(&[json!({"task": "opaque"})]).expect("plan digest"),
        principal_subject: subject.into(),
        purpose: "research".into(),
        effective_projection: vec![
            "memory_id".into(),
            "memory_text".into(),
            "valid_from".into(),
        ],
        identity_context_digest: governed_evidence_digest(
            "lakecat.verified-identity-context.digest.v1",
            &json!({"principal": {"subject": subject}, "attestation-state": "verified"}),
        )
        .expect("identity digest"),
        authorization_receipt_digest: governed_authorization_digest(&json!({"allowed": true}))
            .expect("authorization digest"),
        policy_decision_digest: governed_policy_digest(&json!({"policy": "issue-time"}))
            .expect("policy digest"),
    })
    .expect("governed proof")
}
