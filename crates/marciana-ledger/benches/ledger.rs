use std::hint::black_box;

use chrono::{DateTime, TimeZone, Utc};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use marciana_ledger::{
    Assertion, AssertionId, AssertionLineage, AssertionQuery, AssertionState, AssertionTransition,
    Confidence, TemporalInterval, TransitionEvidence,
};

const BASE_TIMESTAMP: i64 = 1_786_032_000;
const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn at(offset: usize) -> DateTime<Utc> {
    Utc.timestamp_opt(
        BASE_TIMESTAMP + i64::try_from(offset).expect("benchmark timestamp fits i64"),
        0,
    )
    .single()
    .expect("benchmark timestamp is representable")
}

fn assertion(valid_from_offset: usize) -> Assertion {
    Assertion::new(
        AssertionId::new(),
        "account:acme",
        "locatedIn",
        "place:venice",
        Confidence::from_basis_points(8_500).expect("valid confidence"),
        at(valid_from_offset),
        at(valid_from_offset + 1),
        TemporalInterval::new(at(valid_from_offset), None).expect("valid interval"),
        AssertionLineage::new(
            "episode:benchmark",
            format!("record:{valid_from_offset}"),
            "conversation-v1",
            "assertion-v1",
        )
        .expect("valid lineage"),
    )
    .expect("valid assertion")
}

fn assertion_with_history(transition_count: usize) -> Assertion {
    let mut value = assertion(0);
    let cause = AssertionId::new();
    for index in 0..transition_count {
        let (from, to) = if index == 0 {
            (AssertionState::Proposed, AssertionState::Current)
        } else if index % 2 == 1 {
            (AssertionState::Current, AssertionState::Disputed)
        } else {
            (AssertionState::Disputed, AssertionState::Current)
        };
        let evidence = TransitionEvidence::new(vec![cause.clone()], vec![DIGEST.to_owned()])
            .expect("valid evidence");
        value
            .apply_transition(
                AssertionTransition::new(from, to, at(index + 2), evidence)
                    .expect("valid transition"),
            )
            .expect("monotonic transition");
    }
    value
}

fn benchmark_state_at(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("ledger/state_at");
    for transition_count in [16, 256, 4_096] {
        let value = assertion_with_history(transition_count);
        let query_at = at(transition_count.saturating_sub(1) + 2);
        group.bench_with_input(
            BenchmarkId::from_parameter(transition_count),
            &transition_count,
            |bencher, _| {
                bencher.iter(|| black_box(value.state_at(black_box(query_at))));
            },
        );
    }
    group.finish();
}

fn benchmark_query_selection(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("ledger/query_select");
    for assertion_count in [1_024, 16_384] {
        let assertions = (0..assertion_count)
            .map(|index| {
                let mut value = assertion(index);
                value
                    .apply_transition(
                        AssertionTransition::new(
                            AssertionState::Proposed,
                            AssertionState::Current,
                            at(index + 2),
                            TransitionEvidence::new(
                                vec![AssertionId::new()],
                                vec![DIGEST.to_owned()],
                            )
                            .expect("valid evidence"),
                        )
                        .expect("valid transition"),
                    )
                    .expect("monotonic transition");
                value
            })
            .collect::<Vec<_>>();
        let query = AssertionQuery::current_at(at(assertion_count + 3));
        group.bench_with_input(
            BenchmarkId::from_parameter(assertion_count),
            &assertion_count,
            |bencher, _| {
                bencher.iter(|| black_box(query.select(black_box(&assertions))));
            },
        );
    }
    group.finish();
}

fn benchmark_evidence_canonicalization(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("ledger/evidence_canonicalization");
    group.sample_size(50);
    for evidence_count in [16_usize, 256, 4_096] {
        let mut assertions = (0..evidence_count)
            .map(|index| {
                AssertionId::parse(format!("{index:08x}-0000-4000-8000-{index:012x}"))
                    .expect("canonical benchmark UUID")
            })
            .collect::<Vec<_>>();
        let mut digests = (0..evidence_count)
            .map(|index| format!("sha256:{index:064x}"))
            .collect::<Vec<_>>();
        assertions.reverse();
        digests.reverse();
        let (scrambled_assertions, scrambled_digests) = (0..evidence_count)
            .map(|position| {
                let index = position.wrapping_mul(2_654_435_769) % evidence_count;
                (
                    AssertionId::parse(format!("{index:08x}-0000-4000-8000-{index:012x}"))
                        .expect("canonical benchmark UUID"),
                    format!("sha256:{index:064x}"),
                )
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();
        group.bench_with_input(
            BenchmarkId::from_parameter(evidence_count),
            &evidence_count,
            |bencher, _| {
                bencher.iter_batched(
                    || (assertions.clone(), digests.clone()),
                    |(assertions, digests)| {
                        black_box(
                            TransitionEvidence::new(assertions, digests)
                                .expect("valid evidence corpus"),
                        )
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("scrambled", evidence_count),
            &evidence_count,
            |bencher, _| {
                bencher.iter_batched(
                    || (scrambled_assertions.clone(), scrambled_digests.clone()),
                    |(assertions, digests)| {
                        black_box(
                            TransitionEvidence::new(assertions, digests)
                                .expect("valid evidence corpus"),
                        )
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_state_at,
    benchmark_query_selection,
    benchmark_evidence_canonicalization
);
criterion_main!(benches);
