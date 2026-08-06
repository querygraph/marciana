use marciana_cognition::{CostError, CostRates, CostSample, OperationKind, TenantCostAccounting};

fn rates() -> CostRates {
    CostRates {
        source_record: 2,
        output_record: 3,
        input_byte: 5,
        output_byte: 7,
        compute_microsecond: 11,
    }
}

#[test]
fn cost_accounting_is_tenant_scoped_and_deterministic() {
    let mut meter = TenantCostAccounting::new("tenant:coffee".into(), rates()).expect("meter");
    meter.record(CostSample {
        operation: OperationKind::Improve,
        source_records: 2,
        output_records: 1,
        input_bytes: 10,
        output_bytes: 4,
        compute_microseconds: 3,
    });
    let snapshot = meter.snapshot();
    assert_eq!(snapshot.tenant_id, "tenant:coffee");
    assert_eq!(snapshot.samples, [0, 0, 1, 0, 0]);
    assert_eq!(snapshot.source_records[2], 2);
    assert_eq!(
        snapshot.microcredits[2],
        2 * 2 + 3 + 10 * 5 + 4 * 7 + 3 * 11
    );
}

#[test]
fn cost_accounting_separates_operations_and_saturates() {
    let mut meter = TenantCostAccounting::new(
        "tenant:coffee".into(),
        CostRates {
            source_record: u64::MAX,
            output_record: u64::MAX,
            input_byte: u64::MAX,
            output_byte: u64::MAX,
            compute_microsecond: u64::MAX,
        },
    )
    .expect("meter");
    meter.record(CostSample {
        operation: OperationKind::Recall,
        source_records: u32::MAX,
        output_records: u32::MAX,
        input_bytes: u64::MAX,
        output_bytes: u64::MAX,
        compute_microseconds: u64::MAX,
    });
    meter.record(CostSample {
        operation: OperationKind::Forget,
        source_records: 1,
        output_records: 0,
        input_bytes: 0,
        output_bytes: 0,
        compute_microseconds: 0,
    });
    let snapshot = meter.snapshot();
    assert_eq!(snapshot.samples, [0, 1, 0, 1, 0]);
    assert_eq!(snapshot.microcredits[1], u128::MAX);
    assert_eq!(snapshot.source_records[3], 1);
}

#[test]
fn cost_accounting_rejects_invalid_tenant_without_echoing_it() {
    let error = TenantCostAccounting::new("tenant with plaintext".into(), rates())
        .expect_err("invalid tenant");
    assert_eq!(error, CostError::InvalidTenant);
}
