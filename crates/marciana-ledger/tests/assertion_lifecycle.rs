use chrono::{TimeZone, Utc};
use marciana_ledger::{
    Assertion, AssertionId, AssertionLineage, AssertionQuery, AssertionState, AssertionTransition,
    Confidence, LedgerError, LegacyRelation, TemporalInterval, TransitionEvidence,
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
    assert_eq!(value.state_at(at(2)), AssertionState::Current);
    assert_eq!(value.state_at(at(3)), AssertionState::Disputed);
    assert!(value.is_current_at(at(4)));
    assert!(!value.is_current_at(at(3)));
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
    let too_early = AssertionTransition::new(
        AssertionState::Current,
        AssertionState::Disputed,
        at(0),
        evidence(),
    )
    .unwrap();
    assert_eq!(
        value.apply_transition(too_early),
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
fn temporal_intervals_are_half_open_and_reject_reversal() {
    let interval = TemporalInterval::new(at(2), Some(at(4))).unwrap();
    assert!(interval.contains(at(2)));
    assert!(interval.contains(at(3)));
    assert!(!interval.contains(at(4)));
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

#[test]
fn legacy_migration_is_retry_stable_and_keeps_ended_validity_historical() {
    let migration = || {
        LegacyRelation::new(
            "legacy-edge:record-1:1",
            "account:acme",
            "locatedIn",
            "place:venice",
            Confidence::from_basis_points(10_000).unwrap(),
            at(0),
            at(1),
            TemporalInterval::new(at(0), Some(at(3))).unwrap(),
            AssertionLineage::new(
                "episode:legacy-1",
                "record:1",
                "legacy-relates-v1",
                "assertion-v1",
            )
            .unwrap(),
            TransitionEvidence::import(vec![DIGEST.into()]).unwrap(),
        )
        .unwrap()
    };
    let first = migration().migrate().unwrap();
    let retry = migration().migrate().unwrap();

    assert_eq!(first.id(), retry.id());
    assert_eq!(first.state(), AssertionState::Current);
    assert!(first.is_current_at(at(2)));
    assert!(!first.is_current_at(at(3)));
}

#[test]
fn legacy_import_requires_source_evidence_without_fabricated_assertion_cause() {
    let relation = LegacyRelation::new(
        "legacy-edge:record-1:1",
        "account:acme",
        "locatedIn",
        "place:venice",
        Confidence::from_basis_points(10_000).unwrap(),
        at(0),
        at(1),
        TemporalInterval::new(at(0), None).unwrap(),
        AssertionLineage::new(
            "episode:legacy-1",
            "record:1",
            "legacy-relates-v1",
            "assertion-v1",
        )
        .unwrap(),
        evidence(),
    );

    assert!(matches!(
        relation,
        Err(LedgerError::InvalidTransitionEvidence)
    ));
}

#[test]
fn candidate_queries_are_deterministic_and_keep_history_distinct_from_current_validity() {
    let ended = LegacyRelation::new(
        "legacy-edge:record-1:1",
        "account:acme",
        "locatedIn",
        "place:venice",
        Confidence::from_basis_points(10_000).unwrap(),
        at(0),
        at(1),
        TemporalInterval::new(at(0), Some(at(3))).unwrap(),
        AssertionLineage::new(
            "episode:legacy-1",
            "record:1",
            "legacy-relates-v1",
            "assertion-v1",
        )
        .unwrap(),
        TransitionEvidence::import(vec![DIGEST.into()]).unwrap(),
    )
    .unwrap()
    .migrate()
    .unwrap();
    let active = assertion();
    let assertions = vec![active, ended.clone()];

    assert_eq!(
        AssertionQuery::current_at(at(2)).select(&assertions),
        vec![&ended]
    );
    assert_eq!(
        AssertionQuery::current_at(at(4)).select(&assertions),
        Vec::<&Assertion>::new()
    );
    assert_eq!(
        AssertionQuery::states_at(at(4), [AssertionState::Current])
            .unwrap()
            .select(&assertions),
        vec![&ended]
    );
    assert!(matches!(
        AssertionQuery::states_at(at(4), []),
        Err(LedgerError::InvalidQuery)
    ));
}
