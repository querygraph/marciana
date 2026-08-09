use std::hint::black_box;
use std::time::Duration as StdDuration;

use chrono::{Duration, TimeZone, Utc};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use querygraph_memory::TursoMemoryStore;
use querygraph_memory::cognition::{
    CognitionJobClaimRequest, CognitionProgress, CognitionProgressPhase,
};
use querygraph_memory::turso::TursoConfig;
use typesec_memory::CognitionIdempotencyKey;

const REQUEST_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn at(second: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(1_786_032_000 + second, 0)
        .single()
        .expect("benchmark timestamp")
}

fn job_key_for(job_id: &str) -> CognitionIdempotencyKey {
    CognitionIdempotencyKey::for_authority(
        "memory/user:benchmark/semantic",
        "agent:benchmark",
        "benchmark",
        job_id,
    )
    .expect("canonical cognition job key")
}

fn job_key() -> CognitionIdempotencyKey {
    job_key_for("job-benchmark")
}

fn store() -> TursoMemoryStore {
    TursoMemoryStore::open_with_config(TursoConfig {
        path: ":memory:".to_owned(),
        table_prefix: "marciana_persistence_bench".to_owned(),
        batch_size: 32,
        ..TursoConfig::default()
    })
    .expect("open benchmark store")
}

fn pending_store() -> (TursoMemoryStore, CognitionIdempotencyKey) {
    let store = store();
    let key = job_key();
    store
        .submit_cognition_job(&key, "scheduler", REQUEST_DIGEST, 3, at(0))
        .expect("submit benchmark job");
    (store, key)
}

fn warm_store() -> (TursoMemoryStore, CognitionIdempotencyKey) {
    let store = store();
    store
        .submit_cognition_job(
            &job_key_for("seed-job"),
            "scheduler",
            REQUEST_DIGEST,
            3,
            at(0),
        )
        .expect("seed benchmark store");
    (store, job_key())
}

fn leased_store() -> (TursoMemoryStore, CognitionIdempotencyKey, String) {
    let (store, key) = pending_store();
    let lease = store
        .acquire_cognition_lease(&key, "worker", at(1), Duration::minutes(5))
        .expect("lease benchmark job");
    let token = lease.token().to_owned();
    (store, key, token)
}

fn benchmark_cognition_submission(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("memory/cognition_submission");
    group.sample_size(30);
    group.measurement_time(StdDuration::from_secs(5));

    group.bench_function("submit", |bencher| {
        bencher.iter_batched(
            || (store(), job_key()),
            |(store, key)| {
                black_box(
                    store
                        .submit_cognition_job(
                            black_box(&key),
                            black_box("scheduler"),
                            black_box(REQUEST_DIGEST),
                            black_box(3),
                            black_box(at(0)),
                        )
                        .expect("submit benchmark job"),
                )
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("submit_warm", |bencher| {
        bencher.iter_batched(
            warm_store,
            |(store, key)| {
                black_box(
                    store
                        .submit_cognition_job(
                            black_box(&key),
                            black_box("scheduler"),
                            black_box(REQUEST_DIGEST),
                            black_box(3),
                            black_box(at(1)),
                        )
                        .expect("submit benchmark job to warm store"),
                )
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn benchmark_cognition_scheduler(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("memory/cognition_scheduler");
    group.sample_size(30);
    group.measurement_time(StdDuration::from_secs(5));

    let (store, key) = pending_store();
    group.bench_function("load_pending", |bencher| {
        bencher.iter(|| {
            black_box(
                store
                    .cognition_job(black_box(&key))
                    .expect("load benchmark job"),
            )
        });
    });

    group.bench_function("claim_pending", |bencher| {
        bencher.iter_batched(
            pending_store,
            |(store, key)| {
                black_box(
                    store
                        .claim_cognition_job(CognitionJobClaimRequest {
                            key: black_box(&key),
                            submitter: black_box("scheduler"),
                            worker: black_box("worker"),
                            typedid_request_digest: black_box(REQUEST_DIGEST),
                            max_attempts: black_box(3),
                            now: black_box(at(1)),
                            lease_ttl: black_box(Duration::minutes(5)),
                        })
                        .expect("claim benchmark job"),
                )
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("acquire_pending", |bencher| {
        bencher.iter_batched(
            pending_store,
            |(store, key)| {
                black_box(
                    store
                        .acquire_cognition_lease(
                            black_box(&key),
                            black_box("worker"),
                            black_box(at(1)),
                            black_box(Duration::minutes(5)),
                        )
                        .expect("lease benchmark job"),
                )
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("update_progress", |bencher| {
        bencher.iter_batched(
            leased_store,
            |(store, key, token)| {
                black_box(
                    store
                        .update_cognition_progress(
                            black_box(&key),
                            black_box(&token),
                            black_box(CognitionProgress {
                                phase: CognitionProgressPhase::Scanning,
                                completed_units: 64,
                                total_units: Some(1_024),
                                detail_digest: Some(REQUEST_DIGEST.to_owned()),
                                updated_at: at(2),
                            }),
                            black_box(at(2)),
                        )
                        .expect("update benchmark progress"),
                )
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(
    benches,
    benchmark_cognition_submission,
    benchmark_cognition_scheduler
);
criterion_main!(benches);
