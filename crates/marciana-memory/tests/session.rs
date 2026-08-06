use chrono::{TimeZone, Utc};
use querygraph_memory::context::{ContextRecipe, ContextView, RecallIntent};
use querygraph_memory::session::{SessionError, SessionMetadata};

fn digest(label: &str) -> String {
    let marker = if label == "policy" { "b" } else { "a" };
    format!("sha256:{}", marker.repeat(64))
}

fn intent() -> RecallIntent {
    RecallIntent {
        query_digest: digest("query"),
        working_set_digest: None,
        pinned_memory_ids: Vec::new(),
        view: ContextView::Assertions,
        recipe: ContextRecipe::Ranked,
        as_of: Utc.timestamp_opt(10, 0).unwrap(),
        token_budget: 16,
    }
}

#[test]
fn session_metadata_is_bounded_and_stable() {
    let first =
        SessionMetadata::new("session-1".into(), "tenant/space".into(), digest("policy")).unwrap();
    let second =
        SessionMetadata::new("session-1".into(), "tenant/space".into(), digest("policy")).unwrap();
    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.session_id(), "session-1");
    assert_eq!(first.space_id(), "tenant/space");
    assert!(matches!(
        SessionMetadata::new("".into(), "space".into(), digest("policy")),
        Err(SessionError::Invalid)
    ));
}

#[test]
fn binding_changes_plan_identity_without_granting_authority() {
    let session =
        SessionMetadata::new("session-1".into(), "space".into(), digest("policy")).unwrap();
    let bound = session.bind_intent(intent()).unwrap();
    assert_ne!(bound.query_digest, digest("query"));
    assert!(bound.validate().is_ok());
    let other = SessionMetadata::new("session-2".into(), "space".into(), digest("policy")).unwrap();
    assert_ne!(
        bound.query_digest,
        other.bind_intent(intent()).unwrap().query_digest
    );
}
