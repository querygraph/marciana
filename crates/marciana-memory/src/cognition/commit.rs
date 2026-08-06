//! Atomic `TypeSec` cognition application over [`GraphCommitStore`].

use grust_core::prelude::{
    GraphCommitStore, GraphExpectation, GraphMutation, GrustError, GuardedGraphCommit,
};
use typesec_memory::{
    CognitionCommitError, CognitionCommitOutcome, CognitionCommitStatus, CognitionCommitStore,
    CognitionIdempotencyKey, PreparedCognitionCommit, StoreBatchOp,
};

use std::collections::BTreeMap;

use grust_core::prelude::Node;
use typesec_memory::MemoryId;

use super::CognitionJobStatus;
use super::commit_envelope::{completed_job_digest, envelope_digest, resulting_version};
use super::commit_outcome::{map_outcome, recover};
use super::commit_sources::ExactSource;
use super::commit_support::{
    AUDIT_DOMAIN, json_commit_digest, map_graph_error, state_store_error, store_error,
};
use super::commit_validation::validate_prepared_commit;
use super::graph::{
    DurableOutcome, audit_node_id, commit_ledger_key, encode_audit, encode_outbox, encode_outcome,
    outcome_node_id,
};
use super::state::CognitionJob;
use crate::{GraphStoreMemoryStore, record_node_id};

const PRIOR_DOMAIN: &str = "querygraph.cognition.prior-version.v1";

impl<G: GraphCommitStore> CognitionCommitStore for GraphStoreMemoryStore<G> {
    fn recover_cognition(
        &self,
        key: &CognitionIdempotencyKey,
        proposal_digest: &str,
    ) -> Result<Option<CognitionCommitOutcome>, CognitionCommitError> {
        recover(self, key, proposal_digest)
    }

    fn commit_cognition(
        &self,
        commit: PreparedCognitionCommit,
    ) -> Result<CognitionCommitOutcome, CognitionCommitError> {
        validate_prepared_commit(&commit)?;
        if let Some(recovered) =
            self.recover_cognition(commit.idempotency_key(), commit.proposal_digest())?
        {
            return Ok(recovered);
        }

        let sources = match self.load_exact_sources(
            commit.source_preconditions(),
            commit.idempotency_key().space_id(),
        ) {
            Ok(sources) => sources,
            Err(stale @ CognitionCommitError::StaleSource(_)) => {
                if let Some(recovered) =
                    self.recover_cognition(commit.idempotency_key(), commit.proposal_digest())?
                {
                    return Ok(recovered);
                }
                return Err(stale);
            }
            Err(error) => return Err(error),
        };
        let prior_version = json_commit_digest(PRIOR_DOMAIN, commit.source_preconditions())?;
        let prepared_digest = commit
            .canonical_digest()
            .map_err(|_| store_error("prepared cognition digest failed"))?;
        let resulting_version =
            resulting_version(commit.effect(), &prior_version, &prepared_digest);
        let Some((job_node, mut job)) = self.staged_job_for_commit(&commit)? else {
            // An identical commit may have completed after the initial recovery
            // lookup but before this job read.
            if let Some(recovered) =
                self.recover_cognition(commit.idempotency_key(), commit.proposal_digest())?
            {
                return Ok(recovered);
            }
            return Err(store_error(
                "cognition job has no matching staged proposal and TypeDID request",
            ));
        };
        // A worker lease only gates proposal staging. Its expiry cannot confer
        // mutation authority or invalidate the vault's freshly prepared,
        // opaque TypeSec commit token checked above.

        let mut expectations = base_commit_expectations(&commit, &sources, job_node);
        let record_changes = self.commit_record_changes(commit.operations(), &sources)?;
        expectations.extend(record_changes.shared_node_expectations);
        let mut mutations = record_changes.mutations;
        let mut outbox_node_ids = Vec::with_capacity(commit.index_outbox().len());
        for (ordinal, mutation) in commit.index_outbox().iter().enumerate() {
            let node = encode_outbox(commit.idempotency_key(), ordinal, mutation)
                .map_err(state_store_error)?;
            expectations.push(GraphExpectation::Absent(node.id.clone()));
            outbox_node_ids.push(node.id.as_str().to_owned());
            mutations.push(GraphMutation::UpsertNode(node));
        }
        let audit_node =
            encode_audit(commit.idempotency_key(), commit.audit()).map_err(state_store_error)?;
        mutations.push(GraphMutation::UpsertNode(audit_node.clone()));
        super::backend::mark_job_completed(&mut job, &prepared_digest, commit.audit().prepared_at)
            .map_err(state_store_error)?;
        let job_node =
            super::graph::encode_job(commit.idempotency_key(), &job).map_err(state_store_error)?;
        let durable = durable_outcome(
            &commit,
            prepared_digest,
            prior_version,
            resulting_version,
            audit_node.id.as_str().to_owned(),
            &job,
            outbox_node_ids,
        )?;
        mutations.push(GraphMutation::UpsertNode(
            encode_outcome(commit.idempotency_key(), &durable).map_err(state_store_error)?,
        ));
        mutations.push(GraphMutation::UpsertNode(job_node));

        let guarded = GuardedGraphCommit::new(
            commit_ledger_key(commit.idempotency_key()),
            durable.envelope_digest.clone(),
            mutations,
        )
        .with_expectations(expectations);
        let receipt = match self.bridge.run(self.graph.commit_guarded(&guarded)) {
            Ok(receipt) => receipt,
            Err(GrustError::GraphIdempotencyConflict(_)) => {
                return self
                    .recover_cognition(commit.idempotency_key(), commit.proposal_digest())?
                    .ok_or(CognitionCommitError::IdempotencyConflict);
            }
            Err(GrustError::GraphExpectationFailed(_)) => {
                return self.recover_expectation_failure(&commit);
            }
            Err(error) => return Err(map_graph_error(&error)),
        };
        if receipt.replayed {
            return self
                .recover_cognition(commit.idempotency_key(), commit.proposal_digest())?
                .ok_or_else(|| store_error("replayed commit has no durable outcome"));
        }
        map_outcome(
            &receipt,
            &durable,
            commit.audit().clone(),
            CognitionCommitStatus::Applied,
        )
    }
}

impl<G: GraphCommitStore> GraphStoreMemoryStore<G> {
    /// Load the staged job and check it agrees with this prepared commit.
    /// `None` means the job moved on; the caller must retry recovery.
    fn staged_job_for_commit(
        &self,
        commit: &PreparedCognitionCommit,
    ) -> Result<Option<(Node, CognitionJob)>, CognitionCommitError> {
        let (job_node, job) = self
            .load_job_node(commit.idempotency_key())
            .map_err(state_store_error)?
            .ok_or_else(|| store_error("cognition job was not durably submitted"))?;
        let staged = job.status == CognitionJobStatus::ProposalReady
            && job.proposal_digest.as_deref() == Some(commit.proposal_digest())
            && job.typedid_request_digest == commit.audit().typedid_request_digest;
        Ok(staged.then_some((job_node, job)))
    }

    /// A failed CAS means another process advanced the job or a source moved:
    /// prefer the durable outcome, then a stale-source diagnosis.
    fn recover_expectation_failure(
        &self,
        commit: &PreparedCognitionCommit,
    ) -> Result<CognitionCommitOutcome, CognitionCommitError> {
        if let Some(recovered) =
            self.recover_cognition(commit.idempotency_key(), commit.proposal_digest())?
        {
            return Ok(recovered);
        }
        if let Some(stale) = self.find_stale_source(commit.source_preconditions())? {
            return Err(CognitionCommitError::StaleSource(stale));
        }
        Err(store_error(
            "cognition job or proposal changed before commit",
        ))
    }
}

/// Assemble the durable outcome and bind its envelope digest last, so the
/// envelope covers every other field.
fn durable_outcome(
    commit: &PreparedCognitionCommit,
    prepared_digest: String,
    prior_version: String,
    resulting_version: String,
    audit_node_id: String,
    job: &CognitionJob,
    outbox_node_ids: Vec<String>,
) -> Result<DurableOutcome, CognitionCommitError> {
    let mut durable = DurableOutcome {
        schema_version: super::graph::OUTCOME_SCHEMA_VERSION,
        effect: commit.effect(),
        proposal_digest: commit.proposal_digest().to_owned(),
        prepared_digest,
        prior_version,
        resulting_version,
        audit_node_id,
        audit_digest: json_commit_digest(AUDIT_DOMAIN, commit.audit())?,
        completed_job_digest: completed_job_digest(job)?,
        outbox_node_ids,
        envelope_digest: String::new(),
    };
    durable.envelope_digest = envelope_digest(&durable)?;
    Ok(durable)
}

/// The CAS expectation set every cognition commit starts from: exact source
/// revisions, the exact staged job, and the absence of the outcome, audit,
/// and newly created record nodes.
fn base_commit_expectations(
    commit: &PreparedCognitionCommit,
    sources: &BTreeMap<MemoryId, ExactSource>,
    job_node: Node,
) -> Vec<GraphExpectation> {
    let mut expectations = sources
        .values()
        .map(|source| GraphExpectation::Exact(source.node.clone()))
        .collect::<Vec<_>>();
    expectations.extend([
        GraphExpectation::Exact(job_node),
        GraphExpectation::Absent(outcome_node_id(commit.idempotency_key())),
        GraphExpectation::Absent(audit_node_id(commit.idempotency_key())),
    ]);
    for operation in commit.operations() {
        if let StoreBatchOp::Put(record) = operation {
            expectations.push(GraphExpectation::Absent(record_node_id(&record.id)));
        }
    }
    expectations
}
