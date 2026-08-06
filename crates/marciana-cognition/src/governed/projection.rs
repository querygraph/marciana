//! Exact canonical cognition projection.

use crate::CognitionBindingError;
use lakecat_core::governed_scan::GovernedScanProof;

use super::intent::is_canonical_identity;
use querygraph_memory::cognition::{
    MAX_COGNITION_PROJECTION_BYTES, MAX_COGNITION_PROJECTION_FIELDS,
};

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

pub(crate) fn is_canonical_projection(fields: &[String]) -> bool {
    if fields.is_empty() || fields.len() > MAX_COGNITION_PROJECTION_FIELDS {
        return false;
    }
    let total = fields.iter().try_fold(0usize, |total, field| {
        is_canonical_identity(field).then_some(())?;
        total.checked_add(field.len())
    });
    total.is_some_and(|bytes| bytes <= MAX_COGNITION_PROJECTION_BYTES)
        && fields
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == fields.len()
}
