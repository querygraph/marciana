use marciana_cognition::{MetricsSnapshot, OperationKind, SloError, SloPolicy};

fn policy() -> SloPolicy {
    SloPolicy::new([100, 100, 100, 100, 100], [500; 5]).expect("policy")
}

#[test]
fn slo_evaluation_is_operation_scoped_and_conservative() {
    let report = policy().evaluate(MetricsSnapshot {
        counts: [0, 4, 0, 1, 0],
        denials: [0, 1, 0, 0, 0],
        total_latency_micros: [0, 0, 0, 0, 0],
        max_latency_micros: [0, 99, 0, 101, 0],
    });
    assert!(!report.is_compliant());
    assert_eq!(
        report.operation(OperationKind::Recall),
        (true, false, 99, 2500)
    );
    assert_eq!(
        report.operation(OperationKind::Forget),
        (true, false, 101, 0)
    );
    assert_eq!(
        report.operation(OperationKind::Remember),
        (false, true, 0, 0)
    );
}

#[test]
fn slo_evaluation_accepts_empty_traffic_and_rounds_denials_up() {
    let report = policy().evaluate(MetricsSnapshot {
        counts: [0, 3, 0, 0, 0],
        denials: [0, 1, 0, 0, 0],
        total_latency_micros: [0; 5],
        max_latency_micros: [0; 5],
    });
    assert_eq!(report.denial_basis_points[1], 3334);
    assert!(!report.compliant[1]);
    assert!(report.compliant[0]);
}

#[test]
fn slo_policy_rejects_unsafe_targets() {
    assert_eq!(
        SloPolicy::new([0; 5], [0; 5]).expect_err("latency target"),
        SloError::InvalidLatencyTarget
    );
    assert_eq!(
        SloPolicy::new([1; 5], [10_001; 5]).expect_err("denial target"),
        SloError::InvalidDenialTarget
    );
}
