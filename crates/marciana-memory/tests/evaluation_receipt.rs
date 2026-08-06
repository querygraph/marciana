use chrono::{TimeZone, Utc};
use querygraph_memory::context::{
    ContextCandidate, ContextRecipe, ContextView, RecallIntent, plan_context,
};
use querygraph_memory::{ContextEvaluationCase, ContextEvaluationCorpus, ContextEvaluationReceipt};
use sha2::Digest;
use typesec_memory::MemoryId;

fn digest(value: &str) -> String {
    format!("sha256:{:x}", sha2::Sha256::digest(value.as_bytes()))
}

#[test]
fn receipt_binds_summary_to_evaluator_identity() {
    let case = ContextEvaluationCase::new(
        digest("case"),
        vec![MemoryId::from_string("mem-a")],
        Vec::new(),
        8,
    )
    .unwrap();
    let corpus = ContextEvaluationCorpus::new(vec![case]).unwrap();
    let plan = plan_context(
        RecallIntent {
            query_digest: digest("query"),
            working_set_digest: None,
            pinned_memory_ids: Vec::new(),
            view: ContextView::Assertions,
            recipe: ContextRecipe::Ranked,
            as_of: Utc.timestamp_opt(10, 0).unwrap(),
            token_budget: 8,
        },
        vec![ContextCandidate {
            id: MemoryId::from_string("mem-a"),
            score_basis_points: 100,
            estimated_tokens: 2,
            reason_digest: digest("reason"),
        }],
    )
    .unwrap();
    let summary = corpus.evaluate(&[plan]).unwrap();
    let receipt = ContextEvaluationReceipt::new(&summary, digest("evaluator-v1")).unwrap();
    assert_eq!(receipt.corpus_digest(), summary.corpus_digest);
    assert_eq!(receipt.summary_digest(), summary.summary_digest);
    assert_eq!(receipt.evaluator_digest(), digest("evaluator-v1"));
    assert!(receipt.receipt_digest().starts_with("sha256:"));
}
