use super::CognitionBindingError;

#[test]
fn public_binding_errors_are_fixed_and_non_disclosing() {
    assert_eq!(
        CognitionBindingError::IntentClaimMismatch("jobId").to_string(),
        "verified TypeDID cognition claim does not match: jobId"
    );
    assert_eq!(
        CognitionBindingError::InvalidProof.to_string(),
        "invalid LakeCat governed scan evidence"
    );
    assert_eq!(
        CognitionBindingError::ProposalNotPlanned.to_string(),
        "cognition proposal was not produced by the bound engine"
    );
}
