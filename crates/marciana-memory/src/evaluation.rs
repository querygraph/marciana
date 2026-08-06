//! Deterministic, content-free evaluation for governed context plans.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use typesec_memory::MemoryId;

use crate::context::ContextPlan;

const MAX_EVALUATION_IDS: usize = 1_000;
const MAX_EVALUATION_CASES: usize = 1_000;

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// A bounded, ordered suite of synthetic or user-owned evaluation cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEvaluationCorpus {
    cases: Vec<ContextEvaluationCase>,
    corpus_digest: String,
}

/// Aggregate quality and safety measurements for one corpus run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEvaluationSummary {
    pub corpus_digest: String,
    pub case_count: usize,
    pub passed_count: usize,
    pub leakage_case_count: usize,
    pub average_precision_basis_points: u16,
    pub average_recall_basis_points: u16,
    pub average_token_utility_basis_points: u16,
    pub passed: bool,
    pub summary_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum EvaluationError {
    #[error("evaluation case is invalid")]
    InvalidCase,
    #[error("context plan exceeds evaluation budget")]
    BudgetExceeded,
    #[error("context plan digest is invalid")]
    InvalidPlan,
    #[error("evaluation corpus is invalid")]
    InvalidCorpus,
    #[error("evaluation plan count does not match its corpus")]
    PlanCountMismatch,
}

impl ContextEvaluationCase {
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError`] when a case identity, digest, or bound is
    /// invalid.
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
            let case = Self {
                case_digest,
                expected_ids,
                forbidden_ids,
                token_budget,
            };
            case.validate().map(|()| case)
        } else {
            Err(EvaluationError::InvalidCase)
        }
    }

    /// Validate a decoded fixture before it enters a corpus.
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError`] when a case identity, digest, or bound is
    /// invalid.
    pub fn validate(&self) -> Result<(), EvaluationError> {
        if !is_digest(&self.case_digest)
            || self.token_budget == 0
            || self.expected_ids.len() > MAX_EVALUATION_IDS
            || self.forbidden_ids.len() > MAX_EVALUATION_IDS
            || self
                .expected_ids
                .iter()
                .chain(self.forbidden_ids.iter())
                .any(|id| id.is_empty() || id.len() > 256)
            || !self.expected_ids.is_disjoint(&self.forbidden_ids)
        {
            return Err(EvaluationError::InvalidCase);
        }
        Ok(())
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
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError`] when the case is invalid or the rendered
    /// context violates the evaluation bounds.
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

impl ContextEvaluationCorpus {
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError`] when the corpus is empty, oversized, or
    /// contains an invalid or duplicate case.
    pub fn new(cases: Vec<ContextEvaluationCase>) -> Result<Self, EvaluationError> {
        if cases.is_empty() || cases.len() > MAX_EVALUATION_CASES {
            return Err(EvaluationError::InvalidCorpus);
        }
        let mut digests = BTreeSet::new();
        for case in &cases {
            case.validate()?;
            if !digests.insert(case.case_digest.clone()) {
                return Err(EvaluationError::InvalidCorpus);
            }
        }
        let mut hasher = Sha256::new();
        hasher.update(b"querygraph.marciana.context-evaluation-corpus.v1\0");
        for case in &cases {
            hasher.update(case.case_digest.as_bytes());
            hasher.update([0]);
        }
        Ok(Self {
            cases,
            corpus_digest: format!("sha256:{:x}", hasher.finalize()),
        })
    }

    #[must_use]
    pub fn cases(&self) -> &[ContextEvaluationCase] {
        &self.cases
    }

    #[must_use]
    pub fn corpus_digest(&self) -> &str {
        &self.corpus_digest
    }

    /// Evaluate plans in the corpus's declared stable order.
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError`] when any case in the corpus fails to
    /// evaluate.
    pub fn evaluate(
        &self,
        plans: &[ContextPlan],
    ) -> Result<ContextEvaluationSummary, EvaluationError> {
        if plans.len() != self.cases.len() {
            return Err(EvaluationError::PlanCountMismatch);
        }
        let reports = self
            .cases
            .iter()
            .zip(plans)
            .map(|(case, plan)| ContextEvaluationReport::evaluate(case, plan))
            .collect::<Result<Vec<_>, _>>()?;
        let case_count = reports.len();
        let passed_count = reports.iter().filter(|report| report.passed).count();
        let leakage_case_count = reports
            .iter()
            .filter(|report| report.forbidden_count > 0)
            .count();
        let average_precision_basis_points =
            average_metric(reports.iter().map(|report| report.precision_basis_points));
        let average_recall_basis_points =
            average_metric(reports.iter().map(|report| report.recall_basis_points));
        let average_token_utility_basis_points = average_metric(
            reports
                .iter()
                .map(|report| report.token_utility_basis_points),
        );
        let passed = passed_count == case_count;
        let summary_digest = summary_digest(
            self,
            &reports,
            average_precision_basis_points,
            average_recall_basis_points,
            average_token_utility_basis_points,
            passed,
        );
        Ok(ContextEvaluationSummary {
            corpus_digest: self.corpus_digest.clone(),
            case_count,
            passed_count,
            leakage_case_count,
            average_precision_basis_points,
            average_recall_basis_points,
            average_token_utility_basis_points,
            passed,
            summary_digest,
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

fn average_metric(values: impl Iterator<Item = u16>) -> u16 {
    let values = values.collect::<Vec<_>>();
    if values.is_empty() {
        return 0;
    }
    let total = values.iter().map(|value| u64::from(*value)).sum::<u64>();
    u16::try_from(total / u64::try_from(values.len()).unwrap_or(1)).unwrap_or(u16::MAX)
}

fn summary_digest(
    corpus: &ContextEvaluationCorpus,
    reports: &[ContextEvaluationReport],
    precision: u16,
    recall: u16,
    utility: u16,
    passed: bool,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.context-evaluation-summary.v1\0");
    hasher.update(corpus.corpus_digest.as_bytes());
    for report in reports {
        hasher.update(report.report_digest.as_bytes());
        hasher.update([0]);
    }
    hasher.update(format!("{precision}|{recall}|{utility}|{passed}").as_bytes());
    format!("sha256:{:x}", hasher.finalize())
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
