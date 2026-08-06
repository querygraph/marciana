//! `LakeCat` adapters owned by Marciana's governed cognition boundary.
//!
//! `LakeCat` remains authoritative for proof validation and canonical digests.
//! This crate translates its opaque, already-verified proof into the compact
//! source identity consumed by Marciana cognition; it does not expose a Sail
//! plan token or authorization receipt.

use lakecat_core::governed_scan::{GovernedScanDigests, GovernedScanProof};
use querygraph_memory::cognition::GovernedLakeCatSnapshot;

/// A `LakeCat` proof could not be represented as a valid cognition source.
#[derive(Debug, thiserror::Error)]
pub enum LakeCatCognitionSourceError {
    /// LakeCat-owned proof material produced an invalid source identity.
    #[error("invalid governed LakeCat cognition source")]
    InvalidProof,
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
