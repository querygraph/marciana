use chrono::{TimeZone, Utc};
use grust_core::prelude::{Edge, GraphMutationStore, NodeId, Value};
use grust_memory::MemoryGraphStore;
use marciana_ledger::AssertionQuery;
use querygraph_memory::{
    GraphStoreMemoryStore, assertion_projection::project_legacy_relation,
    assertion_recall::recall_assertions_at,
};
use typesec_core::policy::{MintOptions, RequestContext, mint_capability_for_id};
use typesec_core::{CanRead, Capability, Resource};
use typesec_memory::{Label, MemorySpace, MemoryStore, MemoryVault, StoredRecord};

const POLICY: &str = r#"
roles:
  - name: reader
    permissions: [read]
    resources: ["memory/user:alice/**"]
assignments:
  - subject: "agent:reader"
    roles: [reader]
"#;

fn at(second: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, second).unwrap()
}

fn record() -> StoredRecord {
    serde_json::from_value(serde_json::json!({
        "id": "memory-1",
        "space_id": "memory/user:alice/semantic",
        "kind": "semantic",
        "label": "public",
        "quarantined": false,
        "entities": [],
        "provenance": { "source": "operator" },
        "observed_at": at(0),
        "valid_from": at(0),
        "invalid_at": null,
        "expires_at": null,
        "purposes": ["research"],
        "content": { "text": "Honduras coffee price is 4.20 USD/kg" }
    }))
    .unwrap()
}

#[test]
fn assertion_candidates_are_materialized_only_through_the_vault() {
    let store = GraphStoreMemoryStore::new(MemoryGraphStore::default());
    let source = record();
    store.put(source.clone()).unwrap();
    let edge = Edge::new(
        "RELATES",
        NodeId::from("ent:coffee:honduras"),
        NodeId::from("ent:price:4.20-usd-kg"),
        std::collections::BTreeMap::from([
            ("rel".into(), Value::String("marketPrice".into())),
            ("fact_id".into(), Value::String(source.id.as_str().into())),
        ]),
    );
    let projection = project_legacy_relation(&edge, &source).unwrap();
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(store.graph().apply_mutations(&projection))
        .unwrap();

    let vault = MemoryVault::new(store);
    let space = MemorySpace::new("user:alice", "semantic");
    let policy = typesec_rbac::RbacEngine::from_yaml(POLICY).unwrap();
    let capability: Capability<CanRead, _> = mint_capability_for_id(
        &policy,
        "agent:reader",
        space.resource_id(),
        &MintOptions::default(),
    )
    .unwrap();
    let query = AssertionQuery::current_at(at(1));

    let (hits, redacted) = recall_assertions_at(
        &vault,
        &space,
        &capability,
        &query,
        at(1),
        Label::Public,
        &RequestContext::new().with_purpose("research"),
    )
    .unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, source.id);
    assert_eq!(hits[0].content.text, "Honduras coffee price is 4.20 USD/kg");
    assert!(redacted.is_empty());
}
