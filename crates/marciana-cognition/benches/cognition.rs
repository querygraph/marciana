use std::hint::black_box;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use marciana_cognition::{
    EvaluationReport, FeedbackDataset, FeedbackRecord, LatencySamples, Observation, OssieAdapter,
    WorkingSet, WorkingSetSlot, WorkingSetSource,
};
use querygraph_memory::context::{ContextRecipe, ContextView};
use serde_json::json;
use typesec_memory::MemoryId;

const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn benchmark_latency_snapshot(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("cognition/latency_snapshot");
    for sample_count in [1_024, 100_000] {
        let mut samples = LatencySamples::default();
        for item in 0..sample_count {
            let latency =
                u64::try_from((item * 2_654_435_761_usize) % 100_000).expect("latency fits u64");
            samples.record(latency).expect("within sample bound");
        }
        group.throughput(Throughput::Elements(
            u64::try_from(sample_count).expect("benchmark size fits u64"),
        ));
        group.bench_with_input(
            BenchmarkId::from_parameter(sample_count),
            &sample_count,
            |bencher, _| bencher.iter(|| black_box(samples.snapshot())),
        );
    }
    group.finish();
}

fn feedback_records(count: usize) -> Vec<FeedbackRecord> {
    (0..count)
        .rev()
        .map(|item| FeedbackRecord {
            trajectory_digest: format!("sha256:{item:064x}"),
            outcome_basis_points: u16::try_from(item % 10_001).expect("score fits u16"),
            recorded_at: Utc
                .timestamp_opt(
                    1_786_032_000 + i64::try_from(item).expect("timestamp offset fits i64"),
                    0,
                )
                .single()
                .expect("benchmark timestamp"),
        })
        .collect()
}

fn benchmark_feedback_dataset(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("cognition/feedback_dataset");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(5));
    for record_count in [1_024, 10_000] {
        let records = feedback_records(record_count);
        group.throughput(Throughput::Elements(
            u64::try_from(record_count).expect("benchmark size fits u64"),
        ));
        group.bench_with_input(
            BenchmarkId::from_parameter(record_count),
            &record_count,
            |bencher, _| {
                bencher.iter_batched(
                    || records.clone(),
                    |records| {
                        black_box(FeedbackDataset::new(records).expect("valid feedback fixture"))
                    },
                    BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

fn observation_evidence(count: usize) -> Vec<String> {
    (0..count)
        .rev()
        .map(|item| format!("sha256:{item:064x}"))
        .collect()
}

fn scrambled_observation_evidence(count: usize) -> Vec<String> {
    (0..count)
        .map(|position| {
            let item = position.wrapping_mul(2_654_435_769) % count;
            format!("sha256:{item:064x}")
        })
        .collect()
}

fn benchmark_learning_artifacts(criterion: &mut Criterion) {
    let at = Utc
        .timestamp_opt(1_786_032_000, 0)
        .single()
        .expect("benchmark timestamp");
    let mut group = criterion.benchmark_group("cognition/learning");
    for evidence_count in [1, 256] {
        let evidence = observation_evidence(evidence_count);
        let scrambled_evidence = scrambled_observation_evidence(evidence_count);
        group.throughput(Throughput::Elements(
            u64::try_from(evidence_count).expect("benchmark size fits u64"),
        ));
        group.bench_with_input(
            BenchmarkId::new("observation_propose", evidence_count),
            &evidence_count,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(
                        Observation::propose(
                            black_box(&evidence),
                            u32::try_from(evidence_count).expect("evidence count fits u32"),
                            8_500,
                            at,
                        )
                        .expect("valid observation"),
                    )
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("observation_propose_scrambled", evidence_count),
            &evidence_count,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(
                        Observation::propose(
                            black_box(&scrambled_evidence),
                            u32::try_from(evidence_count).expect("evidence count fits u32"),
                            8_500,
                            at,
                        )
                        .expect("valid observation"),
                    )
                });
            },
        );
    }
    group.throughput(Throughput::Elements(1));
    group.bench_function("evaluation_report", |bencher| {
        bencher.iter_batched(
            || (DIGEST.to_owned(), DIGEST.to_owned()),
            |(procedure_digest, dataset_digest)| {
                black_box(
                    EvaluationReport::new(procedure_digest, dataset_digest, 8_000)
                        .expect("valid evaluation report"),
                )
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn working_set_slots(count: usize) -> Vec<WorkingSetSlot> {
    (0..count)
        .rev()
        .map(|item| WorkingSetSlot {
            memory_id: MemoryId::from_string(format!("memory-{item:08}")),
        })
        .collect()
}

fn benchmark_working_set(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("cognition/working_set");
    for slot_count in [1, 16, 64] {
        let slots = working_set_slots(slot_count);
        group.bench_with_input(
            BenchmarkId::new("propose", slot_count),
            &slot_count,
            |bencher, _| {
                bencher.iter_batched(
                    || slots.clone(),
                    |slots| {
                        black_box(
                            WorkingSet::propose(
                                "memory/user:benchmark/semantic".to_owned(),
                                DIGEST.to_owned(),
                                ContextView::Assertions,
                                ContextRecipe::Ranked,
                                64_000,
                                slots,
                                WorkingSetSource::Operator,
                            )
                            .expect("valid working set"),
                        )
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        let working_set = WorkingSet::propose(
            "memory/user:benchmark/semantic".to_owned(),
            DIGEST.to_owned(),
            ContextView::Assertions,
            ContextRecipe::Ranked,
            64_000,
            slots,
            WorkingSetSource::Operator,
        )
        .expect("valid working set");
        group.bench_with_input(
            BenchmarkId::new("validate", slot_count),
            &slot_count,
            |bencher, _| {
                bencher.iter(|| {
                    working_set.validate().expect("working set remains valid");
                    black_box(&working_set);
                });
            },
        );

        let mut active_working_set = working_set.clone();
        active_working_set.approve().expect("working set approval");
        active_working_set
            .activate()
            .expect("working set activation");
        let as_of = Utc
            .timestamp_opt(1_786_032_000, 0)
            .single()
            .expect("benchmark timestamp");
        group.bench_with_input(
            BenchmarkId::new("recall_intent", slot_count),
            &slot_count,
            |bencher, _| {
                bencher.iter_batched(
                    || DIGEST.to_owned(),
                    |query_digest| {
                        black_box(
                            active_working_set
                                .recall_intent(query_digest, as_of)
                                .expect("valid recall intent"),
                        )
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn ossie_model() -> String {
    let metrics = (0..64)
        .map(|item| {
            json!({
                "name": format!("metric_{item:03}"),
                "expression": format!("sum(value_{item:03})")
            })
        })
        .collect::<Vec<_>>();
    let dimensions = (0..64)
        .map(|item| {
            json!({
                "name": format!("dimension_{item:03}"),
                "role": if item == 0 { "identifier" } else { "attribute" }
            })
        })
        .collect::<Vec<_>>();
    let relationships = (0..64)
        .map(|item| {
            json!({
                "name": format!("relationship_{item:03}"),
                "from": format!("dimension_{item:03}"),
                "to": format!("dimension_{:03}", (item + 1) % 64)
            })
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&json!({
        "namespace": "benchmark",
        "name": "semantic_model",
        "version": 1,
        "metrics": metrics,
        "dimensions": dimensions,
        "relationships": relationships
    }))
    .expect("serialize Ossie fixture")
}

fn benchmark_ossie(criterion: &mut Criterion) {
    let model = ossie_model();
    let binding = OssieAdapter::import_json("lakecat:benchmark/model/v1", &model)
        .expect("valid Ossie fixture");
    let dimensions = (0..64)
        .rev()
        .map(|item| format!("dimension_{item:03}"))
        .collect::<Vec<_>>();
    let mut group = criterion.benchmark_group("cognition/ossie");
    group.bench_function("import_128_fields_64_edges", |bencher| {
        bencher.iter(|| {
            black_box(
                OssieAdapter::import_json(
                    black_box("lakecat:benchmark/model/v1"),
                    black_box(&model),
                )
                .expect("valid Ossie fixture"),
            )
        });
    });
    group.bench_function("plan_64_dimensions", |bencher| {
        bencher.iter_batched(
            || dimensions.clone(),
            |dimensions| {
                black_box(
                    OssieAdapter::plan_query(&binding, "metric_000", dimensions)
                        .expect("valid Ossie query"),
                )
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(
    benches,
    benchmark_latency_snapshot,
    benchmark_feedback_dataset,
    benchmark_learning_artifacts,
    benchmark_working_set,
    benchmark_ossie
);
criterion_main!(benches);
