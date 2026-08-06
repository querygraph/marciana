use std::str::FromStr;

use querygraph_memory::cognition::CognitionOperation;

use crate::FormationProfile;

#[test]
fn profiles_are_closed_versioned_and_bound_to_one_operation() {
    let profiles = [
        (
            FormationProfile::BackgroundDeduplicationV1,
            CognitionOperation::Deduplicate,
        ),
        (
            FormationProfile::BackgroundReconciliationV1,
            CognitionOperation::Reconcile,
        ),
    ];

    for (profile, operation) in profiles {
        assert_eq!(FormationProfile::from_str(profile.as_str()), Ok(profile));
        assert_eq!(profile.operation(), operation);
        assert_eq!(profile.schema_version(), "1");
    }
    assert!(FormationProfile::from_str("conversation-v1").is_err());
}
