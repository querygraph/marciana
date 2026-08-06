use marciana_cognition::{
    FormationExplanationError, FormationProfile, FormationProvider, FormationRegistry,
    FormationRunMode,
};
use querygraph_memory::cognition::CognitionOperation;

fn digest(value: char) -> String {
    format!("sha256:{}", value.to_string().repeat(64))
}

#[test]
fn explanation_is_content_free_and_bound_to_the_resolved_contract() {
    let binding = FormationRegistry
        .resolve_for_mode(
            FormationProfile::ConversationDeduplicationV1,
            FormationProvider::ReferenceV1,
            FormationRunMode::HotPathProposal,
        )
        .expect("binding");
    let explanation = binding
        .explain(digest('a'), digest('b'), 2, 1)
        .expect("explanation");
    assert_eq!(
        explanation.profile,
        FormationProfile::ConversationDeduplicationV1
    );
    assert_eq!(explanation.operation, CognitionOperation::Deduplicate);
    assert_eq!(explanation.run_mode, FormationRunMode::HotPathProposal);
    assert_eq!(explanation.considered_records, 2);
    assert_eq!(explanation.proposed_records, 1);
    assert!(explanation.digest().starts_with("sha256:"));
}

#[test]
fn explanation_rejects_bad_digests_and_budget_overruns() {
    let binding = FormationRegistry
        .resolve(
            FormationProfile::BackgroundDeduplicationV1,
            FormationProvider::ReferenceV1,
        )
        .expect("binding");
    assert_eq!(
        binding
            .explain("plaintext".into(), digest('b'), 1, 1)
            .expect_err("digest"),
        FormationExplanationError::InvalidDigest
    );
    assert!(matches!(
        binding.explain(digest('a'), digest('b'), 10_001, 1),
        Err(FormationExplanationError::SourceBudget(_))
    ));
    assert!(matches!(
        binding.explain(digest('a'), digest('b'), 1, 10_001),
        Err(FormationExplanationError::OutputBudget(_))
    ));
}
