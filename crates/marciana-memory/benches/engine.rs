use std::hint::black_box;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use querygraph_memory::cognition::{
    CognitionEngine, CognitionFieldMapping, CognitionOperation, CognitionRequest,
    GovernedLakeCatSnapshot, SailCognitionEngine, SailCognitionExecutor,
    SailCognitionExecutorError, SailCognitionOutput,
};
use serde_json::json;
use tokio::runtime::Builder;
use typesec_core::policy::{MintOptions, RequestContext, mint_capability_for_id};
use typesec_core::{CanRead, Capability, Resource};
use typesec_memory::{
    AuthorizedCognitionInput, CognitionBinding, ConsolidationPlan, InMemoryStore, Label, MemoryId,
    MemorySpace, MemoryStore, MemoryVault, StoredRecord,
};

const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SNAPSHOT_DIGEST: &str =
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const POLICY: &str = r#"
roles:
  - name: cognition-reader
    permissions: [read]
    resources: ["memory/user:benchmark/**"]
assignments:
  - subject: "agent:benchmark"
    roles: [cognition-reader]
"#;

struct EmptySailExecutor;

#[async_trait::async_trait]
impl SailCognitionExecutor for EmptySailExecutor {
    async fn execute(
        &self,
        _request: &CognitionRequest<'_>,
    ) -> Result<SailCognitionOutput, SailCognitionExecutorError> {
        Ok(SailCognitionOutput {
            plan: ConsolidationPlan::new(),
            evidence: Vec::new(),
        })
    }
}

fn record(item: usize) -> StoredRecord {
    serde_json::from_value(json!({
        "id": format!("memory-{item:08}"),
        "space_id": "memory/user:benchmark/semantic",
        "kind": "semantic",
        "label": "internal",
        "quarantined": false,
        "entities": [],
        "provenance": { "source": "operator" },
        "observed_at": Utc.timestamp_opt(1_786_032_000, 0).single().expect("benchmark timestamp"),
        "valid_from": Utc.timestamp_opt(1_786_032_000, 0).single().expect("benchmark timestamp"),
        "invalid_at": null,
        "expires_at": null,
        "purposes": ["benchmark"],
        "content": { "text": format!("unique benchmark assertion {item}") }
    }))
    .expect("valid cognition record")
}

fn authorized_input(source_count: usize, reverse_source_ids: bool) -> AuthorizedCognitionInput {
    let store = InMemoryStore::new();
    for item in 0..source_count {
        store.put(record(item)).expect("seed cognition source");
    }
    let mut source_ids = (0..source_count)
        .map(|item| MemoryId::from_string(format!("memory-{item:08}")))
        .collect::<Vec<_>>();
    if reverse_source_ids {
        source_ids.reverse();
    }
    let vault = MemoryVault::new(store);
    let space = MemorySpace::new("user:benchmark", "semantic");
    let policy = typesec_rbac::RbacEngine::from_yaml(POLICY).expect("valid benchmark policy");
    let capability: Capability<CanRead, MemorySpace> = mint_capability_for_id(
        &policy,
        "agent:benchmark",
        space.resource_id(),
        &MintOptions::default(),
    )
    .expect("benchmark capability");
    vault
        .cognition_input_at(
            &space,
            &capability,
            &source_ids,
            &RequestContext::new().with_purpose("benchmark"),
            Label::Secret,
        )
        .expect("authorized cognition input")
}

fn governed_source() -> GovernedLakeCatSnapshot {
    let mut effective_projection = (0..125)
        .map(|item| format!("field_{item:03}"))
        .collect::<Vec<_>>();
    effective_projection.extend(["id".to_owned(), "text".to_owned(), "valid_from".to_owned()]);
    GovernedLakeCatSnapshot {
        catalog: "lakecat://benchmark".to_owned(),
        namespace: "benchmark".to_owned(),
        table: "memories".to_owned(),
        snapshot_id: 42,
        governed_scan_digest: DIGEST.to_owned(),
        snapshot_digest: SNAPSHOT_DIGEST.to_owned(),
        plan_task_digest: DIGEST.to_owned(),
        subject: "agent:benchmark".to_owned(),
        purpose: "benchmark".to_owned(),
        effective_projection,
        authorization_receipt_digest: DIGEST.to_owned(),
    }
}

fn binding(source: &GovernedLakeCatSnapshot, input: &AuthorizedCognitionInput) -> CognitionBinding {
    CognitionBinding {
        space_id: "memory/user:benchmark/semantic".to_owned(),
        subject: source.subject.clone(),
        purpose: source.purpose.clone(),
        governed_source_scope: None,
        governed_scan_digest: source.governed_scan_digest.clone(),
        snapshot_digest: source.snapshot_digest.clone(),
        plan_task_digest: source.plan_task_digest.clone(),
        authorization_receipt_digest: source.authorization_receipt_digest.clone(),
        effective_projection: source.effective_projection.clone(),
        source_manifest_digest: input.manifest().digest.clone(),
        typedid_request_digest: DIGEST.to_owned(),
    }
}

fn benchmark_cognition_engine(criterion: &mut Criterion) {
    let runtime = Builder::new_current_thread()
        .build()
        .expect("benchmark runtime");
    let engine = SailCognitionEngine::new(EmptySailExecutor);
    let source = governed_source();
    let mapping = CognitionFieldMapping {
        id: "id".to_owned(),
        text: "text".to_owned(),
        valid_from: "valid_from".to_owned(),
    };
    let mut group = criterion.benchmark_group("memory/cognition_engine");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(5));
    for source_count in [64, 1_024, 4_096] {
        let input = authorized_input(source_count, true);
        let canonical_binding = binding(&source, &input);
        let mut reordered_binding = canonical_binding.clone();
        reordered_binding.effective_projection.reverse();
        let reordered_request = CognitionRequest {
            job_id: "job-benchmark",
            source: &source,
            binding: &reordered_binding,
            input: &input,
            field_mapping: &mapping,
            operation: CognitionOperation::Deduplicate,
        };
        let canonical_request = CognitionRequest {
            binding: &canonical_binding,
            ..reordered_request
        };
        group.throughput(Throughput::Elements(
            u64::try_from(source_count).expect("benchmark size fits u64"),
        ));
        group.bench_with_input(
            BenchmarkId::from_parameter(source_count),
            &source_count,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(runtime.block_on(engine.propose(black_box(reordered_request))))
                        .expect("valid cognition proposal")
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("canonical_projection", source_count),
            &source_count,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(runtime.block_on(engine.propose(black_box(canonical_request))))
                        .expect("valid cognition proposal")
                });
            },
        );

        let canonical_input = authorized_input(source_count, false);
        let canonical_source_binding = binding(&source, &canonical_input);
        let canonical_source_request = CognitionRequest {
            binding: &canonical_source_binding,
            input: &canonical_input,
            ..canonical_request
        };
        group.bench_with_input(
            BenchmarkId::new("canonical_sources", source_count),
            &source_count,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(runtime.block_on(engine.propose(black_box(canonical_source_request))))
                        .expect("valid cognition proposal")
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, benchmark_cognition_engine);
criterion_main!(benches);
