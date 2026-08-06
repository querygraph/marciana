use chrono::{TimeZone, Utc};

use super::{ComponentHealth, ComponentState, HealthSnapshot};

fn component(state: ComponentState) -> ComponentHealth {
    ComponentHealth {
        name: "typesec".into(),
        revision: "14bd5427".into(),
        state,
    }
}

#[test]
fn readiness_is_content_free_and_requires_all_components() {
    let at = Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, 0).unwrap();
    assert!(
        HealthSnapshot::new(at, vec![component(ComponentState::Ready)])
            .unwrap()
            .is_ready()
    );
    assert!(
        !HealthSnapshot::new(at, vec![component(ComponentState::Degraded)])
            .unwrap()
            .is_ready()
    );
    assert!(!HealthSnapshot::new(at, Vec::new()).unwrap().is_ready());
}
