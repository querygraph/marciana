use chrono::{TimeDelta, TimeZone, Utc};
use marciana_cognition::{OperationKind, QuotaError, QuotaLimits, TenantQuota};

fn limits() -> QuotaLimits {
    QuotaLimits {
        operations: [2, 1, 1, 1, 3],
    }
}

#[test]
fn quota_enforces_boundaries_and_resets_at_window_boundary() {
    let start = Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, 0).unwrap();
    let mut quota = TenantQuota::new("tenant:coffee".into(), start, TimeDelta::hours(1), limits())
        .expect("quota");
    quota.try_consume(OperationKind::Context, 3, start).unwrap();
    assert_eq!(quota.remaining(OperationKind::Context), 0);
    assert_eq!(
        quota.try_consume(OperationKind::Context, 1, start),
        Err(QuotaError::Exceeded)
    );
    quota
        .try_consume(OperationKind::Context, 1, start + TimeDelta::hours(1))
        .unwrap();
    assert_eq!(quota.remaining(OperationKind::Context), 2);
}

#[test]
fn quota_rejects_invalid_amount_clock_and_tenant_without_content() {
    let start = Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, 0).unwrap();
    assert_eq!(
        TenantQuota::new("bad tenant".into(), start, TimeDelta::hours(1), limits())
            .expect_err("invalid tenant"),
        QuotaError::InvalidTenant
    );
    let mut quota =
        TenantQuota::new("tenant:coffee".into(), start, TimeDelta::hours(1), limits()).unwrap();
    assert_eq!(
        quota.try_consume(OperationKind::Recall, 0, start),
        Err(QuotaError::InvalidAmount)
    );
    assert_eq!(
        quota.try_consume(OperationKind::Recall, 1, start - TimeDelta::seconds(1)),
        Err(QuotaError::ClockRegression)
    );
}
