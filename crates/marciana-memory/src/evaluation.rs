//! Deterministic, content-free evaluation for governed context plans.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};
use typesec_memory::MemoryId;

use crate::context::ContextPlan;

const MAX_EVALUATION_IDS: usize = 1_000;

#[derive(Debug, Clone, Copy)]
struct EvaluationMetrics {
    relevant: usize,
    forbidden: usize,
    precision: u16,
    recall: u16,
    utility: u16,
    passed: bool,
}

/// Expected retrieval behavior for one reproducible evaluation case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEvaluationCase {
    case_digest: String,
    expected_ids: BTreeSet<String>,
    forbidden_ids: BTreeSet<String>,
    token_budget: u32,
}

/// Content-free quality and safety measurements for one context plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEvaluationReport {
    pub case_digest: String,
    pub plan_digest: String,
    pub expected_count: usize,
    pub selected_count: usize,
    pub relevant_count: usize,
    pub forbidden_count: usize,
    pub precision_basis_points: u16,
    pub recall_basis_points: u16,
    pub token_utility_basis_points: u16,
    pub passed: bool,
    pub report_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum EvaluationError {
    #[error("evaluation case is invalid")]
    InvalidCase,
    #[error("context plan exceeds evaluation budget")]
    BudgetExceeded,
    #[error("context plan digest is invalid")]
    InvalidPlan,
}

impl ContextEvaluationCase {
    pub fn new(
        case_digest: String,
        expected_ids: Vec<MemoryId>,
        forbidden_ids: Vec<MemoryId>,
        token_budget: u32,
    ) -> Result<Self, EvaluationError> {
        if !is_digest(&case_digest) || token_budget == 0 {
            return Err(EvaluationError::InvalidCase);
        }
        let expected_ids = id_set(expected_ids)?;
        let forbidden_ids = id_set(forbidden_ids)?;
        if expected_ids.is_disjoint(&forbidden_ids) {
            Ok(Self {
                case_digest,
                expected_ids,
                forbidden_ids,
                token_budget,
            })
        } else {
            Err(EvaluationError::InvalidCase)
        }
    }

    #[must_use]
    pub fn case_digest(&self) -> &str {
        &self.case_digest
    }

    #[must_use]
    pub fn token_budget(&self) -> u32 {
        self.token_budget
    }
}

impl ContextEvaluationReport {
    /// Evaluate only IDs and bounded planner accounting; no content is read.
    pub fn evaluate(
        case: &ContextEvaluationCase,
        plan: &ContextPlan,
    ) -> Result<Self, EvaluationError> {
        plan.validate().map_err(|_| EvaluationError::InvalidPlan)?;
        if plan.estimated_tokens > case.token_budget {
            return Err(EvaluationError::BudgetExceeded);
        }
        let selected = plan
            .candidates
            .iter()
            .map(|candidate| candidate.id.as_str().to_owned())
            .collect::<BTreeSet<_>>();
        let relevant_count = selected.intersection(&case.expected_ids).count();
        let forbidden_count = selected.intersection(&case.forbidden_ids).count();
        let precision = ratio_basis_points(relevant_count, selected.len());
        let recall = ratio_basis_points(relevant_count, case.expected_ids.len());
        let token_utility = ratio_basis_points(
            relevant_count,
            usize::try_from(plan.estimated_tokens).unwrap_or(usize::MAX),
        );
        let passed = forbidden_count == 0;
        let metrics = EvaluationMetrics {
            relevant: relevant_count,
            forbidden: forbidden_count,
            precision,
            recall,
            utility: token_utility,
            passed,
        };
        let report_digest = report_digest(case, plan, metrics);
        Ok(Self {
            case_digest: case.case_digest.clone(),
            plan_digest: plan.plan_digest.clone(),
            expected_count: case.expected_ids.len(),
            selected_count: selected.len(),
            relevant_count,
            forbidden_count,
            precision_basis_points: precision,
            recall_basis_points: recall,
            token_utility_basis_points: token_utility,
            passed,
            report_digest,
        })
    }
}

fn id_set(ids: Vec<MemoryId>) -> Result<BTreeSet<String>, EvaluationError> {
    if ids.len() > MAX_EVALUATION_IDS {
        return Err(EvaluationError::InvalidCase);
    }
    let set = ids
        .into_iter()
        .map(|id| id.as_str().to_owned())
        .collect::<BTreeSet<_>>();
    if set.len() > MAX_EVALUATION_IDS || set.iter().any(|id| id.is_empty() || id.len() > 256) {
        return Err(EvaluationError::InvalidCase);
    }
    Ok(set)
}

fn ratio_basis_points(numerator: usize, denominator: usize) -> u16 {
    if denominator == 0 {
        return 0;
    }
    u16::try_from(
        (u64::try_from(numerator).unwrap_or(u64::MAX) * 10_000)
            .saturating_div(u64::try_from(denominator).unwrap_or(u64::MAX))
            .min(10_000),
    )
    .unwrap_or(10_000)
}

fn report_digest(
    case: &ContextEvaluationCase,
    plan: &ContextPlan,
    metrics: EvaluationMetrics,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.context-evaluation.v1\0");
    for value in [case.case_digest.as_str(), plan.plan_digest.as_str()] {
        hasher.update(value.as_bytes());
        hasher.update([0]);
    }
    hasher.update(
        format!(
            "{}|{}|{}|{}|{}|{}",
            metrics.relevant,
            metrics.forbidden,
            metrics.precision,
            metrics.recall,
            metrics.utility,
            metrics.passed
        )
        .as_bytes(),
    );
    format!("sha256:{:x}", hasher.finalize())
}

fn is_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}
