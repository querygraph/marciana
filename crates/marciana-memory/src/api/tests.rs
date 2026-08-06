use super::{ApiError, ForgetRequest, RecallRequest, RememberRequest};

fn remember() -> RememberRequest {
    RememberRequest {
        space_id: "tenant/coffee".into(),
        text: "price observed".into(),
        purpose: "research".into(),
    }
}

#[test]
fn public_requests_validate_without_performing_authorization() {
    assert!(remember().validate().is_ok());
    assert!(
        RecallRequest {
            space_id: "tenant/coffee".into(),
            query: "price".into(),
            purpose: "research".into()
        }
        .validate()
        .is_ok()
    );
    assert!(
        ForgetRequest {
            space_id: "tenant/coffee".into(),
            memory_ids: vec!["m1".into()],
            purpose: "research".into()
        }
        .validate()
        .is_ok()
    );
}

#[test]
fn validation_is_bounded_and_non_disclosing() {
    let invalid = RememberRequest {
        space_id: "tenant coffee".into(),
        text: "secret plaintext".into(),
        purpose: "research".into(),
    };
    assert_eq!(invalid.validate(), Err(ApiError::InvalidIdentity));
    assert_eq!(
        ForgetRequest {
            space_id: "tenant/coffee".into(),
            memory_ids: Vec::new(),
            purpose: "research".into()
        }
        .validate(),
        Err(ApiError::InvalidIds)
    );
}
