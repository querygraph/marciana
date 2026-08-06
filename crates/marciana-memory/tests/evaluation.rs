use chrono::{TimeZone, Utc};
use querygraph_memory::context::{
    ContextCandidate, ContextRecipe, ContextView, RecallIntent, plan_context,
};
use querygraph_memory::{ContextEvaluationCase, ContextEvaluationReport};
use sha2::Digest;
use typesec_memory::MemoryId;

fn digest(value: &str) -> String {
    format!("sha256:{:x}", sha2::Sha256::digest(value.as_bytes()))
}

fn plan(ids: &[&str]) -> querygraph_memory::context::ContextPlan {
    plan_context(
        RecallIntent {
            query_digest: digest("query"),
            working_set_digest: None,
            pinned_memory_ids: Vec::new(),
            view: ContextView::Assertions,
            recipe: ContextRecipe::Ranked,
            as_of: Utc.timestamp_opt(10, 0).unwrap(),
            token_budget: 32,
        },
        ids.iter()
            .map(|id| ContextCandidate {
                id: MemoryId::from_string(*id),
                score_basis_points: 100,
                estimated_tokens: 2,
                reason_digest: digest(id),
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn evaluation_reports_quality_and_token_utility_without_content() {
    let case = ContextEvaluationCase::new(
        digest("case"),
        vec![
            MemoryId::from_string("mem-a"),
            MemoryId::from_string("mem-b"),
        ],
        vec![MemoryId::from_string("mem-secret")],
        8,
    )
    .unwrap();
    let report = ContextEvaluationReport::evaluate(&case, &plan(&["mem-a", "mem-c"])).unwrap();
    assert_eq!(report.relevant_count, 1);
    assert_eq!(report.precision_basis_points, 5_000);
    assert_eq!(report.recall_basis_points, 5_000);
    assert_eq!(report.token_utility_basis_points, 2_500);
    assert!(report.passed);
    assert!(report.report_digest.starts_with("sha256:"));
}

#[test]
fn forbidden_ids_fail_the_evaluation_without_leaking_values() {
    let case = ContextEvaluationCase::new(
        digest("case"),
        vec![MemoryId::from_string("mem-a")],
        vec![MemoryId::from_string("mem-secret")],
        8,
    )
    .unwrap();
    let report = ContextEvaluationReport::evaluate(&case, &plan(&["mem-secret"])).unwrap();
    assert_eq!(report.forbidden_count, 1);
    assert!(!report.passed);
}
