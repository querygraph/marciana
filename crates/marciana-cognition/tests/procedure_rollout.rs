use marciana_cognition::{EvaluationReport, Procedure, ProcedureRollout, ProcedureRolloutStatus};
use sha2::Digest;

fn digest(value: &str) -> String {
    format!("sha256:{:x}", sha2::Sha256::digest(value.as_bytes()))
}

fn approved_procedure() -> Procedure {
    let mut procedure = Procedure::propose(digest("procedure")).expect("procedure");
    let report =
        EvaluationReport::new(procedure.procedure_digest.clone(), digest("dataset"), 8_000)
            .expect("report");
    procedure.record_evaluation(&report).expect("evaluation");
    procedure.approve().expect("approval");
    procedure
}

#[test]
fn rollout_requires_active_evaluated_procedure_and_supports_rollback() {
    let mut procedure = approved_procedure();
    let mut rollout = ProcedureRollout::propose(&procedure, digest("cohort"), 2_500, 90)
        .expect("rollout proposal");
    assert_eq!(rollout.status, ProcedureRolloutStatus::Proposed);
    rollout.approve().expect("rollout approval");
    assert!(rollout.activate(&procedure).is_err());
    procedure.activate().expect("procedure activation");
    rollout.activate(&procedure).expect("rollout activation");
    rollout.rollback().expect("rollback");
    assert_eq!(rollout.status, ProcedureRolloutStatus::RolledBack);
}

#[test]
fn rollout_rejects_tampering_and_invalid_retention() {
    let procedure = approved_procedure();
    assert!(ProcedureRollout::propose(&procedure, digest("cohort"), 1, 0).is_err());
    let mut rollout =
        ProcedureRollout::propose(&procedure, digest("cohort"), 1, 30).expect("rollout proposal");
    rollout.traffic_basis_points = 2;
    assert!(rollout.validate().is_err());
}
