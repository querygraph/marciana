//! Exact canonical cognition projection.

use crate::CognitionBindingError;
use lakecat_core::governed_scan::GovernedScanProof;

use querygraph_memory::cognition::is_canonical_projection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequiredProjection(Vec<String>);

impl RequiredProjection {
    pub(crate) fn new(
        fields: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, CognitionBindingError> {
        let mut fields: Vec<String> = fields.into_iter().map(Into::into).collect();
        if !is_canonical_projection(&fields) {
            return Err(CognitionBindingError::InvalidProjection);
        }
        fields.sort();
        Ok(Self(fields))
    }

    pub(crate) fn verify(&self, proof: &GovernedScanProof) -> Result<(), CognitionBindingError> {
        if self
            .0
            .iter()
            .any(|required| !proof.effective_projection().contains(required))
        {
            return Err(CognitionBindingError::ProjectionMismatch);
        }
        Ok(())
    }

    pub(crate) fn fields(&self) -> &[String] {
        &self.0
    }
}
