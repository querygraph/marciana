use std::hint::black_box;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use grust_memory::MemoryGraphStore;
use querygraph_memory::cognition::GovernedLakeCatSnapshot;
use querygraph_memory::context::{
    ContextCandidate, ContextRecipe, ContextView, RecallIntent, plan_context,
};
use querygraph_memory::{Embedder, GraphStoreMemoryStore, VectorIndex};
use serde_json::json;
use typesec_memory::{
    IndexError, Label, MemoryId, MemoryStore, SemanticIndex, StoreBatchOp, StoreQuery, StoredRecord,
};

const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const VECTOR_DIMENSIONS: usize = 128;

struct FixedEmbedder;

impl Embedder for FixedEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>, IndexError> {
        let mut state = text
            .strip_prefix("record-")
            .and_then(|suffix| suffix.parse::<u64>().ok())
            .unwrap_or(0x9e37_79b9_7f4a_7c15);
        Ok((0..VECTOR_DIMENSIONS)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let component =
                    u16::try_from((state >> 16) & 0xffff).expect("component is masked to u16");
                f32::from(component) / f32::from(u16::MAX)
            })
            .collect())
    }

    fn is_local(&self) -> bool {
        true
    }
}

fn vector_index(size: usize) -> VectorIndex<FixedEmbedder> {
    let index = VectorIndex::new(FixedEmbedder);
    for item in 0..size {
        let id = MemoryId::from_string(format!("memory-{item:08}"));
        index
            .index(&id, Label::Internal, &format!("record-{item}"))
            .expect("fixed embedder cannot fail");
        index.note_entities(
            &id,
            [
                format!("entity-{:04}", item % 2_048),
                format!("group-{:02}", item % 32),
            ],
        );
    }
    index
}

fn benchmark_vector_search(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("memory/vector_search");
    for size in [1_024, 16_384, 65_536] {
        group.throughput(Throughput::Elements(
            u64::try_from(size).expect("benchmark size fits u64"),
        ));
        let index = vector_index(size);
        let boost = (0..32)
            .map(|item| format!("entity-{item:04}"))
            .collect::<Vec<_>>();
        group.bench_with_input(BenchmarkId::new("top_10", size), &size, |bencher, _| {
            bencher.iter(|| {
                black_box(
                    index
                        .search(black_box("benchmark-query"), black_box(10))
                        .expect("fixed embedder cannot fail"),
                )
            });
        });
        group.bench_with_input(
            BenchmarkId::new("hybrid_top_10", size),
            &size,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(
                        index
                            .search_hybrid(
                                black_box("benchmark-query"),
                                black_box(10),
                                black_box(&boost),
                            )
                            .expect("fixed embedder cannot fail"),
                    )
                });
            },
        );
    }
    group.finish();
}

fn context_fixture(size: usize, pinned_count: usize) -> (RecallIntent, Vec<ContextCandidate>) {
    let candidates = (0..size)
        .map(|item| ContextCandidate {
            id: MemoryId::from_string(format!("memory-{item:08}")),
            score_basis_points: u32::try_from(item % 10_001).expect("score fits u32"),
            estimated_tokens: 1,
            reason_digest: DIGEST.to_owned(),
        })
        .collect::<Vec<_>>();
    let pinned_memory_ids = candidates
        .iter()
        .rev()
        .take(pinned_count)
        .map(|candidate| candidate.id.clone())
        .collect();
    (
        RecallIntent {
            query_digest: DIGEST.to_owned(),
            working_set_digest: (pinned_count > 0).then(|| DIGEST.to_owned()),
            pinned_memory_ids,
            view: ContextView::Assertions,
            recipe: ContextRecipe::Ranked,
            as_of: Utc
                .timestamp_opt(1_786_032_000, 0)
                .single()
                .expect("benchmark timestamp"),
            token_budget: 64_000,
        },
        candidates,
    )
}

fn benchmark_context_planning(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("memory/context_plan");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(5));
    for (size, pinned_count) in [(1_024, 0), (1_024, 64), (100_000, 0), (100_000, 64)] {
        let (intent, candidates) = context_fixture(size, pinned_count);
        group.throughput(Throughput::Elements(
            u64::try_from(size).expect("benchmark size fits u64"),
        ));
        group.bench_with_input(
            BenchmarkId::new(format!("pinned_{pinned_count}"), size),
            &(size, pinned_count),
            |bencher, _| {
                bencher.iter_batched(
                    || (intent.clone(), candidates.clone()),
                    |(intent, candidates)| {
                        black_box(plan_context(intent, candidates).expect("valid context fixture"))
                    },
                    BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

fn governed_snapshot(projection_count: usize) -> GovernedLakeCatSnapshot {
    GovernedLakeCatSnapshot {
        catalog: "lakecat://benchmark".to_owned(),
        namespace: "benchmark".to_owned(),
        table: "memories".to_owned(),
        snapshot_id: 42,
        governed_scan_digest: DIGEST.to_owned(),
        snapshot_digest: DIGEST.to_owned(),
        plan_task_digest: DIGEST.to_owned(),
        subject: "did:key:benchmark".to_owned(),
        purpose: "benchmark".to_owned(),
        effective_projection: (0..projection_count)
            .map(|item| format!("column_{item:03}"))
            .collect(),
        authorization_receipt_digest: DIGEST.to_owned(),
    }
}

fn benchmark_governed_snapshot_digest(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("memory/governed_snapshot_digest");
    for projection_count in [3, 128] {
        let snapshot = governed_snapshot(projection_count);
        group.bench_with_input(
            BenchmarkId::from_parameter(projection_count),
            &projection_count,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(
                        black_box(&snapshot)
                            .digest()
                            .expect("valid governed snapshot fixture"),
                    )
                });
            },
        );
    }
    group.finish();
}

fn record(item: usize) -> StoredRecord {
    serde_json::from_value(json!({
        "id": format!("memory-{item:08}"),
        "space_id": "memory/user:benchmark/semantic",
        "kind": "semantic",
        "label": "internal",
        "quarantined": false,
        "entities": [{"name": format!("entity-{:04}", item % 2_048), "kind": "subject"}],
        "provenance": {"source": "operator"},
        "observed_at": Utc.timestamp_opt(1_786_032_000 + i64::try_from(item).expect("timestamp offset fits i64"), 0).single().expect("benchmark timestamp"),
        "valid_from": Utc.timestamp_opt(1_786_032_000, 0).single().expect("benchmark timestamp"),
        "invalid_at": null,
        "expires_at": null,
        "purposes": [],
        "content": {"text": format!("benchmark memory {item}"), "attributes": {}}
    }))
    .expect("valid stored-record fixture")
}

fn seeded_store(size: usize) -> GraphStoreMemoryStore<MemoryGraphStore> {
    let store = GraphStoreMemoryStore::new(MemoryGraphStore::default());
    let records = (0..size)
        .map(|item| StoreBatchOp::Put(Box::new(record(item))))
        .collect();
    store.apply_batch(records).expect("seed records");
    store
}

fn benchmark_store_query(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("memory/store_query_top_10");
    let size = 1_024;
    let store = seeded_store(size);
    let query = StoreQuery {
        space_id: Some("memory/user:benchmark/semantic".to_owned()),
        limit: Some(10),
        ..StoreQuery::default()
    };
    group.throughput(Throughput::Elements(
        u64::try_from(size).expect("benchmark size fits u64"),
    ));
    group.bench_with_input(BenchmarkId::from_parameter(size), &size, |bencher, _| {
        bencher.iter(|| {
            black_box(
                store
                    .query(black_box(&query))
                    .expect("in-memory query cannot fail"),
            )
        });
    });
    group.finish();
}

fn linked_store(edge_count: usize) -> (GraphStoreMemoryStore<MemoryGraphStore>, MemoryId) {
    let store = seeded_store(1);
    let id = MemoryId::from_string("memory-00000000");
    for edge in 0..edge_count {
        store
            .link("entity-root", "related", &format!("entity-{edge:08}"), &id)
            .expect("seed relationship");
    }
    (store, id)
}

fn benchmark_store_mutations(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("memory/store_mutation");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(5));
    group.bench_function("put_one_entity", |bencher| {
        bencher.iter_batched(
            || GraphStoreMemoryStore::new(MemoryGraphStore::default()),
            |store| {
                store.put(record(0)).expect("put record");
                black_box(store);
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("link", |bencher| {
        let id = MemoryId::from_string("memory-benchmark");
        bencher.iter_batched(
            || GraphStoreMemoryStore::new(MemoryGraphStore::default()),
            |store| {
                store
                    .link("entity-a", "related", "entity-b", &id)
                    .expect("link entities");
                black_box(store);
            },
            BatchSize::SmallInput,
        );
    });
    for edge_count in [1, 64, 1_024] {
        group.bench_with_input(
            BenchmarkId::new("tombstone_edges", edge_count),
            &edge_count,
            |bencher, &edge_count| {
                bencher.iter_batched(
                    || linked_store(edge_count),
                    |(store, id)| black_box(store.tombstone(&id).expect("tombstone record")),
                    BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_vector_search,
    benchmark_context_planning,
    benchmark_governed_snapshot_digest,
    benchmark_store_query,
    benchmark_store_mutations
);
criterion_main!(benches);
