use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use lakecat_core::governed_scan::{
    GovernedScanCatalogIdentity, GovernedScanProof, GovernedScanProofEvidence,
    governed_authorization_digest, governed_evidence_digest, governed_plan_digest,
    governed_policy_digest, governed_scan_digests,
};
use lakecat_core::{Namespace, TableIdent, TableName, WarehouseName};
use marciana_catalog::{governed_cognition_source, validate_governed_cognition_proof};
use serde_json::json;

fn proof() -> GovernedScanProof {
    let subject = "did:key:marciana-catalog-benchmark";
    GovernedScanProof::issue(GovernedScanProofEvidence {
        catalog_identity: GovernedScanCatalogIdentity::new("lakecat://benchmark")
            .expect("catalog identity"),
        table: TableIdent::new(
            WarehouseName::new("local").expect("warehouse"),
            "benchmark".parse::<Namespace>().expect("namespace"),
            TableName::new("memories").expect("table"),
        ),
        table_version: 7,
        snapshot_id: 42,
        plan_task_digest: governed_plan_digest(&[json!({"task": "opaque"})]).expect("plan digest"),
        principal_subject: subject.into(),
        purpose: "benchmark".into(),
        effective_projection: (0..128).map(|item| format!("column_{item:03}")).collect(),
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

fn benchmark_catalog_adapter(criterion: &mut Criterion) {
    let proof = proof();
    let catalog = proof.catalog_identity().clone();
    let digests = governed_scan_digests(&proof).expect("canonical LakeCat digests");
    let mut group = criterion.benchmark_group("catalog/governed_cognition");
    group.bench_function("validate_proof", |bencher| {
        bencher.iter(|| {
            black_box(
                validate_governed_cognition_proof(black_box(&catalog), black_box(&proof))
                    .expect("valid governed proof"),
            )
        });
    });
    for projection_count in [3, 128] {
        let projection = (0..projection_count)
            .map(|item| format!("column_{item:03}"))
            .collect::<Vec<_>>();
        group.bench_with_input(
            BenchmarkId::new("translate", projection_count),
            &projection_count,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(
                        governed_cognition_source(
                            black_box(&proof),
                            black_box(&digests),
                            black_box(&projection),
                        )
                        .expect("translate governed proof"),
                    )
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, benchmark_catalog_adapter);
criterion_main!(benches);
