//! Vault-authorized materialization of assertion-ranked record identifiers.

use chrono::{DateTime, Utc};
use grust_core::prelude::GraphMutationStore;
use marciana_ledger::AssertionQuery;
use typesec_core::policy::RequestContext;
use typesec_core::{CanRead, Capability};
use typesec_memory::{Label, MemoryError, MemorySpace, MemoryVault, RecalledMemory, RedactedHit};

use crate::GraphStoreMemoryStore;

/// Materialize assertion-ranked candidates through `TypeSec`'s one authorized
/// read path. The graph supplies IDs only; the vault supplies all content and
/// redaction decisions.
///
/// # Errors
///
/// Returns a fixed backend [`StoreError`] when assertion projections cannot
/// be read or decoded; diagnostics never include record values.
pub fn recall_assertions_at<G: GraphMutationStore>(
    vault: &MemoryVault<GraphStoreMemoryStore<G>>,
    space: &MemorySpace,
    capability: &Capability<CanRead, MemorySpace>,
    query: &AssertionQuery,
    at: DateTime<Utc>,
    ceiling: Label,
    context: &RequestContext,
) -> Result<(Vec<RecalledMemory>, Vec<RedactedHit>), MemoryError> {
    let ids = vault
        .store()
        .assertion_candidate_ids(query)
        .map_err(MemoryError::Store)?;
    vault.recall_ids_at(space, capability, ids, at, ceiling, context)
}
