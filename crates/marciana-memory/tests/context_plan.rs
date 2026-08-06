use chrono::{TimeZone, Utc};
use querygraph_memory::context::{
    plan_context, ContextCandidate, ContextError, ContextRecipe, ContextView, RecallIntent,
};
use typesec_memory::MemoryId;

fn digest(label: &str) -> String {
    format!("sha256:{}", format!("{label:0<64}")[..64].to_owned())
}

#[test]
fn planning_is_deterministic_and_content_free() {
    let intent = RecallIntent {
        query_digest: digest("query"),
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
