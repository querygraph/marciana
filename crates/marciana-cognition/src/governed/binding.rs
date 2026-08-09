//! Construction and revalidation of exact TypeSec cognition bindings.

use crate::CognitionBindingError;
use chrono::{DateTime, Utc};
use lakecat_core::governed_scan::{
    GovernedScanCatalogIdentity, GovernedScanProof, validate_governed_sha256_digest,
};
use marciana_catalog::governed_cognition_source;
use querygraph_memory::cognition::{
    CognitionFieldMapping, GovernedLakeCatSnapshot, is_canonical_projection,
};
use typesec_integrations::VerifiedTypeDidContext;
use typesec_memory::{
    CognitionAuthorityEvidence, CognitionBinding, CognitionSourceManifest, GovernedSourceScope,
    MemoryId, MemorySpace,
};

use super::governed_proof::validate_governed_proof;
use super::intent::{CognitionIntent, IntentInputs, canonical_source_ids};
use super::projection::RequiredProjection;
use crate::FormationProfile;

#[derive(Debug, Clone)]
pub(crate) struct BindingBasis {
    catalog: GovernedScanCatalogIdentity,
    pub(crate) space_id: String,
    pub(crate) proof: GovernedScanProof,
    pub(crate) intent: CognitionIntent,
    pub(crate) source_ids: Vec<MemoryId>,
    required_projection: RequiredProjection,
    field_mapping: CognitionFieldMapping,
    source: GovernedLakeCatSnapshot,
    governed_source_scope: GovernedSourceScope,
}

impl BindingBasis {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        catalog: &GovernedScanCatalogIdentity,
        space: &MemorySpace,
        verified: VerifiedTypeDidContext<'_>,
        proof: GovernedScanProof,
        source_ids: &[MemoryId],
        field_mapping: &CognitionFieldMapping,
        formation_profile: FormationProfile,
        now: DateTime<Utc>,
    ) -> Result<Self, CognitionBindingError> {
        let digests = validate_governed_proof(catalog, &proof)?;
        let source_ids = canonical_source_ids(source_ids)?;
        let intent = CognitionIntent::from_verified(
            verified,
            IntentInputs {
                space,
                catalog: catalog.as_str(),
                proof: &proof,
                source_ids: &source_ids,
                field_mapping,
                formation_profile,
            },
            now,
        )?;
        verify_request(&intent, &proof)?;
        let required_projection = RequiredProjection::new([
            field_mapping.id.clone(),
            field_mapping.text.clone(),
            field_mapping.valid_from.clone(),
        ])?;
        required_projection.verify(&proof)?;
        let source = governed_cognition_source(&proof, &digests, required_projection.fields())
            .map_err(|_| CognitionBindingError::InvalidProof)?;
        let governed_source_scope =
            GovernedSourceScope::from_digest(digests.source_scope_digest().to_owned())
                .map_err(|_| CognitionBindingError::Digest)?;
        Ok(Self {
            catalog: catalog.clone(),
            space_id: typesec_core::Resource::resource_id(space).to_owned(),
            proof,
            intent,
            source_ids,
            required_projection,
            field_mapping: field_mapping.clone(),
            source,
            governed_source_scope,
        })
    }

    pub(crate) fn binding(&self, manifest: &CognitionSourceManifest) -> CognitionBinding {
        CognitionBinding {
            space_id: self.space_id.clone(),
            subject: self.intent.subject.clone(),
            purpose: self.intent.purpose.clone(),
            governed_source_scope: Some(self.governed_source_scope.clone()),
            governed_scan_digest: self.proof.grant_id().to_owned(),
            snapshot_digest: self.source.snapshot_digest.clone(),
            plan_task_digest: self.proof.plan_task_digest().to_owned(),
            authorization_receipt_digest: self.proof.authorization_receipt_digest().to_owned(),
            effective_projection: self.required_projection.fields().to_vec(),
            source_manifest_digest: manifest.digest.clone(),
            typedid_request_digest: self.intent.request_digest.clone(),
        }
    }

    pub(crate) fn verify_binding(
        &self,
        binding: &CognitionBinding,
    ) -> Result<(), CognitionBindingError> {
        validate_governed_sha256_digest(&binding.source_manifest_digest, "source manifest")
            .map_err(|_| CognitionBindingError::InvalidProposalDigest)?;
        if binding.space_id != self.space_id
            || binding.subject != self.intent.subject
            || binding.purpose != self.intent.purpose
            || binding.governed_source_scope.as_ref() != Some(&self.governed_source_scope)
            || binding.governed_scan_digest != self.proof.grant_id()
            || binding.snapshot_digest != self.source.snapshot_digest
            || binding.plan_task_digest != self.proof.plan_task_digest()
            || binding.authorization_receipt_digest != self.proof.authorization_receipt_digest()
            || binding.effective_projection.as_slice() != self.required_projection.fields()
            || binding.typedid_request_digest != self.intent.request_digest
        {
            return Err(CognitionBindingError::ProposalIntentMismatch);
        }
        Ok(())
    }

    pub(crate) fn validate_fresh_proof(
        &self,
        catalog: &GovernedScanCatalogIdentity,
        proof: &GovernedScanProof,
    ) -> Result<(), CognitionBindingError> {
        if catalog != &self.catalog {
            return Err(CognitionBindingError::CatalogMismatch);
        }
        validate_governed_proof(catalog, proof)?;
        verify_request(&self.intent, proof)?;
        if proof != &self.proof {
            return Err(CognitionBindingError::FreshProofMismatch);
        }
        Ok(())
    }

    pub(crate) fn snapshot_identity(&self) -> &str {
        &self.source.snapshot_digest
    }

    pub(crate) fn verify_current_projection(
        &self,
        current: &[String],
    ) -> Result<(), CognitionBindingError> {
        if !is_canonical_projection(current)
            || self
                .required_projection
                .fields()
                .iter()
                .any(|field| !current.contains(field))
            || current
                .iter()
                .any(|field| !self.proof.effective_projection().contains(field))
        {
            return Err(CognitionBindingError::ProjectionMismatch);
        }
        Ok(())
    }

    pub(crate) fn authority_evidence(
        &self,
        policy_decision_id: String,
    ) -> CognitionAuthorityEvidence {
        CognitionAuthorityEvidence {
            space_id: self.space_id.clone(),
            subject: self.intent.subject.clone(),
            purpose: self.intent.purpose.clone(),
            governed_source_scope: Some(self.governed_source_scope.clone()),
            job_id: self.intent.job_id.clone(),
            algorithm: self.intent.algorithm.clone(),
            algorithm_version: self.intent.algorithm_version.clone(),
            governed_scan_digest: self.proof.grant_id().to_owned(),
            snapshot_digest: self.source.snapshot_digest.clone(),
            plan_task_digest: self.proof.plan_task_digest().to_owned(),
            authorization_receipt_digest: self.proof.authorization_receipt_digest().to_owned(),
            effective_projection: self.required_projection.fields().to_vec(),
            typedid_request_digest: self.intent.request_digest.clone(),
            policy_decision_id,
        }
    }

    pub(crate) fn planning_source(&self) -> &GovernedLakeCatSnapshot {
        &self.source
    }

    pub(crate) fn governed_source_scope(&self) -> &GovernedSourceScope {
        &self.governed_source_scope
    }

    pub(crate) fn field_mapping(&self) -> &CognitionFieldMapping {
        &self.field_mapping
    }
}

fn verify_request(
    request: &CognitionIntent,
    proof: &GovernedScanProof,
) -> Result<(), CognitionBindingError> {
    if proof.principal_subject() != request.subject {
        return Err(CognitionBindingError::SubjectMismatch);
    }
    if proof.purpose() != request.purpose {
        return Err(CognitionBindingError::PurposeMismatch);
    }
    Ok(())
}
