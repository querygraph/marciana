//! `LakeCat` adapters owned by Marciana's governed cognition boundary.
//!
//! `LakeCat` remains authoritative for proof validation and canonical digests.
//! This crate translates its opaque, already-verified proof into the compact
//! source identity consumed by Marciana cognition; it does not expose a Sail
//! plan token or authorization receipt.

use lakecat_core::governed_scan::{
    GovernedScanCatalogIdentity, GovernedScanDigests, GovernedScanProof, governed_scan_digests,
};
use querygraph_memory::cognition::{GovernedLakeCatSnapshot, MAX_COGNITION_IDENTITY_BYTES};

/// A `LakeCat` proof could not be represented as a valid cognition source.
#[derive(Debug, thiserror::Error)]
pub enum LakeCatCognitionSourceError {
    /// The proof belongs to a catalog other than the configured cognition catalog.
    #[error("governed LakeCat catalog does not match cognition configuration")]
    CatalogMismatch,
    /// LakeCat-owned proof material produced an invalid source identity.
    #[error("invalid governed LakeCat cognition source")]
    InvalidProof,
}

/// Validate LakeCat-owned proof material at Marciana's cognition boundary.
///
/// `LakeCat` validates its own proof and digest canonicalization. Marciana adds
/// the stricter flattened table-identity budget required by its cognition
/// source schema.
///
/// # Errors
///
/// Returns [`LakeCatCognitionSourceError::CatalogMismatch`] for a configured
/// catalog mismatch and [`LakeCatCognitionSourceError::InvalidProof`] for an
/// invalid proof or an over-budget cognition table identity.
pub fn validate_governed_cognition_proof(
    catalog: &GovernedScanCatalogIdentity,
    proof: &GovernedScanProof,
) -> Result<GovernedScanDigests, LakeCatCognitionSourceError> {
    if proof.catalog_identity() != catalog {
        return Err(LakeCatCognitionSourceError::CatalogMismatch);
    }
    let digests =
        governed_scan_digests(proof).map_err(|_| LakeCatCognitionSourceError::InvalidProof)?;
    if !bounded_table_identity(
        proof.table().warehouse.as_str(),
        proof.table().namespace.parts(),
        proof.table().name.as_str(),
    ) {
        return Err(LakeCatCognitionSourceError::InvalidProof);
    }
    Ok(digests)
}

/// Translate LakeCat-owned proof material into Marciana's cognition source.
///
/// The returned value contains only the identities needed to bind the
/// proposal. The raw Sail plan token and authorization receipt remain owned by
/// `LakeCat` and never cross this adapter.
///
/// # Errors
///
/// Returns [`LakeCatCognitionSourceError::InvalidProof`] when the canonical
/// source identity cannot be represented safely.
pub fn governed_cognition_source(
    proof: &GovernedScanProof,
    digests: &GovernedScanDigests,
    staged_projection: &[String],
) -> Result<GovernedLakeCatSnapshot, LakeCatCognitionSourceError> {
    let source = GovernedLakeCatSnapshot {
        snapshot_digest: digests.snapshot_digest().to_owned(),
        governed_scan_digest: proof.grant_id().to_owned(),
        catalog: proof.catalog_identity().as_str().to_owned(),
        namespace: proof.table().namespace.to_string(),
        table: proof.table().name.to_string(),
        snapshot_id: proof.snapshot_id(),
        plan_task_digest: proof.plan_task_digest().to_owned(),
        subject: proof.principal_subject().to_owned(),
        purpose: proof.purpose().to_owned(),
        effective_projection: staged_projection.to_vec(),
        authorization_receipt_digest: proof.authorization_receipt_digest().to_owned(),
    };
    source
        .digest()
        .map_err(|_| LakeCatCognitionSourceError::InvalidProof)?;
    Ok(source)
}

fn bounded_table_identity(warehouse: &str, namespace: &[String], table: &str) -> bool {
    std::iter::once(warehouse)
        .chain(namespace.iter().map(String::as_str))
        .chain(std::iter::once(table))
        .try_fold((0usize, 0usize), |(bytes, count), part| {
            let separator = usize::from(count > 0);
            Some((
                bytes.checked_add(separator)?.checked_add(part.len())?,
                count.checked_add(1)?,
            ))
        })
        .is_some_and(|(bytes, _)| bytes <= MAX_COGNITION_IDENTITY_BYTES)
}
