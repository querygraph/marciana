//! Inert Grust mutations for Marciana assertion graph projections.
//!
//! The projection intentionally does not own a graph handle. A caller must add
//! its returned mutations to the existing TypeSec-authorized guarded commit;
//! this module cannot create a second protected-memory write path.

use std::collections::BTreeMap;

use grust_core::prelude::{Edge, GraphMutation, Node, NodeId, Value};
use marciana_ledger::Assertion;
use thiserror::Error;

const ASSERTION_LABEL: &str = "MemoryAssertion";
const ENTITY_LABEL: &str = "MemoryEntity";
const SUBJECT_EDGE: &str = "ASSERTS_SUBJECT";
const OBJECT_EDGE: &str = "ASSERTS_OBJECT";

/// Fixed errors while preparing a graph projection from a validated assertion.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AssertionProjectionError {
    #[error("assertion projection encoding failed")]
    Encoding,
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

fn assertion_node_id(assertion: &Assertion) -> NodeId {
    NodeId::from(format!("assertion:{}", assertion.id()).as_str())
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
