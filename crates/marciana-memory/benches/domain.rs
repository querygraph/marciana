use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use grust_core::prelude::{Edge, NodeId, Value};
use marciana_ledger::{Assertion, AssertionId, AssertionLineage, Confidence, TemporalInterval};
use querygraph_memory::analytics::{contradiction_plan, dedup_plan};
use querygraph_memory::assertion_projection::{project_assertion, project_legacy_relation};
use querygraph_memory::context::{
    ContextCandidate, ContextPlan, ContextRecipe, ContextView, RecallIntent, plan_context,
};
use querygraph_memory::{
    ContextEvaluationCase, ContextEvaluationCorpus, ContextEvaluationReport, SessionMetadata,
    VectorIndexManifest, VectorIndexScope, VectorRepairBatch, VectorRepairOperation,
};
use typesec_memory::{
    Label, MemoryContent, MemoryId, MemoryKind, Provenance, RecalledMemory, StoredRecord,
};

const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn recalled_memory(item: usize, text: String) -> RecalledMemory {
    RecalledMemory {
        id: MemoryId::from_string(format!("memory-{item:08}")),
        kind: MemoryKind::Semantic,
        label: Label::Internal,
        content: MemoryContent::text(text),
        entities: Vec::new(),
        provenance: Provenance::Operator,
        valid_from: Utc
            .timestamp_opt(
                1_786_032_000 + i64::try_from(item).expect("timestamp offset fits i64"),
                0,
            )
            .single()
            .expect("benchmark timestamp"),
    }
}

fn duplicate_memories(count: usize) -> Vec<RecalledMemory> {
    (0..count)
        .map(|item| recalled_memory(item, format!("subject-{:04} has value", item / 4)))
        .collect()
}

fn contradiction_memories(count: usize) -> Vec<RecalledMemory> {
    (0..count)
        .map(|item| {
            recalled_memory(
                item,
                format!("subject-{:04} has value-{}", item / 8, item % 8),
            )
        })
        .collect()
}

fn benchmark_analytics(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("memory/analytics");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(5));
    for count in [1_024, 10_000] {
        let memories = duplicate_memories(count);
        group.throughput(Throughput::Elements(
            u64::try_from(count).expect("benchmark size fits u64"),
        ));
        group.bench_with_input(
            BenchmarkId::new("deduplicate", count),
            &count,
            |bencher, _| {
                bencher.iter(|| black_box(dedup_plan(black_box(&memories))));
            },
        );
    }
    for count in [256, 1_024] {
        let memories = contradiction_memories(count);
        group.throughput(Throughput::Elements(
            u64::try_from(count).expect("benchmark size fits u64"),
        ));
        group.bench_with_input(
            BenchmarkId::new("reconcile", count),
            &count,
            |bencher, _| {
                bencher.iter(|| black_box(contradiction_plan(black_box(&memories))));
            },
        );
    }
    group.finish();
}

fn recall_intent(pinned_count: usize) -> RecallIntent {
    RecallIntent {
        query_digest: DIGEST.to_owned(),
        working_set_digest: Some(DIGEST.to_owned()),
        pinned_memory_ids: (0..pinned_count)
            .map(|item| MemoryId::from_string(format!("memory-{item:08}")))
            .collect(),
        view: ContextView::Assertions,
        recipe: ContextRecipe::Ranked,
        as_of: Utc
            .timestamp_opt(1_786_032_000, 0)
            .single()
            .expect("benchmark timestamp"),
        token_budget: 64_000,
    }
}

fn benchmark_session(criterion: &mut Criterion) {
    let session = SessionMetadata::new(
        "session-benchmark".to_owned(),
        "memory/user:benchmark/semantic".to_owned(),
        DIGEST.to_owned(),
    )
    .expect("valid session");
    let mut group = criterion.benchmark_group("memory/session");
    for pinned_count in [0, 64] {
        let intent = recall_intent(pinned_count);
        group.bench_with_input(
            BenchmarkId::new("bind_intent", pinned_count),
            &pinned_count,
            |bencher, _| {
                bencher.iter_batched(
                    || intent.clone(),
                    |intent| black_box(session.bind_intent(intent).expect("valid bound intent")),
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn evaluation_plan(count: usize) -> ContextPlan {
    plan_context(
        recall_intent(0),
        (0..count)
            .map(|item| ContextCandidate {
                id: MemoryId::from_string(format!("memory-{item:08}")),
                score_basis_points: u32::try_from(count - item).expect("score fits u32"),
                estimated_tokens: 1,
                reason_digest: DIGEST.to_owned(),
            })
            .collect(),
    )
    .expect("valid context plan")
}

fn evaluation_case(count: usize, case_digest: String) -> ContextEvaluationCase {
    ContextEvaluationCase::new(
        case_digest,
        (0..count / 2)
            .map(|item| MemoryId::from_string(format!("memory-{item:08}")))
            .collect(),
        Vec::new(),
        64_000,
    )
    .expect("valid evaluation case")
}

fn benchmark_evaluation(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("memory/evaluation");
    for count in [64, 1_000] {
        let plan = evaluation_plan(count);
        let case = evaluation_case(count, DIGEST.to_owned());
        group.throughput(Throughput::Elements(
            u64::try_from(count).expect("benchmark size fits u64"),
        ));
        group.bench_with_input(BenchmarkId::new("report", count), &count, |bencher, _| {
            bencher.iter(|| {
                black_box(
                    ContextEvaluationReport::evaluate(black_box(&case), black_box(&plan))
                        .expect("valid evaluation report"),
                )
            });
        });
    }

    let case_count = 256;
    let cases = (0..case_count)
        .map(|item| evaluation_case(2, format!("sha256:{item:064x}")))
        .collect();
    let corpus = ContextEvaluationCorpus::new(cases).expect("valid evaluation corpus");
    let plans = (0..case_count)
        .map(|_| evaluation_plan(2))
        .collect::<Vec<_>>();
    group.throughput(Throughput::Elements(case_count));
    group.bench_function("corpus_256", |bencher| {
        bencher.iter(|| {
            black_box(
                black_box(&corpus)
                    .evaluate(black_box(&plans))
                    .expect("valid corpus evaluation"),
            )
        });
    });
    group.finish();
}

fn seeded_manifest(size: usize) -> (VectorIndexManifest, VectorIndexScope) {
    let scope = VectorIndexScope::new("tenant-benchmark", "embedding-benchmark")
        .expect("valid vector scope");
    let batch = VectorRepairBatch::new(
        &scope,
        (0..size)
            .map(|item| {
                VectorRepairOperation::Index(MemoryId::from_string(format!("memory-{item:08}")))
            })
            .collect(),
    )
    .expect("valid seed repair");
    let mut manifest = VectorIndexManifest::new(scope.clone());
    manifest.apply(&batch).expect("seed vector manifest");
    (manifest, scope)
}

fn benchmark_vector_manifest(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("memory/vector_manifest");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(5));
    for size in [1_024, 10_000] {
        let (manifest, scope) = seeded_manifest(size);
        let repair = VectorRepairBatch::new(
            &scope,
            (0..8)
                .map(|item| {
                    VectorRepairOperation::Remove(MemoryId::from_string(format!(
                        "memory-{item:08}"
                    )))
                })
                .chain((size..size + 8).map(|item| {
                    VectorRepairOperation::Index(MemoryId::from_string(format!("memory-{item:08}")))
                }))
                .collect(),
        )
        .expect("valid repair batch");
        group.throughput(Throughput::Elements(
            u64::try_from(size).expect("benchmark size fits u64"),
        ));
        group.bench_with_input(BenchmarkId::new("validate", size), &size, |bencher, _| {
            bencher.iter(|| {
                black_box(&manifest)
                    .validate()
                    .expect("valid vector manifest");
                black_box(&manifest)
            });
        });
        group.bench_with_input(BenchmarkId::new("apply_16", size), &size, |bencher, _| {
            bencher.iter_batched(
                || manifest.clone(),
                |mut manifest| {
                    manifest.apply(&repair).expect("apply vector repair");
                    black_box(manifest)
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn assertion() -> Assertion {
    let at = Utc
        .timestamp_opt(1_786_032_000, 0)
        .single()
        .expect("benchmark timestamp");
    Assertion::new(
        AssertionId::new(),
        "account:benchmark",
        "locatedIn",
        "place:benchmark",
        Confidence::from_basis_points(8_500).expect("valid confidence"),
        at,
        at,
        TemporalInterval::new(at, None).expect("valid interval"),
        AssertionLineage::new("episode:1", "memory-1", "document-v1", "assertion-v1")
            .expect("valid lineage"),
    )
    .expect("valid assertion")
}

fn legacy_record() -> StoredRecord {
    serde_json::from_value(serde_json::json!({
        "id": "memory-1",
        "space_id": "memory/user:benchmark/semantic",
        "kind": "semantic",
        "label": "internal",
        "quarantined": false,
        "entities": [],
        "provenance": { "source": "operator" },
        "observed_at": "2026-08-08T12:00:00Z",
        "valid_from": "2026-08-08T12:00:00Z",
        "invalid_at": null,
        "expires_at": null,
        "purposes": ["benchmark"],
        "content": { "text": "benchmark" }
    }))
    .expect("valid legacy record")
}

fn legacy_edge() -> Edge {
    Edge::new(
        "RELATES",
        NodeId::from("ent:account:benchmark"),
        NodeId::from("ent:place:benchmark"),
        BTreeMap::from([
            ("rel".to_owned(), Value::String("locatedIn".to_owned())),
            ("fact_id".to_owned(), Value::String("memory-1".to_owned())),
        ]),
    )
}

fn benchmark_assertion_projection(criterion: &mut Criterion) {
    let assertion = assertion();
    let edge = legacy_edge();
    let record = legacy_record();
    let mut group = criterion.benchmark_group("memory/assertion_projection");
    group.bench_function("current", |bencher| {
        bencher.iter(|| {
            black_box(project_assertion(black_box(&assertion)).expect("valid projection"))
        });
    });
    group.bench_function("legacy", |bencher| {
        bencher.iter(|| {
            black_box(
                project_legacy_relation(black_box(&edge), black_box(&record))
                    .expect("valid legacy projection"),
            )
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    benchmark_analytics,
    benchmark_session,
    benchmark_evaluation,
    benchmark_vector_manifest,
    benchmark_assertion_projection
);
criterion_main!(benches);
