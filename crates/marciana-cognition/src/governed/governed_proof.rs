//! Bounded validation for LakeCat proof values crossing into cognition.

use crate::CognitionBindingError;
use lakecat_core::governed_scan::{
    GovernedScanCatalogIdentity, GovernedScanDigests, GovernedScanProof,
};
use marciana_catalog::{LakeCatCognitionSourceError, validate_governed_cognition_proof};

/// Translate Marciana-owned proof-boundary errors into QueryGraph's public
/// application categories.
pub(crate) fn validate_governed_proof(
    catalog: &GovernedScanCatalogIdentity,
    proof: &GovernedScanProof,
) -> Result<GovernedScanDigests, CognitionBindingError> {
    validate_governed_cognition_proof(catalog, proof).map_err(|error| match error {
        LakeCatCognitionSourceError::CatalogMismatch => CognitionBindingError::CatalogMismatch,
        LakeCatCognitionSourceError::InvalidProof => CognitionBindingError::InvalidProof,
    })
}
