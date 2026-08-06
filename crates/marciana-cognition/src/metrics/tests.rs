use super::{MetricsSnapshot, OperationKind, OperationMetrics, OperationSample};

#[test]
fn metrics_are_verb_scoped_and_content_free() {
    let mut metrics = OperationMetrics::default();
    metrics.record(OperationSample {
        operation: OperationKind::Recall,
        allowed: true,
        latency_micros: 12,
    });
    metrics.record(OperationSample {
        operation: OperationKind::Recall,
        allowed: false,
        latency_micros: 30,
    });
    metrics.record(OperationSample {
        operation: OperationKind::Forget,
        allowed: true,
        latency_micros: 8,
    });
    assert_eq!(
        metrics.snapshot(),
        MetricsSnapshot {
            counts: [0, 2, 0, 1],
            denials: [0, 1, 0, 0],
            total_latency_micros: [0, 42, 0, 8],
            max_latency_micros: [0, 30, 0, 8],
        }
    );
}
