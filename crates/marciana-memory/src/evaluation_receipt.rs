//! Release-bound identity for content-free context evaluation.

use sha2::{Digest, Sha256};

use crate::evaluation::ContextEvaluationSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEvaluationReceipt {
    corpus_digest: String,
    summary_digest: String,
    evaluator_digest: String,
    receipt_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum EvaluationReceiptError {
    #[error("evaluation receipt identity is invalid")]
    Invalid,
    #[error("evaluation summary did not pass its safety checks")]
    SummaryFailed,
}

impl ContextEvaluationReceipt {
    pub fn new(
        summary: &ContextEvaluationSummary,
        evaluator_digest: String,
    ) -> Result<Self, EvaluationReceiptError> {
        if !summary.passed {
            return Err(EvaluationReceiptError::SummaryFailed);
        }
        if !is_digest(&evaluator_digest)
            || !is_digest(&summary.corpus_digest)
            || !is_digest(&summary.summary_digest)
        {
            return Err(EvaluationReceiptError::Invalid);
        }
        let mut hasher = Sha256::new();
        hasher.update(b"querygraph.marciana.context-evaluation-receipt.v1\0");
        for value in [
            summary.corpus_digest.as_str(),
            summary.summary_digest.as_str(),
            evaluator_digest.as_str(),
        ] {
            hasher.update(value.as_bytes());
            hasher.update([0]);
        }
        Ok(Self {
            corpus_digest: summary.corpus_digest.clone(),
            summary_digest: summary.summary_digest.clone(),
            evaluator_digest,
            receipt_digest: format!("sha256:{:x}", hasher.finalize()),
        })
    }

    #[must_use]
    pub fn corpus_digest(&self) -> &str {
        &self.corpus_digest
    }

    #[must_use]
    pub fn summary_digest(&self) -> &str {
        &self.summary_digest
    }

    #[must_use]
    pub fn evaluator_digest(&self) -> &str {
        &self.evaluator_digest
    }

    #[must_use]
    pub fn receipt_digest(&self) -> &str {
        &self.receipt_digest
    }
}

fn is_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}
