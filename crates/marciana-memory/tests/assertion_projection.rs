use chrono::{TimeZone, Utc};
use grust_core::prelude::{Edge, GraphMutation, NodeId, Value};
use marciana_ledger::{Assertion, AssertionId, AssertionLineage, Confidence, TemporalInterval};
use querygraph_memory::assertion_projection::{project_assertion, project_legacy_relation};
use typesec_memory::StoredRecord;

fn assertion() -> Assertion {
    let at = Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, 0).unwrap();
    Assertion::new(
        AssertionId::new(),
        "account:acme",
        "locatedIn",
        "place:venice",
        Confidence::from_basis_points(8_500).unwrap(),
        at,
        at,
        TemporalInterval::new(at, None).unwrap(),
        AssertionLineage::new("episode:1", "record:1", "document-v1", "assertion-v1").unwrap(),
    )
    .unwrap()
}

fn record() -> StoredRecord {
    serde_json::from_value(serde_json::json!({
        "id": "memory-1",
        "space_id": "memory/user:alice/semantic",
        "kind": "semantic",
        "label": "internal",
        "quarantined": false,
        "entities": [],
        "provenance": { "source": "operator" },
        "observed_at": "2026-08-06T12:00:00Z",
        "valid_from": "2026-08-06T12:00:00Z",
        "invalid_at": "2026-08-06T12:00:03Z",
        "expires_at": null,
        "purposes": ["research"],
        "content": { "text": "protected" }
    }))
    .unwrap()
}

fn legacy_edge() -> Edge {
    Edge::new(
        "RELATES",
        NodeId::from("ent:account:acme"),
        NodeId::from("ent:place:venice"),
        std::collections::BTreeMap::from([
            ("rel".into(), Value::String("locatedIn".into())),
            ("fact_id".into(), Value::String("memory-1".into())),
        ]),
    )
}

#[test]
fn equal_structural_triplets_produce_distinct_assertion_nodes_and_edges() {
    let first = assertion();
    let second = assertion();
    let first_mutations = project_assertion(&first).unwrap();
    let second_mutations = project_assertion(&second).unwrap();

    let node_id = |mutations: &[GraphMutation]| match &mutations[0] {
        GraphMutation::UpsertNode(node) => node.id.as_str().to_owned(),
        _ => panic!("first mutation must be the assertion node"),
    };
    let edge_ids = |mutations: &[GraphMutation]| {
        mutations[3..]
            .iter()
            .map(|mutation| match mutation {
                GraphMutation::UpsertEdge(edge) => edge.id.as_ref().unwrap().as_str().to_owned(),
                _ => panic!("assertion links must be edges"),
            })
            .collect::<Vec<_>>()
    };

    assert_ne!(node_id(&first_mutations), node_id(&second_mutations));
    assert_ne!(edge_ids(&first_mutations), edge_ids(&second_mutations));
}

#[test]
fn projection_keeps_the_full_validated_assertion_as_the_single_payload() {
    let value = assertion();
    let mutations = project_assertion(&value).unwrap();
    let GraphMutation::UpsertNode(node) = &mutations[0] else {
        panic!("first mutation must be the assertion node");
    };
    let payload = node.props.get("assertion").unwrap();

    assert!(format!("{payload:?}").contains(value.id().as_str()));
    assert_eq!(mutations.len(), 5);
}

#[test]
fn legacy_relation_migration_is_retry_stable_and_preserves_half_open_validity() {
    let first = project_legacy_relation(&legacy_edge(), &record()).unwrap();
    let retry = project_legacy_relation(&legacy_edge(), &record()).unwrap();
    let GraphMutation::UpsertNode(first_node) = &first[0] else {
        panic!("first mutation must be the assertion node");
    };
    let GraphMutation::UpsertNode(retry_node) = &retry[0] else {
        panic!("first mutation must be the assertion node");
    };

    assert_eq!(first_node.id, retry_node.id);
    let Value::Json(payload) = first_node.props.get("assertion").unwrap() else {
        panic!("assertion payload must be JSON");
    };
    assert_eq!(payload["state"], "current");
    assert_eq!(payload["validity"]["validTo"], "2026-08-06T12:00:03Z");
}
