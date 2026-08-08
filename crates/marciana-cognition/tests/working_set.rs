use chrono::{TimeZone, Utc};
use marciana_cognition::{WorkingSet, WorkingSetSlot, WorkingSetSource, WorkingSetStatus};
use querygraph_memory::context::{ContextRecipe, ContextView};
use sha2::Digest;
use typesec_memory::MemoryId;

fn digest(value: &str) -> String {
    format!("sha256:{:x}", sha2::Sha256::digest(value.as_bytes()))
}

#[test]
fn working_set_requires_approval_before_activation_and_only_compiles_intent() {
    let mut set = WorkingSet::propose(
        "memory/user:alice/semantic".into(),
        digest("policy"),
        ContextView::Episodes,
        ContextRecipe::Recent,
        128,
        vec![WorkingSetSlot {
            memory_id: MemoryId::from_string("memory-1"),
        }],
        WorkingSetSource::AgentProposal,
    )
    .expect("proposal");
    assert_eq!(
        set.working_set_digest,
        "sha256:bb264e511b1a5eddd98236676ae1990f99ee32acf20d60b0fe873221d4a11ef0"
    );
    assert_eq!(set.status, WorkingSetStatus::Proposed);
    assert!(set.activate().is_err());
    set.approve().expect("approval");
    set.activate().expect("activation");
    let intent = set
        .recall_intent(digest("coffee"), Utc.timestamp_opt(10, 0).unwrap())
        .expect("intent");
    assert_eq!(intent.token_budget, 128);
    assert_eq!(
        intent.pinned_memory_ids,
        vec![MemoryId::from_string("memory-1")]
    );
    set.revoke().expect("revoke");
    assert!(set.recall_intent(digest("later"), Utc::now()).is_err());
}

#[test]
fn working_set_rejects_duplicate_slots_and_tampering() {
    let duplicate = WorkingSet::propose(
        "memory/user:alice/semantic".into(),
        digest("policy"),
        ContextView::Assertions,
        ContextRecipe::Ranked,
        128,
        vec![
            WorkingSetSlot {
                memory_id: MemoryId::from_string("same"),
            },
            WorkingSetSlot {
                memory_id: MemoryId::from_string("same"),
            },
        ],
        WorkingSetSource::Operator,
    );
    assert!(duplicate.is_err());

    let mut set = WorkingSet::propose(
        "memory/user:alice/semantic".into(),
        digest("policy"),
        ContextView::Assertions,
        ContextRecipe::Ranked,
        128,
        Vec::new(),
        WorkingSetSource::Operator,
    )
    .expect("proposal");
    set.token_budget = 256;
    assert!(set.validate().is_err());
}
