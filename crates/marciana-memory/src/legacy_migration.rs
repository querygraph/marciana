//! Trusted-storage migration of legacy `RELATES` edges into assertion
//! projections. This is deployment maintenance, not a public memory
//! mutation API; callers run it under migration authority rather than a
//! request capability.

use grust_core::prelude::{
    Edge, EdgeQuery, GraphCommitStore, GraphMutation, GraphMutationStore, GuardedGraphCommit, Value,
};
use sha2::{Digest, Sha256};
use typesec_memory::{MemoryId, StoreError};

use crate::{GraphStoreMemoryStore, RELATES, assertion_projection};

/// Non-disclosing result of an assertion projection migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssertionMigrationReport {
    /// Number of previously unprojected legacy relations added in this run.
    pub migrated: usize,
}

impl<G: GraphMutationStore> GraphStoreMemoryStore<G> {
    /// Adds assertion projections for legacy `RELATES` edges in one backend
    /// mutation batch.
    ///
    /// # Errors
    ///
    /// Returns a fixed backend error when the legacy graph cannot be read,
    /// validated, or atomically projected. It never includes record or edge
    /// values in the diagnostic.
    pub fn migrate_legacy_assertions(&self) -> Result<AssertionMigrationReport, StoreError>
    where
        G: GraphCommitStore,
    {
        let edges = self
            .run(self.graph.get_edges(EdgeQuery {
                from: None,
                to: None,
                label: Some(RELATES.into()),
            }))
            .map_err(|_| legacy_migration_error())?;
        let mut planned = Vec::new();
        for edge in edges {
            let record_id = legacy_record_id(&edge).ok_or_else(legacy_migration_error)?;
            let record = self
                .fetch(&record_id)
                .map_err(|_| legacy_migration_error())?
                .ok_or_else(legacy_migration_error)?;
            let projection = assertion_projection::project_legacy_relation(&edge, &record)
                .map_err(|_| legacy_migration_error())?;
            let assertion_node = match projection.first() {
                Some(GraphMutation::UpsertNode(node)) => &node.id,
                _ => return Err(legacy_migration_error()),
            };
            if self
                .run(self.graph.get_node(assertion_node))
                .map_err(|_| legacy_migration_error())?
                .is_none()
            {
                planned.push((assertion_node.as_str().to_owned(), projection));
            }
        }
        if planned.is_empty() {
            return Ok(AssertionMigrationReport { migrated: 0 });
        }
        planned.sort_by(|left, right| left.0.cmp(&right.0));
        let request = legacy_migration_digest(
            "querygraph.marciana.assertion-migration.request.v1",
            &planned
                .iter()
                .map(|(id, _)| id.as_bytes())
                .collect::<Vec<_>>(),
        );
        let ledger_key = format!("marciana-assertion-migration:{}", &request[7..]);
        let migrated = planned.len();
        let mutations = planned
            .into_iter()
            .flat_map(|(_, mutations)| mutations)
            .collect();
        let commit = GuardedGraphCommit::new(ledger_key, request, mutations);
        let receipt = self
            .bridge
            .run(self.graph.commit_guarded(&commit))
            .map_err(|_| legacy_migration_error())?;
        Ok(AssertionMigrationReport {
            migrated: if receipt.replayed { 0 } else { migrated },
        })
    }
}

fn legacy_record_id(edge: &Edge) -> Option<MemoryId> {
    match edge.props.get("fact_id") {
        Some(Value::String(id)) => Some(MemoryId::from_string(id.clone())),
        _ => None,
    }
}

pub(crate) fn legacy_migration_error() -> StoreError {
    StoreError::Backend("legacy assertion migration failed".into())
}

fn legacy_migration_digest(domain: &str, values: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    hasher.update([0]);
    for value in values {
        hasher.update((value.len() as u64).to_be_bytes());
        hasher.update(value);
    }
    format!("sha256:{:x}", hasher.finalize())
}
