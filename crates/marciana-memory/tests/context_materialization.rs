use chrono::Utc;
use grust_memory::MemoryGraphStore;
use querygraph_memory::GraphStoreMemoryStore;
use querygraph_memory::context::{
    ContextCandidate, ContextView, RecallIntent, materialize_context_plan, plan_context,
};
use querygraph_memory::context_render::{render_text, render_xml};
use sha2::Digest;
use typesec_core::policy::{MintOptions, RequestContext, mint_capability_for_id};
use typesec_core::{CanRead, Capability, Resource};
use typesec_memory::{Label, MemorySpace, MemoryStore, MemoryVault};

const POLICY: &str = r#"
roles:
  - name: reader
    permissions: [read]
    resources: ["memory/user:alice/**"]
assignments:
  - subject: "agent:reader"
    roles: [reader]
"#;

fn digest(value: &str) -> String {
    format!("sha256:{:x}", sha2::Sha256::digest(value.as_bytes()))
}

#[test]
fn materialization_reuses_the_vault_gate_and_reports_redactions() {
    let store = GraphStoreMemoryStore::new(MemoryGraphStore::default());
    let public = support_record("mem-public", "public fact", "public");
    let internal = support_record("mem-internal", "private fact", "internal");
    store.put(public).unwrap();
    store.put(internal).unwrap();
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
    let plan = plan_context(
        RecallIntent {
            query_digest: digest("query"),
            view: ContextView::Episodes,
            as_of: Utc::now(),
            token_budget: 20,
        },
        vec![
            ContextCandidate {
                id: typesec_memory::MemoryId::from_string("mem-public"),
                score_basis_points: 10,
                estimated_tokens: 2,
                reason_digest: digest("public"),
            },
            ContextCandidate {
                id: typesec_memory::MemoryId::from_string("mem-internal"),
                score_basis_points: 9,
                estimated_tokens: 2,
                reason_digest: digest("internal"),
            },
        ],
    )
    .unwrap();
    let bundle = materialize_context_plan(
        &vault,
        &space,
        &capability,
        &plan,
        Label::Public,
        &RequestContext::new(),
    )
    .unwrap();
    assert_eq!(bundle.plan_digest, plan.plan_digest);
    assert_eq!(bundle.memories.len(), 1);
    assert_eq!(bundle.redacted.len(), 1);
    assert_eq!(bundle.memories[0].content.text, "public fact");
    assert!(render_text(&bundle).unwrap().contains("public fact"));
    assert!(render_text(&bundle).unwrap().contains("<redacted>"));
    assert!(render_xml(&bundle).unwrap().contains("redacted=\"true\""));
    assert_eq!(bundle.citations().len(), 2);
}

fn support_record(id: &str, text: &str, label: &str) -> typesec_memory::StoredRecord {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "space_id": "memory/user:alice/semantic",
        "kind": "semantic",
        "label": label,
        "quarantined": false,
        "entities": [],
        "provenance": { "source": "operator" },
        "observed_at": Utc::now(),
        "valid_from": Utc::now(),
        "invalid_at": null,
        "expires_at": null,
        "purposes": [],
        "content": { "text": text }
    }))
    .unwrap()
}
