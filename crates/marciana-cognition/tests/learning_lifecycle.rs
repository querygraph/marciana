use chrono::{TimeZone, Utc};
use marciana_cognition::{
    EvaluationReport, FeedbackDataset, FeedbackRecord, Observation, ObservationStatus, Procedure,
    ProcedureStatus,
};
use sha2::Digest;

fn digest(value: &str) -> String {
    format!("sha256:{:x}", sha2::Sha256::digest(value.as_bytes()))
}

#[test]
fn observations_require_evidence_and_follow_a_closed_lifecycle() {
    let at = Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, 0).unwrap();
    let evidence = vec![digest("a"), digest("b")];
    let mut observation = Observation::propose(&evidence, 2, 8_500, at).expect("proposal");
    observation
        .transition(ObservationStatus::Accepted, at)
        .expect("accept");
    observation
        .transition(ObservationStatus::Expired, at)
        .expect("expire");
    assert_eq!(observation.valid_to, Some(at));
    assert!(
        observation
            .transition(ObservationStatus::Accepted, at)
            .is_err()
    );
}

#[test]
fn procedures_cannot_activate_without_evaluation_and_approval() {
    let mut procedure = Procedure::propose(digest("procedure")).expect("procedure");
    assert_eq!(procedure.status, ProcedureStatus::Proposed);
    assert!(procedure.activate().is_err());
    let report =
        EvaluationReport::new(procedure.procedure_digest.clone(), digest("dataset"), 8_000)
            .expect("evaluation report");
    procedure.record_evaluation(&report).expect("evaluate");
    procedure.approve().expect("approve");
    procedure.activate().expect("activate");
    procedure.rollback().expect("rollback");
    assert_eq!(procedure.status, ProcedureStatus::RolledBack);
}

#[test]
fn feedback_dataset_is_order_stable_and_digest_only() {
    let at = Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, 0).unwrap();
    let left = FeedbackDataset::new(vec![
        FeedbackRecord {
            trajectory_digest: digest("b"),
            outcome_basis_points: 7000,
            recorded_at: at,
        },
        FeedbackRecord {
            trajectory_digest: digest("a"),
            outcome_basis_points: 9000,
            recorded_at: at,
        },
    ])
    .unwrap();
    let right = FeedbackDataset::new(vec![
        FeedbackRecord {
            trajectory_digest: digest("a"),
            outcome_basis_points: 9000,
            recorded_at: at,
        },
        FeedbackRecord {
            trajectory_digest: digest("b"),
            outcome_basis_points: 7000,
            recorded_at: at,
        },
    ])
    .unwrap();
    assert_eq!(left.dataset_digest, right.dataset_digest);
}
