use chrono::{TimeZone, Utc};
use querygraph_memory::context::{
    ContextCandidate, ContextError, ContextRecipe, ContextView, RecallIntent, plan_context,
};
use sha2::Digest;
use typesec_memory::MemoryId;

fn digest(label: &str) -> String {
    format!("sha256:{:x}", sha2::Sha256::digest(label.as_bytes()))
}

#[test]
fn planning_is_deterministic_and_content_free() {
    let intent = RecallIntent {
        query_digest: digest("query"),
        working_set_digest: None,
        pinned_memory_ids: Vec::new(),
        view: ContextView::Assertions,
        recipe: ContextRecipe::CurrentAssertions,
        as_of: Utc.timestamp_opt(10, 0).unwrap(),
        token_budget: 5,
    };
    let candidates = vec![
        ContextCandidate {
            id: MemoryId::from_string("mem-b"),
            score_basis_points: 9000,
            estimated_tokens: 3,
            reason_digest: digest("b"),
        },
        ContextCandidate {
            id: MemoryId::from_string("mem-a"),
            score_basis_points: 9000,
            estimated_tokens: 2,
            reason_digest: digest("a"),
        },
        ContextCandidate {
            id: MemoryId::from_string("mem-c"),
            score_basis_points: 1,
            estimated_tokens: 1,
            reason_digest: digest("c"),
        },
    ];
    let plan = plan_context(intent.clone(), candidates).unwrap();
    assert_eq!(
        plan.candidates
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        ["mem-a", "mem-b"]
    );
    assert_eq!(plan.estimated_tokens, 5);
    assert_eq!(plan.considered_candidates, 3);
    assert!(plan.plan_digest.starts_with("sha256:"));
    assert_ne!(
        plan_context(intent, plan.candidates.clone())
            .unwrap()
            .plan_digest,
        ""
    );
}

#[test]
fn plan_validation_rejects_tampered_digest_and_duplicate_candidates() {
    let intent = RecallIntent {
        query_digest: digest("query"),
        working_set_digest: None,
        pinned_memory_ids: Vec::new(),
        view: ContextView::Episodes,
        recipe: ContextRecipe::Ranked,
        as_of: Utc.timestamp_opt(10, 0).unwrap(),
        token_budget: 5,
    };
    let mut plan = plan_context(
        intent,
        vec![ContextCandidate {
            id: MemoryId::from_string("mem-a"),
            score_basis_points: 1,
            estimated_tokens: 1,
            reason_digest: digest("a"),
        }],
    )
    .unwrap();
    plan.plan_digest = digest("tampered");
    assert!(matches!(plan.validate(), Err(ContextError::PlanDigest)));

    let mut duplicate = plan_context(
        RecallIntent {
            query_digest: digest("query-2"),
            working_set_digest: None,
            pinned_memory_ids: Vec::new(),
            view: ContextView::Episodes,
            recipe: ContextRecipe::Ranked,
            as_of: Utc.timestamp_opt(10, 0).unwrap(),
            token_budget: 5,
        },
        vec![ContextCandidate {
            id: MemoryId::from_string("mem-a"),
            score_basis_points: 1,
            estimated_tokens: 1,
            reason_digest: digest("a"),
        }],
    )
    .unwrap();
    duplicate.candidates.push(duplicate.candidates[0].clone());
    assert!(matches!(
        duplicate.validate(),
        Err(ContextError::InvalidCandidate)
    ));
}

#[test]
fn non_hex_digests_are_rejected_even_with_valid_shape() {
    // Correct "sha256:" prefix and 71-byte length, but not hexadecimal.
    let shaped_not_hex = format!("sha256:{}", "z".repeat(64));
    let intent = RecallIntent {
        query_digest: shaped_not_hex.clone(),
        working_set_digest: None,
        pinned_memory_ids: Vec::new(),
        view: ContextView::Assertions,
        recipe: ContextRecipe::Ranked,
        as_of: Utc.timestamp_opt(10, 0).unwrap(),
        token_budget: 5,
    };
    assert!(matches!(
        intent.validate(),
        Err(ContextError::InvalidIntent)
    ));

    let valid_intent = RecallIntent {
        query_digest: digest("query"),
        working_set_digest: None,
        pinned_memory_ids: Vec::new(),
        view: ContextView::Assertions,
        recipe: ContextRecipe::Ranked,
        as_of: Utc.timestamp_opt(10, 0).unwrap(),
        token_budget: 5,
    };
    assert!(matches!(
        plan_context(
            valid_intent,
            vec![ContextCandidate {
                id: MemoryId::from_string("mem-a"),
                score_basis_points: 1,
                estimated_tokens: 1,
                reason_digest: shaped_not_hex,
            }]
        ),
        Err(ContextError::InvalidCandidate)
    ));
}

#[test]
fn pinned_working_set_candidates_are_selected_first_and_required() {
    let pinned = MemoryId::from_string("mem-pinned");
    let intent = RecallIntent {
        query_digest: digest("pinned-query"),
        working_set_digest: None,
        pinned_memory_ids: vec![pinned.clone()],
        view: ContextView::Episodes,
        recipe: ContextRecipe::Ranked,
        as_of: Utc.timestamp_opt(10, 0).unwrap(),
        token_budget: 4,
    };
    let plan = plan_context(
        intent,
        vec![
            ContextCandidate {
                id: MemoryId::from_string("mem-ranked"),
                score_basis_points: 10_000,
                estimated_tokens: 1,
                reason_digest: digest("ranked"),
            },
            ContextCandidate {
                id: pinned.clone(),
                score_basis_points: 1,
                estimated_tokens: 1,
                reason_digest: digest("pinned"),
            },
        ],
    )
    .unwrap();
    plan.validate().expect("pinned plan remains canonical");
    assert_eq!(plan.candidates[0].id, pinned);

    let missing = RecallIntent {
        query_digest: digest("missing-pinned"),
        working_set_digest: None,
        pinned_memory_ids: vec![MemoryId::from_string("mem-missing")],
        view: ContextView::Episodes,
        recipe: ContextRecipe::Ranked,
        as_of: Utc.timestamp_opt(10, 0).unwrap(),
        token_budget: 4,
    };
    assert!(matches!(
        plan_context(missing, Vec::new()),
        Err(ContextError::PinnedCandidateMissing)
    ));

    let over_budget = RecallIntent {
        query_digest: digest("over-budget-pinned"),
        working_set_digest: None,
        pinned_memory_ids: vec![MemoryId::from_string("mem-pinned")],
        view: ContextView::Episodes,
        recipe: ContextRecipe::Ranked,
        as_of: Utc.timestamp_opt(10, 0).unwrap(),
        token_budget: 1,
    };
    assert!(matches!(
        plan_context(
            over_budget,
            vec![ContextCandidate {
                id: MemoryId::from_string("mem-pinned"),
                score_basis_points: 1,
                estimated_tokens: 2,
                reason_digest: digest("over-budget"),
            }]
        ),
        Err(ContextError::TokenAccounting)
    ));
}

#[test]
fn duplicate_pinned_candidates_fail_before_planning() {
    let pinned = MemoryId::from_string("mem-pinned");
    let intent = RecallIntent {
        query_digest: digest("duplicate-pinned-query"),
        working_set_digest: None,
        pinned_memory_ids: vec![pinned.clone(), pinned],
        view: ContextView::Episodes,
        recipe: ContextRecipe::Ranked,
        as_of: Utc.timestamp_opt(10, 0).unwrap(),
        token_budget: 4,
    };

    assert!(matches!(
        plan_context(intent, Vec::new()),
        Err(ContextError::InvalidIntent)
    ));
}
