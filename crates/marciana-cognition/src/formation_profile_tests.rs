use std::str::FromStr;

use querygraph_memory::cognition::CognitionOperation;

use crate::{
    FormationCapability, FormationProfile, FormationProvider, FormationRegistry,
    FormationResourceBudget,
};

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
        (
            FormationProfile::ConversationDeduplicationV1,
            CognitionOperation::Deduplicate,
        ),
        (
            FormationProfile::DocumentDeduplicationV1,
            CognitionOperation::Deduplicate,
        ),
        (
            FormationProfile::JsonEventReconciliationV1,
            CognitionOperation::Reconcile,
        ),
        (
            FormationProfile::RawDeduplicationV1,
            CognitionOperation::Deduplicate,
        ),
    ];

    for (profile, operation) in profiles {
        assert_eq!(FormationProfile::from_str(profile.as_str()), Ok(profile));
        assert_eq!(profile.operation(), operation);
        assert_eq!(profile.schema_version(), "1");
    }
    assert!(FormationProfile::from_str("conversation-v1").is_err());
    assert!(FormationProfile::from_str("model-chosen-v1").is_err());
}

#[test]
fn binding_is_closed_and_bounded_for_each_provider() {
    let binding = FormationProfile::BackgroundReconciliationV1.bind(FormationProvider::SailV1);
    assert_eq!(binding.operation, CognitionOperation::Reconcile);
    assert_eq!(binding.input_schema_version, "1");
    assert_eq!(binding.output_schema_version, "1");
    assert_eq!(binding.max_source_records, 10_000);
    assert_eq!(binding.max_output_records, 10_000);
    assert_eq!(binding.capability, FormationCapability::Reconcile);
    assert_eq!(
        binding.budget,
        FormationResourceBudget {
            max_source_records: 10_000,
            max_output_records: 10_000,
        }
    );
}

#[test]
fn registry_resolves_only_trusted_provider_profile_pairs() {
    let registry = FormationRegistry;
    let binding = registry
        .resolve(
            FormationProfile::ConversationDeduplicationV1,
            FormationProvider::ReferenceV1,
        )
        .expect("closed native pair");
    assert_eq!(binding.provider, FormationProvider::ReferenceV1);
    assert_eq!(binding.profile.as_str(), "conversation-deduplication-v1");
    assert_eq!(FormationProvider::SailV1.as_str(), "sail-v1");
}

#[test]
fn provider_budget_checks_source_and_output_boundaries() {
    let budget = FormationResourceBudget {
        max_source_records: 2,
        max_output_records: 3,
    };
    budget
        .check_source_records(2)
        .expect("inclusive source bound");
    budget
        .check_output_records(3)
        .expect("inclusive output bound");
    assert!(budget.check_source_records(3).is_err());
    assert!(budget.check_output_records(4).is_err());
}
