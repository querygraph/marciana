//! # querygraph-memory
//!
//! Marciana's Grust adapter: [`typesec-memory`](typesec_memory)'s `MemoryStore`
//! implemented over a compatible Grust [`GraphMutationStore`]. Records and
//! their entity knowledge graph are written *incrementally* as nodes and
//! edges. The v1 durable production path is [`TursoMemoryStore`], while the
//! generic adapter remains useful for reference and explicitly integrated
//! backends.
//!
//! ## Graph shape
//!
//! ```text
//! (:MemoryRecord {record: <json>})-[:MENTIONS]->(:MemoryEntity {name, kind})
//! (:MemoryEntity)-[:RELATES {rel, fact_id}]->(:MemoryEntity)
//! ```
//!
//! The full [`StoredRecord`] rides in one JSON property. Space scoping is
//! pushed into the backend; the remaining dimensions use the shared
//! [`StoreQuery::matches`] semantics. Fuller GQL pushdown is a post-v1
//! optimization: correctness is pinned by the conformance suite today.
//!
//! ## The sync/async bridge
//!
//! `MemoryStore` is synchronous by design (FABLE-MEMORY-1 §5.1); Grust's
//! `GraphStore` is async. This crate owns the one sanctioned bridge: a
//! dedicated current-thread runtime, driven directly when no runtime is on
//! the calling thread and from a scoped thread when one is (so calling the
//! vault from inside tokio — e.g. an MCP server — cannot panic).
//!
//! ## Security posture
//!
//! The store adapter round-trips [`StoredRecord`] as an opaque value; it does
//! not inspect protected content to make authorization decisions. Cognition
//! receives only the transient [`typesec_memory::AuthorizedCognitionInput`]
//! that the capability-gated vault already released, and it returns inert
//! proposals to that vault. Passing `typesec_memory::conformance` is the
//! storage compatibility bar.

use std::collections::{BTreeMap, HashSet};
use std::future::Future;

use chrono::{DateTime, Utc};
use grust_core::prelude::{
    Direction, Edge, EdgeQuery, GraphMutation, GraphMutationStore, Node, NodeId, Start, Step,
    Traversal, Value,
};
use typesec_memory::{MemoryId, MemoryStore, StoreBatchOp, StoreError, StoreQuery, StoredRecord};

use marciana_ledger::AssertionQuery;

use bridge::Bridge;
use graph_codec::{
    decode_record, encode_record, entity_node_id, record_id_from_node, record_node_id,
};

const RECORD_LABEL: &str = "MemoryRecord";
const ENTITY_LABEL: &str = "MemoryEntity";
const MENTIONS: &str = "MENTIONS";
pub(crate) const RELATES: &str = "RELATES";

/// `typesec-memory`'s `MemoryStore` over any Grust [`GraphMutationStore`].
pub struct GraphStoreMemoryStore<G: GraphMutationStore> {
    graph: G,
    bridge: Bridge,
}

impl<G: GraphMutationStore> GraphStoreMemoryStore<G> {
    /// Wrap a Grust backend.
    pub fn new(graph: G) -> Self {
        Self {
            graph,
            bridge: Bridge::new(),
        }
    }

    /// Borrow the underlying Grust store.
    pub fn graph(&self) -> &G {
        &self.graph
    }

    /// Return deterministic source-record identifiers selected by assertion
    /// state. This ranking step never returns protected record content.
    ///
    /// # Errors
    ///
    /// Returns a fixed backend error if an assertion projection is malformed
    /// or cannot be read.
    pub fn assertion_candidate_ids(
        &self,
        query: &AssertionQuery,
    ) -> Result<Vec<MemoryId>, StoreError> {
        let nodes = self
            .run(self.graph.traverse(Traversal {
                start: Start::NodesByLabel(assertion_projection::ASSERTION_LABEL.into()),
                steps: Vec::new(),
                limit: None,
            }))
            .map_err(|_| assertion_candidate_error())?;
        let assertions = nodes
            .iter()
            .map(assertion_projection::decode_assertion_node)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| assertion_candidate_error())?;
        let mut ids = Vec::new();
        let mut members = HashSet::new();
        for assertion in query.select(&assertions) {
            let id = MemoryId::from_string(assertion.lineage().source_record_id().to_owned());
            if members.insert(id.clone()) {
                ids.push(id);
            }
        }
        Ok(ids)
    }

    fn run<T: Send>(
        &self,
        fut: impl Future<Output = grust_core::prelude::Result<T>> + Send,
    ) -> Result<T, StoreError> {
        self.bridge
            .run(fut)
            .map_err(|err| StoreError::Backend(err.to_string()))
    }

    fn fetch(&self, id: &MemoryId) -> Result<Option<StoredRecord>, StoreError> {
        let node = self.run(self.graph.get_node(&record_node_id(id)))?;
        node.map(|node| decode_record(&node)).transpose()
    }

    /// Reachable entity node ids within `hops` RELATES steps (both directions).
    fn reachable_entities(&self, entity: &str, hops: u8) -> Result<Vec<NodeId>, StoreError> {
        let start = entity_node_id(entity);
        let mut seen: Vec<NodeId> = vec![start.clone()];
        let mut members: HashSet<NodeId> = seen.iter().cloned().collect();
        for k in 1..=hops {
            let step = Step {
                direction: Direction::Both,
                edge: Some(RELATES.into()),
                node: None,
            };
            let traversal = Traversal {
                start: Start::Node(start.clone()),
                steps: vec![step; usize::from(k)],
                limit: None,
            };
            for node in self.run(self.graph.traverse(traversal))? {
                if members.insert(node.id.clone()) {
                    seen.push(node.id);
                }
            }
        }
        Ok(seen)
    }

    /// The graph mutations that persist one record: the record node plus its
    /// entity nodes and MENTIONS edges. Shared by `put` and `apply_batch`.
    fn record_mutations(record: &StoredRecord) -> Result<Vec<GraphMutation>, StoreError> {
        let mut muts = vec![GraphMutation::UpsertNode(encode_record(record)?)];
        for entity in &record.entities {
            let mut props: BTreeMap<String, Value> = BTreeMap::new();
            props.insert("name".into(), Value::String(entity.name.clone()));
            props.insert("kind".into(), Value::String(entity.kind.clone()));
            muts.push(GraphMutation::UpsertNode(Node::new(
                ENTITY_LABEL,
                entity_node_id(&entity.name),
                props,
            )));
            muts.push(GraphMutation::UpsertEdge(Edge::new(
                MENTIONS,
                record_node_id(&record.id),
                entity_node_id(&entity.name),
                BTreeMap::new(),
            )));
        }
        Ok(muts)
    }
}

impl<G: GraphMutationStore> MemoryStore for GraphStoreMemoryStore<G> {
    fn put(&self, record: StoredRecord) -> Result<(), StoreError> {
        let muts = Self::record_mutations(&record)?;
        self.run(self.graph.apply_mutations(&muts))
    }

    fn get(&self, id: &MemoryId) -> Result<Option<StoredRecord>, StoreError> {
        self.fetch(id)
    }

    /// Apply the whole batch as one `apply_mutations` call — atomic on any
    /// backend whose `GraphMutationStore` overrides it transactionally (the
    /// consolidation path relies on this so a supersede never half-commits).
    /// An `Invalidate` reads the record, stamps `invalid_at`, and re-upserts
    /// it; that read is outside the mutation set, but the capability that
    /// authorized the consolidation is the logical lock.
    fn apply_batch(&self, ops: Vec<StoreBatchOp>) -> Result<(), StoreError> {
        let mut muts: Vec<GraphMutation> = Vec::new();
        for op in ops {
            match op {
                StoreBatchOp::Put(record) => muts.extend(Self::record_mutations(&record)?),
                StoreBatchOp::Invalidate { id, at } => {
                    let mut record = self
                        .fetch(&id)?
                        .ok_or_else(|| StoreError::Backend(format!("no record {id}")))?;
                    record.invalid_at = Some(at);
                    muts.push(GraphMutation::UpsertNode(encode_record(&record)?));
                }
            }
        }
        self.run(self.graph.apply_mutations(&muts))
    }

    fn query(&self, query: &StoreQuery) -> Result<Vec<StoredRecord>, StoreError> {
        // Pushdown: the space filter travels to the backend as a property
        // predicate (record nodes carry `space` as a plain prop), so a
        // space-scoped query never scans other tenants' records. Remaining
        // dimensions filter through the shared `StoreQuery::matches`
        // semantics the conformance suite pins.
        let start = match &query.space_id {
            Some(space) => Start::NodesByProperty {
                label: RECORD_LABEL.into(),
                key: "space".into(),
                value: Value::String(space.clone()),
            },
            None => Start::NodesByLabel(RECORD_LABEL.into()),
        };
        let nodes = self.run(self.graph.traverse(Traversal {
            start,
            steps: Vec::new(),
            limit: None,
        }))?;
        let mut out = Vec::new();
        for node in nodes {
            let record = decode_record(&node)?;
            if query.matches(&record) {
                out.push(record);
            }
        }
        // Same deterministic order as the reference stores.
        out.sort_by(|a, b| b.observed_at.cmp(&a.observed_at).then(b.id.cmp(&a.id)));
        if let Some(limit) = query.limit {
            out.truncate(limit);
        }
        Ok(out)
    }

    fn invalidate(&self, id: &MemoryId, at: DateTime<Utc>) -> Result<(), StoreError> {
        let mut record = self
            .fetch(id)?
            .ok_or_else(|| StoreError::Backend(format!("no record {id}")))?;
        record.invalid_at = Some(at);
        self.run(self.graph.put_node(&encode_record(&record)?))?;
        Ok(())
    }

    fn tombstone(&self, id: &MemoryId) -> Result<bool, StoreError> {
        let existed = self.fetch(id)?.is_some();
        if !existed {
            return Ok(false);
        }
        // delete_node removes the record and its incident MENTIONS edges;
        // RELATES edges asserted by this record are pruned by fact_id.
        self.run(self.graph.delete_node(&record_node_id(id)))?;
        let relates = self.run(self.graph.get_edges(EdgeQuery {
            from: None,
            to: None,
            label: Some(RELATES.into()),
        }))?;
        for edge in relates {
            if edge.props.get("fact_id") == Some(&Value::String(id.as_str().to_string())) {
                self.run(self.graph.delete_edge(&edge.from, &edge.label, &edge.to))?;
            }
        }
        Ok(true)
    }

    fn link(&self, from: &str, rel: &str, to: &str, record: &MemoryId) -> Result<(), StoreError> {
        for name in [from, to] {
            let mut props: BTreeMap<String, Value> = BTreeMap::new();
            props.insert("name".into(), Value::String(name.to_string()));
            self.run(
                self.graph
                    .put_node(&Node::new(ENTITY_LABEL, entity_node_id(name), props)),
            )?;
        }
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("rel".into(), Value::String(rel.to_string()));
        props.insert("fact_id".into(), Value::String(record.as_str().to_string()));
        self.run(self.graph.put_edge(&Edge::new(
            RELATES,
            entity_node_id(from),
            entity_node_id(to),
            props,
        )))?;
        Ok(())
    }

    fn neighborhood(&self, entity: &str, hops: u8) -> Result<Vec<MemoryId>, StoreError> {
        let mut out: Vec<MemoryId> = Vec::new();
        let mut members: HashSet<MemoryId> = HashSet::new();
        for entity_node in self.reachable_entities(entity, hops)? {
            // Records that mention this entity: one In step over MENTIONS.
            let mentions = self.run(self.graph.traverse(Traversal {
                start: Start::Node(entity_node),
                steps: vec![Step {
                    direction: Direction::In,
                    edge: Some(MENTIONS.into()),
                    node: Some(RECORD_LABEL.into()),
                }],
                limit: None,
            }))?;
            for node in mentions {
                if let Some(id) = record_id_from_node(&node.id)
                    && members.insert(id.clone())
                {
                    out.push(id);
                }
            }
        }
        Ok(out)
    }
}

fn assertion_candidate_error() -> StoreError {
    StoreError::Backend("assertion candidate lookup failed".into())
}

pub mod analytics;
mod api;
pub mod assertion_projection;
pub mod assertion_recall;
mod bridge;
pub mod cognition;
pub mod context;
pub mod context_render;
pub mod evaluation;
pub mod evaluation_receipt;
mod facade;
mod graph_codec;
mod legacy_migration;
pub mod session;
#[cfg(feature = "turso")]
pub mod turso;
pub mod vector;
pub mod vector_manifest;
mod vector_manifest_store;
pub use api::{
    ApiError, ForgetRequest, ImproveRequest, MemoryVerb, RecallRequest, RememberRequest,
};
pub use evaluation::{
    ContextEvaluationCase, ContextEvaluationCorpus, ContextEvaluationReport,
    ContextEvaluationSummary, EvaluationError,
};
pub use evaluation_receipt::{ContextEvaluationReceipt, EvaluationReceiptError};
pub use facade::{FacadeError, MemoryFacade};
pub use legacy_migration::AssertionMigrationReport;
pub use session::{RecallContextMetadata, SessionError, SessionMetadata, ThreadMetadata};
#[cfg(feature = "turso")]
pub use turso::TursoMemoryStore;
pub use vector::{Embedder, TenantIndexError, TenantVectorIndex, VectorIndex, VectorIndexScope};
pub use vector_manifest::{
    VectorIndexManifest, VectorManifestError, VectorRepairBatch, VectorRepairOperation,
};

#[cfg(test)]
mod tests;
