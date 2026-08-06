//! Exact validation of untrusted cognition-engine proposals.

use crate::CognitionBindingError;
use typesec_memory::{CognitionBinding, CognitionProposal, CognitionSourceManifest, MemoryId};

use super::binding::BindingBasis;

/// Engine identity released only after the proposal preserves signed intent.
pub(crate) struct ValidatedProposalIdentity {
    proposal_digest: String,
}

impl ValidatedProposalIdentity {
    pub(crate) fn proposal_digest(&self) -> &str {
        &self.proposal_digest
    }
}

pub(crate) fn validate_planned_proposal(
    proposal: &CognitionProposal,
    basis: &BindingBasis,
    binding: &CognitionBinding,
    manifest: &CognitionSourceManifest,
) -> Result<ValidatedProposalIdentity, CognitionBindingError> {
    let identity = validate_proposal_intent(proposal, basis)?;
    let manifest_sources: Vec<_> = manifest
        .sources
        .iter()
        .map(|source| source.id.clone())
        .collect();
    if manifest_sources != basis.source_ids
        || proposal.source_ids != manifest_sources
        || proposal.binding.as_ref() != Some(binding)
        || proposal.input_snapshot != binding.snapshot_digest
        || proposal.source_digest != binding.source_manifest_digest
        || proposal.joined_label != manifest.joined_label
    {
        return Err(CognitionBindingError::EngineOutputMismatch);
    }
    Ok(identity)
}

pub(crate) fn validate_proposal_intent(
    proposal: &CognitionProposal,
    basis: &BindingBasis,
) -> Result<ValidatedProposalIdentity, CognitionBindingError> {
    let binding = proposal
        .binding
        .as_ref()
        .ok_or(CognitionBindingError::MissingProposalBinding)?;
    basis.verify_binding(binding)?;
    if proposal.job_id != basis.intent.job_id
        || proposal.source_ids != basis.source_ids
        || proposal.input_snapshot != basis.snapshot_identity()
        || proposal.source_digest != binding.source_manifest_digest
        || proposal.algorithm != basis.intent.algorithm
        || proposal.algorithm_version != basis.intent.algorithm_version
    {
        return Err(CognitionBindingError::ProposalIntentMismatch);
    }
    let proposal_digest = proposal
        .canonical_digest()
        .map_err(|_| CognitionBindingError::InvalidProposalDigest)?;
    Ok(ValidatedProposalIdentity { proposal_digest })
}

pub(crate) fn exact_manifest_ids(manifest: &CognitionSourceManifest) -> Vec<MemoryId> {
    manifest
        .sources
        .iter()
        .map(|source| source.id.clone())
        .collect()
}
