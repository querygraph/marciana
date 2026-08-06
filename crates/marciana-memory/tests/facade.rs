use chrono::Utc;
use grust_memory::MemoryGraphStore;
use querygraph_memory::{
    context::{plan_context, ContextCandidate, ContextRecipe, ContextView, RecallIntent},
    ForgetRequest, GraphStoreMemoryStore, MemoryFacade, RecallRequest, RememberRequest,
};
use sha2::Digest;
use typesec_core::policy::{mint_capability_for_id, MintOptions};
use typesec_core::{CanDelete, CanRead, CanWrite, Capability, Resource};
use typesec_memory::{Label, MemorySpace, MemoryVault};

const POLICY: &str = r#"
roles:
  - name: memory
    permissions: [read, write, delete]
    resources: ["memory/user:alice/**"]
assignments:
  - subject: "agent:memory"
    roles: [memory]
"#;

#[test]
fn facade_executes_all_verbs_through_the_vault() {
    let store = GraphStoreMemoryStore::new(MemoryGraphStore::default());
    let vault = MemoryVault::new(store);
    let space = MemorySpace::new("user:alice", "semantic");
    let policy = typesec_rbac::RbacEngine::from_yaml(POLICY).unwrap();
    let write: Capability<CanWrite, _> = mint_capability_for_id(
        &policy,
        "agent:memory",
        space.resource_id(),
        &MintOptions::default(),
    )
    .unwrap();
    let read: Capability<CanRead, _> = mint_capability_for_id(
        &policy,
        "agent:memory",
        space.resource_id(),
        &MintOptions::default(),
    )
    .unwrap();
    let delete: Capability<CanDelete, _> = mint_capability_for_id(
        &policy,
        "agent:memory",
        space.resource_id(),
        &MintOptions::default(),
    )
    .unwrap();
    let facade = MemoryFacade::new(&vault, &space);
    let id = facade
        .remember(
            &write,
            RememberRequest {
                space_id: "memory/user:alice/semantic".into(),
                text: "Honduras coffee price is 4.20 USD/kg".into(),
                purpose: "research".into(),
            },
        )
        .unwrap();
    let (hits, redacted) = facade
        .recall(
            &read,
            RecallRequest {
                space_id: "memory/user:alice/semantic".into(),
                query: "coffee price".into(),
                purpose: "research".into(),
            },
        )
        .unwrap();
    assert!(redacted.is_empty());
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, id);
    let context_plan = plan_context(
        RecallIntent {
            query_digest: digest("coffee-price"),
            view: ContextView::Episodes,
            recipe: ContextRecipe::Ranked,
            as_of: Utc::now(),
            token_budget: 32,
        },
        vec![ContextCandidate {
            id: id.clone(),
            score_basis_points: 100,
            estimated_tokens: 8,
            reason_digest: digest("query-match"),
        }],
    )
    .unwrap();
    let context_bundle = facade
        .materialize_context(
            &read,
            &context_plan,
            Label::Internal,
            &typesec_core::policy::RequestContext::new().with_purpose("research"),
        )
        .unwrap();
    assert_eq!(context_bundle.memories.len(), 1);
    assert_eq!(context_bundle.memories[0].id, id);
    assert_eq!(context_bundle.plan_digest, context_plan.plan_digest);
    let replacement = facade
        .improve(
            &write,
            querygraph_memory::ImproveRequest {
                space_id: "memory/user:alice/semantic".into(),
                memory_id: id.as_str().into(),
                replacement: RememberRequest {
                    space_id: "memory/user:alice/semantic".into(),
                    text: "Honduras coffee price is 4.80 USD/kg".into(),
                    purpose: "research".into(),
                },
            },
        )
        .unwrap();
    let (updated, _) = facade
        .recall(
            &read,
            RecallRequest {
                space_id: "memory/user:alice/semantic".into(),
                query: "coffee price".into(),
                purpose: "research".into(),
            },
        )
        .unwrap();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].id, replacement);
    assert_eq!(
        updated[0].content.text,
        "Honduras coffee price is 4.80 USD/kg"
    );
    let tombstone = facade
        .forget(
            &delete,
            ForgetRequest {
                space_id: "memory/user:alice/semantic".into(),
                memory_ids: vec![replacement.as_str().into()],
                purpose: "research".into(),
            },
        )
        .unwrap();
    assert_eq!(tombstone.forgotten, vec![replacement]);
}

fn digest(value: &str) -> String {
    format!("sha256:{:x}", sha2::Sha256::digest(value.as_bytes()))
}
