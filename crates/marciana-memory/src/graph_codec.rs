//! Canonical node identity and record codec for the memory graph shape.
//!
//! One authoritative implementation of the `rec:`/`ent:` node-id scheme and
//! of [`StoredRecord`] round-tripping through a single opaque JSON property;
//! every store path and commit source uses these helpers.

use std::collections::BTreeMap;

use grust_core::prelude::{Node, NodeId, Value};
use typesec_memory::{MemoryId, StoreError, StoredRecord};

use crate::RECORD_LABEL;

pub(crate) fn record_node_id(id: &MemoryId) -> NodeId {
    NodeId::from(format!("rec:{}", id.as_str()).as_str())
}

pub(crate) fn record_id_from_node(node: &NodeId) -> Option<MemoryId> {
    node.as_str()
        .strip_prefix("rec:")
        .map(MemoryId::from_string)
}

pub(crate) fn entity_node_id(name: &str) -> NodeId {
    NodeId::from(format!("ent:{name}").as_str())
}

pub(crate) fn encode_record(record: &StoredRecord) -> Result<Node, StoreError> {
    let json = serde_json::to_value(record)
        .map_err(|err| StoreError::Backend(format!("record serialization failed: {err}")))?;
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("record".into(), Value::Json(json));
    props.insert("space".into(), Value::String(record.space_id.clone()));
    Ok(Node::new(RECORD_LABEL, record_node_id(&record.id), props))
}

pub(crate) fn decode_record(node: &Node) -> Result<StoredRecord, StoreError> {
    match node.props.get("record") {
        Some(Value::Json(json)) => serde_json::from_value(json.clone())
            .map_err(|err| StoreError::Backend(format!("record deserialization failed: {err}"))),
        _ => Err(StoreError::Backend(format!(
            "node {} has no record payload",
            node.id.as_str()
        ))),
    }
}
