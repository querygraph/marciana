use marciana_cognition::LatencySamples;

#[test]
fn latency_percentiles_are_nearest_rank_and_order_stable() {
    let mut samples = LatencySamples::default();
    for value in [30, 10, 90, 20, 50] {
        samples.record(value).unwrap();
    }
    let first = samples.snapshot();
    assert_eq!(first.count, 5);
    assert_eq!(first.p50_micros, 30);
    assert_eq!(first.p95_micros, 90);
    assert_eq!(first.p99_micros, 90);
    samples.record(1).unwrap();
    assert_ne!(first.digest, samples.snapshot().digest);
}

#[test]
fn empty_latency_snapshot_is_content_free_and_stable() {
    let first = LatencySamples::default().snapshot();
    let second = LatencySamples::default().snapshot();
    assert_eq!(first, second);
    assert_eq!(first.count, 0);
    assert_eq!(first.p99_micros, 0);
    assert!(first.digest.starts_with("sha256:"));
}
