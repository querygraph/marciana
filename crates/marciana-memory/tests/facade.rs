use grust_memory::MemoryGraphStore;
use querygraph_memory::{
    ForgetRequest, GraphStoreMemoryStore, MemoryFacade, RecallRequest, RememberRequest,
};
use typesec_core::policy::{MintOptions, mint_capability_for_id};
use typesec_core::{CanDelete, CanRead, CanWrite, Capability, Resource};
use typesec_memory::{MemorySpace, MemoryVault};

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
    let tombstone = facade
        .forget(
            &delete,
            ForgetRequest {
                space_id: "memory/user:alice/semantic".into(),
                memory_ids: vec![id.as_str().into()],
                purpose: "research".into(),
            },
        )
        .unwrap();
    assert_eq!(tombstone.forgotten, vec![id]);
}
