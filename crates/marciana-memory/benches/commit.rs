use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration as StdDuration;

use chrono::{Duration, TimeZone, Utc};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use querygraph_memory::TursoMemoryStore;
use querygraph_memory::turso::TursoConfig;
use sha2::{Digest, Sha256};
use typesec_core::policy::{MintOptions, RequestContext, mint_capability_for_id};
use typesec_core::{CanRead, CanWrite, Capability, Permission, Resource};
use typesec_memory::{
    CognitionAuthorityError, CognitionAuthorityEvidence, CognitionAuthorityVerifier,
    CognitionBinding, CognitionCommitOutcome, CognitionEffect, CognitionIdempotencyKey,
    CognitionProposal, ConsolidationPlan, ConsolidationStep, Label, MemorySpace, MemoryStore,
    MemoryVault, StoredRecord,
};

const SUBJECT: &str = "agent:benchmark";
const PURPOSE: &str = "benchmark";
const POLICY: &str = r#"
roles:
  - name: cognition-worker
    permissions: [read, write]
    resources: ["memory/user:benchmark/**"]
assignments:
  - subject: "agent:benchmark"
    roles: [cognition-worker]
"#;

fn at(second: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(1_786_032_000 + second, 0)
        .single()
        .expect("benchmark timestamp")
}

fn digest(value: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(value.as_bytes()))
}

fn source() -> StoredRecord {
    serde_json::from_value(serde_json::json!({
        "id": "source-benchmark",
        "space_id": "memory/user:benchmark/semantic",
        "kind": "semantic",
        "label": "internal",
        "quarantined": false,
        "entities": [],
        "provenance": { "source": "operator" },
        "observed_at": at(0),
        "valid_from": at(0),
        "invalid_at": null,
        "expires_at": null,
        "purposes": [PURPOSE],
        "content": { "text": "benchmark source text" }
    }))
    .expect("valid benchmark source")
}

fn store() -> TursoMemoryStore {
    TursoMemoryStore::open_with_config(TursoConfig {
        path: ":memory:".to_owned(),
        table_prefix: "marciana_commit_bench".to_owned(),
        batch_size: 32,
        ..TursoConfig::default()
    })
    .expect("open benchmark store")
}

fn capability<P: Permission>(
    policy: &typesec_rbac::RbacEngine,
    space: &MemorySpace,
    context: &RequestContext,
) -> Capability<P, MemorySpace> {
    mint_capability_for_id(
        policy,
        SUBJECT,
        space.resource_id(),
        &MintOptions {
            context: context.clone(),
            ..MintOptions::default()
        },
    )
    .expect("benchmark capability")
}

fn job_key() -> CognitionIdempotencyKey {
    CognitionIdempotencyKey::for_authority(
        "memory/user:benchmark/semantic",
        SUBJECT,
        PURPOSE,
        "job-benchmark",
    )
    .expect("canonical cognition job key")
}

struct CommitFixture {
    vault: MemoryVault<TursoMemoryStore>,
    space: MemorySpace,
    write: Capability<CanWrite, MemorySpace>,
    context: RequestContext,
    proposal: CognitionProposal,
    key: CognitionIdempotencyKey,
}

impl CommitFixture {
    fn new(effect: CognitionEffect) -> Self {
        let policy = Arc::new(typesec_rbac::RbacEngine::from_yaml(POLICY).expect("valid policy"));
        let space = MemorySpace::new("user:benchmark", "semantic");
        let context = RequestContext::new().with_purpose(PURPOSE);
        let read = capability::<CanRead>(&policy, &space, &context);
        let write = capability::<CanWrite>(&policy, &space, &context);
        let vault = MemoryVault::new(store()).with_policy(policy);
        let source = source();
        vault.store().put(source.clone()).expect("persist source");
        let manifest = vault
            .cognition_source_manifest(&space, &read, std::slice::from_ref(&source.id), &context)
            .expect("source manifest");
        let binding = CognitionBinding {
            space_id: space.resource_id().to_owned(),
            subject: SUBJECT.to_owned(),
            purpose: PURPOSE.to_owned(),
            governed_source_scope: None,
            governed_scan_digest: digest("scan"),
            snapshot_digest: digest("snapshot"),
            plan_task_digest: digest("plan"),
            authorization_receipt_digest: digest("authorization"),
            effective_projection: vec!["finding".to_owned()],
            source_manifest_digest: manifest.digest,
            typedid_request_digest: digest("typedid"),
        };
        let mut proposal = CognitionProposal::new(
            "job-benchmark",
            binding.snapshot_digest.clone(),
            binding.source_manifest_digest.clone(),
            "marciana.benchmark",
            "1",
            vec![source.id.clone()],
            Label::Internal,
        )
        .with_plan(
            ConsolidationPlan::new().then(ConsolidationStep::Invalidate {
                ids: vec![source.id],
            }),
        )
        .with_binding(binding.clone());
        if effect == CognitionEffect::NoChange {
            proposal.effect = CognitionEffect::NoChange;
            proposal.plan = ConsolidationPlan::new();
        }
        let key = job_key();
        vault
            .store()
            .submit_cognition_job(&key, "scheduler", &binding.typedid_request_digest, 3, at(1))
            .expect("submit benchmark job");
        let lease = vault
            .store()
            .acquire_cognition_lease(&key, "worker", at(2), Duration::minutes(5))
            .expect("lease benchmark job");
        vault
            .store()
            .persist_cognition_proposal(&key, lease.token(), &proposal, at(3))
            .expect("stage benchmark proposal");
        let authority = Arc::new(EchoAuthority::for_proposal(&proposal));
        Self {
            vault: vault.with_cognition_authority(authority),
            space,
            write,
            context,
            proposal,
            key,
        }
    }

    fn apply(&self) -> CognitionCommitOutcome {
        self.vault
            .apply_cognition(&self.space, &self.write, &self.proposal, &self.context)
            .expect("apply benchmark cognition")
    }
}

struct EchoAuthority {
    job_id: String,
    algorithm: String,
    algorithm_version: String,
}

impl EchoAuthority {
    fn for_proposal(proposal: &CognitionProposal) -> Self {
        Self {
            job_id: proposal.job_id.clone(),
            algorithm: proposal.algorithm.clone(),
            algorithm_version: proposal.algorithm_version.clone(),
        }
    }
}

impl CognitionAuthorityVerifier for EchoAuthority {
    fn revalidate(
        &self,
        binding: &CognitionBinding,
        _context: &RequestContext,
    ) -> Result<CognitionAuthorityEvidence, CognitionAuthorityError> {
        Ok(CognitionAuthorityEvidence {
            space_id: binding.space_id.clone(),
            subject: binding.subject.clone(),
            purpose: binding.purpose.clone(),
            governed_source_scope: None,
            job_id: self.job_id.clone(),
            algorithm: self.algorithm.clone(),
            algorithm_version: self.algorithm_version.clone(),
            governed_scan_digest: binding.governed_scan_digest.clone(),
            snapshot_digest: binding.snapshot_digest.clone(),
            plan_task_digest: binding.plan_task_digest.clone(),
            authorization_receipt_digest: binding.authorization_receipt_digest.clone(),
            effective_projection: binding.effective_projection.clone(),
            typedid_request_digest: binding.typedid_request_digest.clone(),
            policy_decision_id: "policy-decision:benchmark".to_owned(),
        })
    }
}

fn benchmark_cognition_commit(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("memory/cognition_commit");
    group.sample_size(20);
    group.measurement_time(StdDuration::from_secs(5));
    for (name, effect) in [
        ("apply_mutation", CognitionEffect::Mutated),
        ("apply_no_change", CognitionEffect::NoChange),
    ] {
        group.bench_function(name, |bencher| {
            bencher.iter_batched(
                || CommitFixture::new(effect),
                |fixture| black_box(fixture.apply()),
                BatchSize::SmallInput,
            );
        });
    }
    for (name, effect) in [
        ("recover_mutation", CognitionEffect::Mutated),
        ("recover_no_change", CognitionEffect::NoChange),
    ] {
        group.bench_function(name, |bencher| {
            bencher.iter_batched(
                || {
                    let fixture = CommitFixture::new(effect);
                    black_box(fixture.apply());
                    fixture
                },
                |fixture| black_box(fixture.apply()),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn benchmark_cognition_outbox(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("memory/cognition_outbox");
    group.sample_size(20);
    group.measurement_time(StdDuration::from_secs(5));
    group.bench_function("claim_one", |bencher| {
        bencher.iter_batched(
            || {
                let fixture = CommitFixture::new(CognitionEffect::Mutated);
                black_box(fixture.apply());
                fixture
            },
            |fixture| {
                black_box(
                    fixture
                        .vault
                        .store()
                        .claim_cognition_outbox(
                            black_box(&fixture.key),
                            black_box("index-worker"),
                            black_box(at(10)),
                            black_box(Duration::minutes(5)),
                            black_box(1),
                        )
                        .expect("claim benchmark outbox"),
                )
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(
    benches,
    benchmark_cognition_commit,
    benchmark_cognition_outbox
);
criterion_main!(benches);
