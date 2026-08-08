use std::hint::black_box;

use chrono::{TimeDelta, TimeZone, Utc};
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use marciana_cognition::{
    AuditExportRecord, BackupManifest, ComponentHealth, ComponentState, CostRates, CostSample,
    EncryptionBoundary, EvaluationReport, HealthSnapshot, LineageInspection, MetricsSnapshot,
    OperationKind, OperationMetrics, OperationSample, Procedure, ProcedureRollout, QuotaLimits,
    SchemaDefinition, SchemaEdge, SchemaField, SchemaFieldKind, SchemaIdentity, SchemaRegistry,
    SchemaWindow, SloPolicy, TenantCostAccounting, TenantQuota,
};
use typesec_memory::{CognitionAuditEvidence, CognitionEffect, MemoryId};

const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn benchmark_accounting(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("cognition/accounting");
    let operation_sample = OperationSample {
        operation: OperationKind::Improve,
        allowed: true,
        latency_micros: 1_024,
    };
    let mut metrics = OperationMetrics::default();
    group.bench_function("metrics_record", |bencher| {
        bencher.iter(|| {
            metrics.record(black_box(operation_sample));
            black_box(&metrics);
        });
    });

    let rates = CostRates {
        source_record: 2,
        output_record: 3,
        input_byte: 5,
        output_byte: 7,
        compute_microsecond: 11,
    };
    let cost_sample = CostSample {
        operation: OperationKind::Improve,
        source_records: 8,
        output_records: 4,
        input_bytes: 16_384,
        output_bytes: 4_096,
        compute_microseconds: 32_768,
    };
    let mut accounting =
        TenantCostAccounting::new("tenant:benchmark".to_owned(), rates).expect("valid tenant");
    group.bench_function("cost_record", |bencher| {
        bencher.iter(|| {
            accounting.record(black_box(cost_sample));
            black_box(&accounting);
        });
    });

    let now = Utc
        .timestamp_opt(1_786_032_000, 0)
        .single()
        .expect("benchmark timestamp");
    let mut quota = TenantQuota::new(
        "tenant:benchmark".to_owned(),
        now,
        TimeDelta::days(1),
        QuotaLimits {
            operations: [u64::MAX; 5],
        },
    )
    .expect("valid quota");
    group.bench_function("quota_consume", |bencher| {
        bencher.iter(|| {
            quota
                .try_consume(
                    black_box(OperationKind::Context),
                    black_box(1),
                    black_box(now),
                )
                .expect("quota remains available");
            black_box(&quota);
        });
    });

    let policy = SloPolicy::new([10_000; 5], [500; 5]).expect("valid SLO policy");
    let snapshot = MetricsSnapshot {
        counts: [10_000; 5],
        denials: [10; 5],
        total_latency_micros: [1_000_000; 5],
        max_latency_micros: [1_000; 5],
    };
    group.bench_function("slo_evaluate", |bencher| {
        bencher.iter(|| black_box(black_box(policy).evaluate(black_box(snapshot))));
    });
    group.finish();
}

fn benchmark_operational_metadata(criterion: &mut Criterion) {
    let now = Utc
        .timestamp_opt(1_786_032_000, 0)
        .single()
        .expect("benchmark timestamp");
    let components = (0..32)
        .map(|item| ComponentHealth {
            name: format!("component-{item:02}"),
            revision: format!("revision-{item:02}"),
            state: ComponentState::Ready,
        })
        .collect();
    let health = HealthSnapshot::new(now, components).expect("valid health snapshot");
    let schema_window =
        SchemaWindow::new("querygraph-memory".to_owned(), 1, 10).expect("valid schema window");
    let backup = BackupManifest::new(
        "backup-benchmark".to_owned(),
        now,
        "querygraph-memory-v7".to_owned(),
        (0..32)
            .map(|item| {
                (
                    format!("component-{item:02}"),
                    format!("revision-{item:02}"),
                )
            })
            .collect(),
    )
    .expect("valid backup manifest");
    let boundary = EncryptionBoundary::new(
        "tenant:benchmark".to_owned(),
        "kms:benchmark-memory".to_owned(),
        42,
    )
    .expect("valid encryption boundary");

    let mut group = criterion.benchmark_group("cognition/operational_metadata");
    group.bench_function("health_ready_32", |bencher| {
        bencher.iter(|| black_box(black_box(&health).is_ready()));
    });
    group.bench_function("schema_window_accept", |bencher| {
        bencher.iter(|| {
            black_box(black_box(&schema_window).accepts(black_box("querygraph-memory-v7")))
        });
    });
    group.bench_function("backup_restore_window", |bencher| {
        bencher.iter(|| {
            black_box(black_box(&backup).validate_restore_window(black_box(&schema_window)))
                .expect("compatible schema");
        });
    });
    group.bench_function("encryption_boundary_digest", |bencher| {
        bencher.iter(|| black_box(black_box(&boundary).digest()));
    });
    group.bench_function("encryption_boundary_match", |bencher| {
        bencher.iter(|| {
            black_box(black_box(&boundary).matches(
                black_box("tenant:benchmark"),
                black_box("kms:benchmark-memory"),
                black_box(42),
            ))
            .expect("matching boundary");
        });
    });
    group.finish();
}

fn schema_parts(
    field_count: usize,
    edge_count: usize,
) -> (SchemaIdentity, Vec<SchemaField>, Vec<SchemaEdge>) {
    let identity = SchemaIdentity::new("benchmark".to_owned(), "semantic_model".to_owned(), 1)
        .expect("valid schema identity");
    let fields = (0..field_count)
        .rev()
        .map(|item| SchemaField {
            name: format!("field_{item:03}"),
            kind: if item == 0 {
                SchemaFieldKind::Identifier
            } else {
                SchemaFieldKind::Text
            },
        })
        .collect();
    let edges = (0..edge_count)
        .rev()
        .map(|item| SchemaEdge {
            name: format!("edge_{item:03}"),
            from_kind: format!("field_{:03}", item % field_count),
            to_kind: format!("field_{:03}", (item + 1) % field_count),
        })
        .collect();
    (identity, fields, edges)
}

fn benchmark_ontology(criterion: &mut Criterion) {
    let parts = schema_parts(128, 128);
    let schemas = (0..64)
        .rev()
        .map(|version| {
            let (mut identity, fields, edges) = schema_parts(8, 8);
            identity.version = version + 1;
            SchemaDefinition::new(identity, fields, edges).expect("valid schema definition")
        })
        .collect::<Vec<_>>();
    let registry = SchemaRegistry::new(schemas.clone()).expect("valid schema registry");
    let sought = SchemaIdentity::new("benchmark".to_owned(), "semantic_model".to_owned(), 64)
        .expect("valid schema identity");

    let mut group = criterion.benchmark_group("cognition/ontology");
    group.bench_function("definition_128_fields_128_edges", |bencher| {
        bencher.iter_batched(
            || parts.clone(),
            |(identity, fields, edges)| {
                black_box(
                    SchemaDefinition::new(identity, fields, edges)
                        .expect("valid schema definition"),
                )
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("registry_64", |bencher| {
        bencher.iter_batched(
            || schemas.clone(),
            |schemas| black_box(SchemaRegistry::new(schemas).expect("valid schema registry")),
            BatchSize::LargeInput,
        );
    });
    group.bench_function("resolve_64", |bencher| {
        bencher.iter(|| black_box(black_box(&registry).resolve(black_box(&sought))));
    });
    group.finish();
}

fn audit(affected_id_count: usize) -> CognitionAuditEvidence {
    let at = Utc
        .timestamp_opt(1_786_032_000, 0)
        .single()
        .expect("benchmark timestamp");
    CognitionAuditEvidence {
        schema_version: CognitionAuditEvidence::SCHEMA_VERSION,
        effect: CognitionEffect::NoChange,
        operation_id: "job:benchmark".to_owned(),
        subject: "agent:benchmark".to_owned(),
        space_id: "memory/user:benchmark/semantic".to_owned(),
        purpose: "benchmark".to_owned(),
        governed_source_scope: None,
        proposal_digest: DIGEST.to_owned(),
        binding_digest: DIGEST.to_owned(),
        source_manifest_digest: DIGEST.to_owned(),
        typedid_request_digest: DIGEST.to_owned(),
        governed_scan_digest: DIGEST.to_owned(),
        snapshot_digest: DIGEST.to_owned(),
        authorization_receipt_digest: DIGEST.to_owned(),
        policy_decision_id: "policy:benchmark".to_owned(),
        evidence_digest: DIGEST.to_owned(),
        affected_ids: (0..affected_id_count)
            .rev()
            .map(|item| MemoryId::from_string(format!("memory-{item:08}")))
            .collect(),
        authority_revalidated_at: at,
        prepared_at: at,
    }
}

fn benchmark_audit_and_lineage(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("cognition/audit_export");
    group.sample_size(20);
    for affected_id_count in [1, 4_096] {
        let audit = audit(affected_id_count);
        group.throughput(Throughput::Elements(
            u64::try_from(affected_id_count).expect("benchmark size fits u64"),
        ));
        group.bench_with_input(
            BenchmarkId::from_parameter(affected_id_count),
            &affected_id_count,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(
                        AuditExportRecord::from_audit(black_box(&audit))
                            .expect("valid audit export"),
                    )
                });
            },
        );
    }
    group.finish();

    let export = AuditExportRecord::from_audit(&audit(64)).expect("valid audit export");
    criterion.bench_function("cognition/lineage/project_9_nodes", |bencher| {
        bencher.iter(|| {
            black_box(LineageInspection::from_export(black_box(&export)).expect("valid lineage"))
        });
    });
}

fn approved_procedure() -> Procedure {
    let mut procedure = Procedure::propose(DIGEST.to_owned()).expect("valid procedure");
    let report =
        EvaluationReport::new(procedure.procedure_digest.clone(), DIGEST.to_owned(), 8_000)
            .expect("valid evaluation");
    procedure
        .record_evaluation(&report)
        .expect("record evaluation");
    procedure.approve().expect("approve procedure");
    procedure
}

fn benchmark_rollout(criterion: &mut Criterion) {
    let procedure = approved_procedure();
    let rollout =
        ProcedureRollout::propose(&procedure, DIGEST.to_owned(), 2_500, 90).expect("valid rollout");
    criterion.bench_function("cognition/procedure_rollout/validate", |bencher| {
        bencher.iter(|| {
            black_box(black_box(&rollout).validate()).expect("valid rollout identity");
        });
    });
}

criterion_group!(
    benches,
    benchmark_accounting,
    benchmark_operational_metadata,
    benchmark_ontology,
    benchmark_audit_and_lineage,
    benchmark_rollout
);
criterion_main!(benches);
