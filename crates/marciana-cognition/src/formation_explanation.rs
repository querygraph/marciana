//! Content-free explanations for one resolved formation proposal.

use sha2::{Digest, Sha256};

use crate::{
    FormationBinding, FormationBudgetError, FormationProfile, FormationProvider, FormationRunMode,
};
use querygraph_memory::cognition::CognitionOperation;

/// Stable explanation metadata for an inferred formation proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormationExplanation {
    pub profile: FormationProfile,
    pub provider: FormationProvider,
    pub operation: CognitionOperation,
    pub run_mode: FormationRunMode,
    pub source_manifest_digest: String,
    pub proposal_digest: String,
    pub considered_records: u32,
    pub proposed_records: u32,
    digest: String,
}

impl FormationBinding {
    /// Explain a bounded proposal without retaining source or model text.
    ///
    /// # Errors
    /// Returns a fixed error for malformed digests or counts over the
    /// resolved provider's source/output budgets.
    pub fn explain(
        &self,
        source_manifest_digest: String,
        proposal_digest: String,
        considered_records: usize,
        proposed_records: usize,
    ) -> Result<FormationExplanation, FormationExplanationError> {
        if !is_digest(&source_manifest_digest) || !is_digest(&proposal_digest) {
            return Err(FormationExplanationError::InvalidDigest);
        }
        self.budget
            .check_source_records(considered_records)
            .map_err(FormationExplanationError::SourceBudget)?;
        self.budget
            .check_output_records(proposed_records)
            .map_err(FormationExplanationError::OutputBudget)?;
        let considered_records =
            u32::try_from(considered_records).map_err(|_| FormationExplanationError::Bounds)?;
        let proposed_records =
            u32::try_from(proposed_records).map_err(|_| FormationExplanationError::Bounds)?;
        let digest = explanation_digest(
            self,
            &source_manifest_digest,
            &proposal_digest,
            considered_records,
            proposed_records,
        );
        Ok(FormationExplanation {
            profile: self.profile,
            provider: self.provider,
            operation: self.operation,
            run_mode: self.run_mode,
            source_manifest_digest,
            proposal_digest,
            considered_records,
            proposed_records,
            digest,
        })
    }
}

impl FormationExplanation {
    /// Stable content-free explanation identity.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
}

/// Fixed formation-explanation failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FormationExplanationError {
    #[error("formation explanation digest is invalid")]
    InvalidDigest,
    #[error("formation explanation exceeds its source budget")]
    SourceBudget(#[source] FormationBudgetError),
    #[error("formation explanation exceeds its output budget")]
    OutputBudget(#[source] FormationBudgetError),
    #[error("formation explanation count is out of bounds")]
    Bounds,
}

fn is_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn explanation_digest(
    binding: &FormationBinding,
    source_manifest_digest: &str,
    proposal_digest: &str,
    considered_records: u32,
    proposed_records: u32,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.formation-explanation.v1\0");
    for value in [
        binding.profile.as_str(),
        binding.provider.as_str(),
        binding.operation.as_str(),
        match binding.run_mode {
            FormationRunMode::Background => "background",
            FormationRunMode::HotPathProposal => "hot-path-proposal",
        },
        source_manifest_digest,
        proposal_digest,
    ] {
        hasher.update(value.as_bytes());
        hasher.update([0]);
    }
    hasher.update(considered_records.to_be_bytes());
    hasher.update(proposed_records.to_be_bytes());
    format!("sha256:{:x}", hasher.finalize())
}
