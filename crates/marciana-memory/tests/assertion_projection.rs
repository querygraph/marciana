use chrono::{TimeZone, Utc};
use grust_core::prelude::GraphMutation;
use marciana_ledger::{Assertion, AssertionId, AssertionLineage, Confidence, TemporalInterval};
use querygraph_memory::assertion_projection::project_assertion;

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
