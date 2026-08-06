#![cfg(feature = "turso")]

mod support;

use support::{job_key, job_key_for_authority};

use chrono::Duration;
use grust_core::prelude::{GraphStore, Start, Traversal};
use querygraph_memory::TursoMemoryStore;
use querygraph_memory::cognition::{
    CognitionJobClaim, CognitionJobClaimRequest, CognitionJobStatus, CognitionProgress,
    CognitionProgressPhase, CognitionStateError, MAX_COGNITION_BEARER_TOKEN_BYTES,
    MAX_COGNITION_FAILURE_BYTES,
};
use typesec_memory::{MemoryContent, MemoryDraft, MemoryStore, Provenance};

use support::{at, config, digest, proposal, record};

#[tokio::test]
async fn progress_is_lease_bound_bounded_and_digest_only() {
    let dir = tempfile::tempdir().expect("temporary database");
    let store =
        TursoMemoryStore::open_with_config(config(&dir, "cognition_progress")).expect("open store");
    let key = job_key("tenant/progress");
    store
        .submit_cognition_job(&key, "worker/owner", &digest("request"), 2, at(0))
        .expect("submit job");
    let lease = store
        .acquire_cognition_lease(&key, "worker/owner", at(1), Duration::minutes(5))
        .expect("acquire lease");
    let progress = CognitionProgress {
        phase: CognitionProgressPhase::Scanning,
        completed_units: 2,
        total_units: Some(3),
        detail_digest: Some(digest("scan")),
        updated_at: at(2),
    };
    let updated = store
        .update_cognition_progress(&key, lease.token(), progress)
        .expect("update progress");
    assert_eq!(updated.progress.phase, CognitionProgressPhase::Scanning);
    assert_eq!(updated.progress.completed_units, 2);
    assert!(
        serde_json::to_string(&updated)
            .expect("serialize progress")
            .contains("sha256:")
    );
    let err = store
        .update_cognition_progress(
            &key,
            "sha256:stale",
            CognitionProgress {
                phase: CognitionProgressPhase::Planning,
                completed_units: 3,
                total_units: Some(3),
                detail_digest: None,
                updated_at: at(3),
            },
        )
        .expect_err("stale token must not update progress");
    assert!(matches!(err, CognitionStateError::StaleLease));
}

#[tokio::test]
async fn staged_proposal_digest_survives_reopen_without_proposal_content() {
    let dir = tempfile::tempdir().expect("temporary database");
    let config = config(&dir, "cognition_job_reopen");
    let source = record("source-private", "private source text", None);
    let mut expected = proposal("tenant/private-job", &source);
    expected.drafts.push(MemoryDraft::new(
        source.kind,
        MemoryContent::text("sensitive derived proposal text"),
        Provenance::Operator,
    ));
    let expected_digest = expected.canonical_digest().expect("expected digest");
    {
        let store = TursoMemoryStore::open_with_config(config.clone()).expect("open store");
        store.put(source).expect("persist source");
        store
            .submit_cognition_job(
                &job_key("tenant/private-job"),
                "worker/private-owner",
                &digest("request"),
                3,
                at(0),
            )
            .expect("submit job");
        let lease = store
            .acquire_cognition_lease(
                &job_key("tenant/private-job"),
                "worker/private-owner",
                at(1),
                Duration::minutes(5),
            )
            .expect("acquire lease");
        store
            .persist_cognition_proposal(
                &job_key("tenant/private-job"),
                lease.token(),
                &expected,
                at(2),
            )
            .expect("persist proposal");
    }

    let reopened = TursoMemoryStore::open_with_config(config).expect("reopen store");
    let job = reopened
        .cognition_job(&job_key("tenant/private-job"))
        .expect("load job")
        .expect("job persisted");
    assert_eq!(job.status, CognitionJobStatus::ProposalReady);
    assert_ne!(job.job_digest, "tenant/private-job");
    assert_ne!(job.owner_digest, "worker/private-owner");
    assert_eq!(
        job.proposal_digest.as_deref(),
        Some(expected_digest.as_str())
    );
    let nodes = reopened
        .graph()
        .traverse(Traversal {
            start: Start::NodesByLabel("CognitionProposal".into()),
            steps: Vec::new(),
            limit: None,
        })
        .await
        .expect("read legacy proposal label");
    assert!(nodes.is_empty(), "proposal content must never be persisted");
    let encoded = serde_json::to_string(&job).expect("serialize staged job");
    for forbidden in [
        "private source text",
        "sensitive derived proposal text",
        "marciana.test",
    ] {
        assert!(!encoded.contains(forbidden), "job exposed {forbidden:?}");
    }
}

#[test]
fn staged_jobs_return_only_the_durable_digest_and_never_reacquire_a_lease() {
    let dir = tempfile::tempdir().expect("temporary database");
    let store =
        TursoMemoryStore::open_with_config(config(&dir, "staged_job_claim")).expect("open store");
    let source = record("source", "private source", None);
    let proposal = proposal("job", &source);
    let proposal_digest = proposal.canonical_digest().expect("proposal digest");
    store.put(source).expect("persist source");
    store
        .submit_cognition_job(&job_key("job"), "scheduler", &digest("request"), 3, at(0))
        .expect("submit job");
    let lease = store
        .acquire_cognition_lease(&job_key("job"), "worker", at(1), Duration::minutes(5))
        .expect("acquire lease");
    store
        .persist_cognition_proposal(&job_key("job"), lease.token(), &proposal, at(2))
        .expect("stage proposal");

    assert!(matches!(
        store.acquire_cognition_lease(&job_key("job"), "worker", at(3), Duration::minutes(5)),
        Err(CognitionStateError::InvalidTransition(_))
    ));
    assert!(matches!(
        store
            .claim_cognition_job(CognitionJobClaimRequest {
                key: &job_key("job"),
                submitter: "scheduler",
                worker: "recovery-worker",
                typedid_request_digest: &digest("request"),
                max_attempts: 3,
                now: at(3),
                lease_ttl: Duration::minutes(5),
            })
            .expect("recover staged job"),
        CognitionJobClaim::ProposalReady { proposal_digest: digest }
            if digest == proposal_digest
    ));
}

#[test]
fn leases_are_exclusive_and_stale_tokens_fail_closed() {
    let dir = tempfile::tempdir().expect("temporary database");
    let store = TursoMemoryStore::open_with_config(config(&dir, "cognition_lease_exclusive"))
        .expect("open store");
    store
        .submit_cognition_job(&job_key("job"), "scheduler", &digest("request"), 3, at(0))
        .expect("submit job");
    let lease = store
        .acquire_cognition_lease(&job_key("job"), "worker-a", at(1), Duration::minutes(1))
        .expect("worker A lease");
    assert!(matches!(
        store.acquire_cognition_lease(&job_key("job"), "worker-b", at(2), Duration::minutes(1)),
        Err(CognitionStateError::LeaseHeld)
    ));
    assert!(matches!(
        store.renew_cognition_lease(&job_key("job"), "sha256:stale", at(2), Duration::minutes(1)),
        Err(CognitionStateError::StaleLease)
    ));
    assert!(matches!(
        store.renew_cognition_lease(&job_key("job"), lease.token(), at(2), Duration::minutes(61)),
        Err(CognitionStateError::Invalid(_))
    ));
    let renewed = store
        .renew_cognition_lease(&job_key("job"), lease.token(), at(2), Duration::minutes(2))
        .expect("renew active lease");
    assert_eq!(renewed.token(), lease.token());
    assert!(renewed.expires_at() > lease.expires_at());
    let failed = store
        .fail_cognition_job(&job_key("job"), renewed.token(), "retryable failure", at(3))
        .expect("record failure under renewed lease");
    assert_eq!(failed.status, CognitionJobStatus::Failed);
    assert!(failed.lease.is_none());
}

#[test]
fn caller_controlled_hash_inputs_are_bounded_before_state_access() {
    let dir = tempfile::tempdir().expect("temporary database");
    let store = TursoMemoryStore::open_with_config(config(&dir, "cognition_hash_bounds"))
        .expect("open store");
    store
        .submit_cognition_job(&job_key("job"), "scheduler", &digest("request"), 3, at(0))
        .expect("submit job");
    let lease = store
        .acquire_cognition_lease(&job_key("job"), "worker", at(1), Duration::minutes(5))
        .expect("acquire lease");

    let exact_stale_token = "x".repeat(MAX_COGNITION_BEARER_TOKEN_BYTES);
    assert!(matches!(
        store.renew_cognition_lease(
            &job_key("job"),
            &exact_stale_token,
            at(2),
            Duration::minutes(6),
        ),
        Err(CognitionStateError::StaleLease)
    ));
    let over_token = format!("{exact_stale_token}x");
    assert!(matches!(
        store.renew_cognition_lease(&job_key("job"), &over_token, at(2), Duration::minutes(6),),
        Err(CognitionStateError::Invalid(_))
    ));

    let over_failure = "x".repeat(MAX_COGNITION_FAILURE_BYTES + 1);
    assert!(matches!(
        store.fail_cognition_job(&job_key("job"), lease.token(), &over_failure, at(2)),
        Err(CognitionStateError::Invalid(_))
    ));
    let exact_failure = "x".repeat(MAX_COGNITION_FAILURE_BYTES);
    let failed = store
        .fail_cognition_job(&job_key("job"), lease.token(), &exact_failure, at(2))
        .expect("inclusive failure diagnostic bound");
    assert_eq!(failed.status, CognitionJobStatus::Failed);
}

#[test]
fn identical_job_ids_are_isolated_by_memory_space() {
    let dir = tempfile::tempdir().expect("temporary database");
    let store = TursoMemoryStore::open_with_config(config(&dir, "cognition_job_spaces"))
        .expect("open store");
    let left = job_key("shared-job");
    let right = job_key_for_authority(
        "memory/user:bob/semantic",
        "did:key:alice",
        "research",
        "shared-job",
    );

    let left_job = store
        .submit_cognition_job(&left, "left-owner", &digest("left-request"), 2, at(0))
        .expect("submit left job");
    let right_job = store
        .submit_cognition_job(&right, "right-owner", &digest("right-request"), 3, at(0))
        .expect("submit right job");
    assert_ne!(left_job.job_digest, right_job.job_digest);
    store
        .acquire_cognition_lease(&left, "left-worker", at(1), Duration::minutes(1))
        .expect("lease left job");

    assert_eq!(
        store
            .cognition_job(&right)
            .expect("read right job")
            .expect("right job exists")
            .status,
        CognitionJobStatus::Pending
    );
}

#[test]
fn identical_space_and_job_ids_are_isolated_by_authority_scope() {
    let dir = tempfile::tempdir().expect("temporary database");
    let store = TursoMemoryStore::open_with_config(config(&dir, "cognition_job_authorities"))
        .expect("open store");
    let left = job_key_for_authority(
        "memory/user:alice/semantic",
        "did:key:alice",
        "research",
        "shared-job",
    );
    let right = job_key_for_authority(
        "memory/user:alice/semantic",
        "did:key:bob",
        "research",
        "shared-job",
    );

    let left_job = store
        .submit_cognition_job(&left, "left-owner", &digest("left-request"), 2, at(0))
        .expect("submit left authority job");
    let right_job = store
        .submit_cognition_job(&right, "right-owner", &digest("right-request"), 2, at(0))
        .expect("submit right authority job");
    assert_ne!(
        left.authority_scope_digest(),
        right.authority_scope_digest()
    );
    assert_ne!(left_job.job_digest, right_job.job_digest);
}

#[test]
fn expired_leases_recover_until_the_bounded_attempt_budget_cancels() {
    let dir = tempfile::tempdir().expect("temporary database");
    let store = TursoMemoryStore::open_with_config(config(&dir, "cognition_lease_recovery"))
        .expect("open store");
    store
        .submit_cognition_job(&job_key("job"), "scheduler", &digest("request"), 2, at(0))
        .expect("submit job");
    let first = store
        .acquire_cognition_lease(&job_key("job"), "worker-a", at(1), Duration::seconds(1))
        .expect("first attempt");
    let recovered = store
        .acquire_cognition_lease(&job_key("job"), "worker-b", at(3), Duration::seconds(1))
        .expect("recover expired lease");
    assert_eq!(recovered.attempt(), 2);
    assert_ne!(first.token(), recovered.token());
    assert!(matches!(
        store.acquire_cognition_lease(&job_key("job"), "worker-c", at(5), Duration::seconds(1)),
        Err(CognitionStateError::AttemptsExhausted)
    ));
    assert_eq!(
        store
            .cognition_job(&job_key("job"))
            .expect("load job")
            .expect("job exists")
            .status,
        CognitionJobStatus::Cancelled
    );
}

#[test]
fn failure_text_is_hashed_and_digest_inputs_reject_plaintext() {
    let dir = tempfile::tempdir().expect("temporary database");
    let store = TursoMemoryStore::open_with_config(config(&dir, "cognition_redaction"))
        .expect("open store");
    assert!(matches!(
        store.submit_cognition_job(&job_key("job"), "scheduler", "raw request", 2, at(0)),
        Err(CognitionStateError::Invalid(_))
    ));
    store
        .submit_cognition_job(
            &job_key("secret/job-id"),
            "secret/owner-id",
            &digest("request"),
            2,
            at(0),
        )
        .expect("submit job");
    let lease = store
        .acquire_cognition_lease(
            &job_key("secret/job-id"),
            "secret/worker-id",
            at(1),
            Duration::minutes(1),
        )
        .expect("lease");
    let job = store
        .fail_cognition_job(
            &job_key("secret/job-id"),
            lease.token(),
            "database password leaked in stack trace",
            at(2),
        )
        .expect("record failure");
    let encoded = serde_json::to_string(&job).expect("serialize job");
    for forbidden in [
        "secret/job-id",
        "secret/owner-id",
        "secret/worker-id",
        "did:key:alice",
        "research",
        "database password",
        lease.token(),
    ] {
        assert!(!encoded.contains(forbidden), "job exposed {forbidden:?}");
    }
}

#[test]
fn submission_and_proposal_digest_collisions_fail_closed() {
    let dir = tempfile::tempdir().expect("temporary database");
    let store = TursoMemoryStore::open_with_config(config(&dir, "cognition_collision"))
        .expect("open store");
    store
        .submit_cognition_job(&job_key("job"), "scheduler", &digest("request"), 3, at(0))
        .expect("submit job");
    assert!(matches!(
        store.submit_cognition_job(&job_key("job"), "scheduler", &digest("request-b"), 3, at(0)),
        Err(CognitionStateError::DigestCollision)
    ));
    let source = record("source", "source text", None);
    let first = proposal("job", &source);
    let lease = store
        .acquire_cognition_lease(&job_key("job"), "worker", at(1), Duration::minutes(1))
        .expect("lease");
    store
        .persist_cognition_proposal(&job_key("job"), lease.token(), &first, at(2))
        .expect("persist first proposal");

    let mut observational_retry = first.clone();
    observational_retry.created_at = at(59);
    assert_eq!(
        store
            .persist_cognition_proposal(&job_key("job"), lease.token(), &observational_retry, at(3))
            .expect("created-at-only retry is idempotent"),
        first.canonical_digest().expect("first digest")
    );
    let mut conflicting = first;
    conflicting.algorithm_version = "2".into();
    assert!(matches!(
        store.persist_cognition_proposal(&job_key("job"), lease.token(), &conflicting, at(3)),
        Err(CognitionStateError::DigestCollision)
    ));
}

#[test]
fn proposal_staging_rejects_a_substituted_typedid_request() {
    let dir = tempfile::tempdir().expect("temporary database");
    let store = TursoMemoryStore::open_with_config(config(&dir, "cognition_request_binding"))
        .expect("open store");
    let proposal = proposal("job", &record("source", "source text", None));
    store
        .submit_cognition_job(
            &job_key("job"),
            "scheduler",
            &digest("another verified TypeDID request"),
            3,
            at(0),
        )
        .expect("submit job");
    let lease = store
        .acquire_cognition_lease(&job_key("job"), "worker", at(1), Duration::minutes(1))
        .expect("lease");

    assert!(matches!(
        store.persist_cognition_proposal(&job_key("job"), lease.token(), &proposal, at(2)),
        Err(CognitionStateError::DigestCollision)
    ));
    let job = store
        .cognition_job(&job_key("job"))
        .expect("load job")
        .expect("job exists");
    assert_eq!(job.status, CognitionJobStatus::Leased);
    assert!(job.proposal_digest.is_none());
}

#[tokio::test]
async fn graph_job_nodes_contain_only_digest_safe_state() {
    let dir = tempfile::tempdir().expect("temporary database");
    let store = TursoMemoryStore::open_with_config(config(&dir, "cognition_job_graph"))
        .expect("open store");
    store
        .submit_cognition_job(
            &job_key("secret/job-id"),
            "secret/owner-id",
            &digest("request"),
            2,
            at(0),
        )
        .expect("submit job");
    let nodes = store
        .graph()
        .traverse(Traversal {
            start: Start::NodesByLabel("CognitionJob".into()),
            steps: Vec::new(),
            limit: None,
        })
        .await
        .expect("read job graph");
    let encoded = serde_json::to_string(&nodes).expect("serialize nodes");
    assert!(!encoded.contains("secret/job-id"));
    assert!(!encoded.contains("secret/owner-id"));
}
