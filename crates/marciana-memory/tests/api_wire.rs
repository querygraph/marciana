use querygraph_memory::{ForgetRequest, ImproveRequest, RecallRequest, RememberRequest};

#[test]
fn four_verb_wire_fixture_is_strict_and_snake_case() {
    let remember: RememberRequest = serde_json::from_str(include_str!(
        "../../../compat/fixtures/api_remember_v1.json"
    ))
    .expect("remember fixture");
    assert_eq!(remember.space_id, "memory/user:alice/semantic");
    let encoded = serde_json::to_value(&remember).expect("serialize remember");
    assert!(encoded.get("space_id").is_some());
    assert!(encoded.get("spaceId").is_none());

    let improve: ImproveRequest = serde_json::from_value(serde_json::json!({
        "space_id": "memory/user:alice/semantic",
        "memory_id": "mem-old",
        "replacement": serde_json::from_str::<RememberRequest>(include_str!("../../../compat/fixtures/api_remember_v1.json")).unwrap(),
    }))
    .expect("improve request");
    assert_eq!(improve.memory_id, "mem-old");
    let _: RecallRequest = serde_json::from_value(serde_json::json!({
        "space_id": "memory/user:alice/semantic",
        "query": "coffee price",
        "purpose": "research"
    }))
    .expect("recall request");
    let _: ForgetRequest = serde_json::from_value(serde_json::json!({
        "space_id": "memory/user:alice/semantic",
        "memory_ids": ["mem-old"],
        "purpose": "research"
    }))
    .expect("forget request");
}

#[test]
fn wire_fixture_rejects_unknown_fields() {
    let error = serde_json::from_value::<RememberRequest>(serde_json::json!({
        "space_id": "memory/user:alice/semantic",
        "text": "coffee",
        "purpose": "research",
        "spaceId": "spoof"
    }))
    .expect_err("unknown field must fail closed");
    assert!(error.to_string().contains("unknown field"));
}
