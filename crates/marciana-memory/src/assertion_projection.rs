//! Inert Grust mutations for Marciana assertion graph projections.
//!
//! The projection intentionally does not own a graph handle. A caller must add
//! its returned mutations to the existing TypeSec-authorized guarded commit;
//! this module cannot create a second protected-memory write path.

use std::collections::BTreeMap;

use grust_core::prelude::{Edge, GraphMutation, Node, NodeId, Value};
use marciana_ledger::{
    Assertion, AssertionLineage, Confidence, LegacyRelation, TemporalInterval, TransitionEvidence,
};
use sha2::{Digest, Sha256};
use thiserror::Error;
use typesec_memory::StoredRecord;

use crate::RELATES;

pub(crate) const ASSERTION_LABEL: &str = "MemoryAssertion";
const ENTITY_LABEL: &str = "MemoryEntity";
const SUBJECT_EDGE: &str = "ASSERTS_SUBJECT";
const OBJECT_EDGE: &str = "ASSERTS_OBJECT";

/// Fixed errors while preparing a graph projection from a validated assertion.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AssertionProjectionError {
    #[error("assertion projection encoding failed")]
    Encoding,
    #[error("legacy assertion migration input is invalid")]
    LegacyInput,
}

/// Produces the immutable assertion node and its subject/object graph links.
/// The stable edge identifiers prevent two equal structural triples from
/// overwriting each other.
///
/// # Errors
///
/// Returns [`AssertionProjectionError::Encoding`] if the durable assertion
/// payload cannot be encoded.
pub fn project_assertion(
    assertion: &Assertion,
) -> Result<Vec<GraphMutation>, AssertionProjectionError> {
    let assertion_id = assertion_node_id(assertion);
    let subject_id = entity_node_id(assertion.subject());
    let object_id = entity_node_id(assertion.object());
    let payload =
        serde_json::to_value(assertion).map_err(|_| AssertionProjectionError::Encoding)?;

    Ok(vec![
        GraphMutation::UpsertNode(Node::new(
            ASSERTION_LABEL,
            assertion_id.clone(),
            BTreeMap::from([("assertion".into(), Value::Json(payload))]),
        )),
        GraphMutation::UpsertNode(entity_node(assertion.subject(), subject_id.clone())),
        GraphMutation::UpsertNode(entity_node(assertion.object(), object_id.clone())),
        GraphMutation::UpsertEdge(
            Edge::new(
                SUBJECT_EDGE,
                assertion_id.clone(),
                subject_id,
                BTreeMap::new(),
            )
            .with_id(format!("assertion-subject:{}", assertion.id())),
        ),
        GraphMutation::UpsertEdge(
            Edge::new(OBJECT_EDGE, assertion_id, object_id, BTreeMap::new())
                .with_id(format!("assertion-object:{}", assertion.id())),
        ),
    ])
}

/// Converts one legacy `RELATES` edge plus its trusted source record into an
/// inert assertion projection. Repeating the conversion yields the same node
/// and edge identities, so a guarded migration retry is idempotent.
///
/// # Errors
///
/// Returns [`AssertionProjectionError::LegacyInput`] for a malformed legacy
/// edge or inconsistent source record, without returning its values.
pub fn project_legacy_relation(
    edge: &Edge,
    record: &StoredRecord,
) -> Result<Vec<GraphMutation>, AssertionProjectionError> {
    let (subject, predicate, object) = legacy_terms(edge, record)?;
    let key = digest(
        "querygraph.marciana.legacy-relation-key.v1",
        &[
            record.id.as_str().as_bytes(),
            subject.as_bytes(),
            predicate.as_bytes(),
            object.as_bytes(),
            edge.id
                .as_ref()
                .map_or(&[][..], |id| id.as_str().as_bytes()),
        ],
    );
    let evidence = digest(
        "querygraph.marciana.legacy-relation-evidence.v1",
        &[key.as_bytes(), record.id.as_str().as_bytes()],
    );
    let lineage = AssertionLineage::new(
        format!("legacy-record:{}", record.id.as_str()),
        record.id.as_str(),
        "legacy-relates-v1",
        "assertion-v1",
    )
    .map_err(|_| AssertionProjectionError::LegacyInput)?;
    let relation = LegacyRelation::new(
        key,
        subject,
        predicate,
        object,
        Confidence::from_basis_points(Confidence::MAX)
            .map_err(|_| AssertionProjectionError::LegacyInput)?,
        record.observed_at,
        record.observed_at,
        TemporalInterval::new(record.valid_from, record.invalid_at)
            .map_err(|_| AssertionProjectionError::LegacyInput)?,
        lineage,
        TransitionEvidence::import(vec![evidence])
            .map_err(|_| AssertionProjectionError::LegacyInput)?,
    )
    .map_err(|_| AssertionProjectionError::LegacyInput)?;
    project_assertion(
        &relation
            .migrate()
            .map_err(|_| AssertionProjectionError::LegacyInput)?,
    )
}

fn assertion_node_id(assertion: &Assertion) -> NodeId {
    NodeId::from(format!("assertion:{}", assertion.id()).as_str())
}

pub(crate) fn decode_assertion_node(node: &Node) -> Result<Assertion, AssertionProjectionError> {
    if node.label.as_str() != ASSERTION_LABEL {
        return Err(AssertionProjectionError::LegacyInput);
    }
    let Some(Value::Json(payload)) = node.props.get("assertion") else {
        return Err(AssertionProjectionError::LegacyInput);
    };
    serde_json::from_value(payload.clone()).map_err(|_| AssertionProjectionError::LegacyInput)
}

fn entity_node_id(name: &str) -> NodeId {
    NodeId::from(format!("ent:{name}").as_str())
}

fn entity_node(name: &str, id: NodeId) -> Node {
    Node::new(
        ENTITY_LABEL,
        id,
        BTreeMap::from([("name".into(), Value::String(name.into()))]),
    )
}

fn legacy_terms<'a>(
    edge: &'a Edge,
    record: &StoredRecord,
) -> Result<(&'a str, &'a str, &'a str), AssertionProjectionError> {
    if edge.label.as_str() != RELATES
        || edge.props.get("fact_id") != Some(&Value::String(record.id.as_str().into()))
    {
        return Err(AssertionProjectionError::LegacyInput);
    }
    let subject = edge
        .from
        .as_str()
        .strip_prefix("ent:")
        .ok_or(AssertionProjectionError::LegacyInput)?;
    let object = edge
        .to
        .as_str()
        .strip_prefix("ent:")
        .ok_or(AssertionProjectionError::LegacyInput)?;
    let predicate = match edge.props.get("rel") {
        Some(Value::String(value)) => value.as_str(),
        _ => return Err(AssertionProjectionError::LegacyInput),
    };
    Ok((subject, predicate, object))
}

fn digest(domain: &str, values: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    hasher.update([0]);
    for value in values {
        hasher.update((value.len() as u64).to_be_bytes());
        hasher.update(value);
    }
    format!("sha256:{:x}", hasher.finalize())
}
