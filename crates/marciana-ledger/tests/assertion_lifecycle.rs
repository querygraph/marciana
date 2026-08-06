use chrono::{TimeZone, Utc};
use marciana_ledger::{
    Assertion, AssertionId, AssertionLineage, AssertionState, AssertionTransition, Confidence,
    LedgerError, TemporalInterval, TransitionEvidence,
};

const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn at(second: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, second).unwrap()
}

fn evidence() -> TransitionEvidence {
    TransitionEvidence::new(vec![AssertionId::new()], vec![DIGEST.into()]).unwrap()
}

fn assertion() -> Assertion {
    Assertion::new(
        AssertionId::new(),
        "account:acme",
        "locatedIn",
        "place:venice",
        Confidence::from_basis_points(8_500).unwrap(),
        at(0),
        at(1),
        TemporalInterval::new(at(0), None).unwrap(),
        AssertionLineage::new("episode:1", "record:1", "conversation-v1", "assertion-v1").unwrap(),
    )
    .unwrap()
}

#[test]
fn identical_triplets_retain_distinct_assertion_identity_and_lineage() {
    let first = assertion();
    let second = assertion();

    assert_ne!(first.id(), second.id());
    assert_eq!(first.subject(), second.subject());
    assert_eq!(first.lineage().source_record_id(), "record:1");
}

#[test]
fn lifecycle_preserves_dispute_then_current_resurrection_history() {
    let mut value = assertion();
    value
        .apply_transition(
            AssertionTransition::new(
                AssertionState::Proposed,
                AssertionState::Current,
                at(2),
                evidence(),
            )
            .unwrap(),
        )
        .unwrap();
    value
        .apply_transition(
            AssertionTransition::new(
                AssertionState::Current,
                AssertionState::Disputed,
                at(3),
                evidence(),
            )
            .unwrap(),
        )
        .unwrap();
    value
        .apply_transition(
            AssertionTransition::new(
                AssertionState::Disputed,
                AssertionState::Current,
                at(4),
                evidence(),
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(value.state(), AssertionState::Current);
    assert_eq!(value.transitions().len(), 3);
}

#[test]
fn invalid_edges_and_non_monotonic_history_fail_closed() {
    let mut value = assertion();
    assert_eq!(
        AssertionTransition::new(
            AssertionState::Proposed,
            AssertionState::Superseded,
            at(2),
            evidence(),
        ),
        Err(LedgerError::InvalidTransition)
    );

    value
        .apply_transition(
            AssertionTransition::new(
                AssertionState::Proposed,
                AssertionState::Current,
                at(3),
                evidence(),
            )
            .unwrap(),
        )
        .unwrap();
    let transition = AssertionTransition::new(
        AssertionState::Current,
        AssertionState::Disputed,
        at(2),
        evidence(),
    )
    .unwrap();
    assert_eq!(
        value.apply_transition(transition),
        Err(LedgerError::InvalidTransition)
    );
}

#[test]
fn evidence_is_canonical_and_requires_both_causes_and_digests() {
    assert_eq!(
        TransitionEvidence::new(vec![AssertionId::new()], vec!["backend value".into()]),
        Err(LedgerError::InvalidTransitionEvidence)
    );
    assert_eq!(
        TransitionEvidence::new(Vec::new(), vec![DIGEST.into()]),
        Err(LedgerError::InvalidTransitionEvidence)
    );
}

#[test]
fn temporal_intervals_are_inclusive_and_reject_reversal() {
    let interval = TemporalInterval::new(at(2), Some(at(4))).unwrap();
    assert!(interval.contains(at(2)));
    assert!(interval.contains(at(4)));
    assert!(!interval.contains(at(5)));
    assert_eq!(
        TemporalInterval::new(at(4), Some(at(2))),
        Err(LedgerError::InvalidTemporalInterval)
    );
}

#[test]
fn deserialization_cannot_bypass_temporal_or_lifecycle_validation() {
    let mut encoded = serde_json::to_value(assertion()).unwrap();
    encoded["validity"]["validFrom"] = serde_json::json!(at(1));
    encoded["validity"]["validTo"] = serde_json::json!(at(0));
    assert!(serde_json::from_value::<Assertion>(encoded).is_err());

    let mut encoded = serde_json::to_value(assertion()).unwrap();
    encoded["state"] = serde_json::json!("current");
    assert!(serde_json::from_value::<Assertion>(encoded).is_err());
}
