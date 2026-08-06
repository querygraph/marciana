//! Pure, content-free context planning over authorized candidate identities.

use chrono::{DateTime, Utc};
use grust_core::prelude::GraphMutationStore;
use sha2::{Digest, Sha256};
use typesec_core::policy::RequestContext;
use typesec_core::{CanRead, Capability};
use typesec_memory::{
    Label, MemoryError, MemoryId, MemorySpace, MemoryVault, RecalledMemory, RedactedHit,
};

use crate::GraphStoreMemoryStore;

/// Closed views a planner may request from the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum ContextView {
    Assertions,
    Episodes,
    Summaries,
}

/// Caller intent that is safe to hash and bind to a materialization receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallIntent {
    pub query_digest: String,
    pub view: ContextView,
    pub as_of: DateTime<Utc>,
    pub token_budget: u32,
}

/// Candidate metadata supplied by an index; it contains no protected content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextCandidate {
    pub id: MemoryId,
    pub score_basis_points: u32,
    pub estimated_tokens: u32,
    pub reason_digest: String,
}

/// Deterministic selection result; vault materialization must apply visibility gates again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextPlan {
    pub intent: RecallIntent,
    pub candidates: Vec<ContextCandidate>,
    pub estimated_tokens: u32,
    pub plan_digest: String,
}

/// Typed result of vault-authorized plan materialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBundle {
    pub plan_digest: String,
    pub estimated_tokens: u32,
    pub memories: Vec<RecalledMemory>,
    pub redacted: Vec<RedactedHit>,
}

/// Traceability metadata for one visible or redacted candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextCitation {
    pub id: MemoryId,
    pub valid_from: Option<DateTime<Utc>>,
    pub provenance_digest: Option<String>,
    pub redacted: bool,
}

impl ContextBundle {
    /// Return stable citations without exposing a redacted candidate's content.
    #[must_use]
    pub fn citations(&self) -> Vec<ContextCitation> {
        let mut citations = self
            .memories
            .iter()
            .map(|memory| ContextCitation {
                id: memory.id.clone(),
                valid_from: Some(memory.valid_from),
                provenance_digest: Some(digest_serialized(&memory.provenance)),
                redacted: false,
            })
            .chain(self.redacted.iter().map(|memory| ContextCitation {
                id: memory.id.clone(),
                valid_from: None,
                provenance_digest: None,
                redacted: true,
            }))
            .collect::<Vec<_>>();
        citations.sort_by(|left, right| left.id.cmp(&right.id));
        citations
    }
}

fn digest_serialized<T: serde::Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("provenance is serializable");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

impl RecallIntent {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.query_digest.starts_with("sha256:") || self.query_digest.len() != 71 {
            return Err("query digest must be canonical SHA-256");
        }
        if self.token_budget == 0 || self.token_budget > 64_000 {
            return Err("token budget is outside its fixed bound");
        }
        Ok(())
    }
}

/// Plan without reading or disclosing candidate content.
pub fn plan_context(
    intent: RecallIntent,
    mut candidates: Vec<ContextCandidate>,
) -> Result<ContextPlan, &'static str> {
    intent.validate()?;
    if candidates.len() > 100_000 {
        return Err("candidate set exceeds its fixed bound");
    }
    if candidates.iter().any(|candidate| {
        candidate.estimated_tokens == 0
            || candidate.reason_digest.len() != 71
            || !candidate.reason_digest.starts_with("sha256:")
    }) {
        return Err("candidate metadata is not canonical");
    }
    candidates.sort_by(|left, right| {
        right
            .score_basis_points
            .cmp(&left.score_basis_points)
            .then_with(|| left.id.cmp(&right.id))
    });
    let mut used = 0_u32;
    candidates.retain(|candidate| {
        let fits = candidate.estimated_tokens <= intent.token_budget.saturating_sub(used);
        if fits {
            used = used.saturating_add(candidate.estimated_tokens);
        }
        fits
    });
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.context-plan.v1\0");
    hasher.update(intent.query_digest.as_bytes());
    hasher.update(format!("{:?}|{}|{}", intent.view, intent.as_of, used).as_bytes());
    for candidate in &candidates {
        hasher.update(candidate.id.as_str().as_bytes());
        hasher.update(candidate.score_basis_points.to_be_bytes());
        hasher.update(candidate.estimated_tokens.to_be_bytes());
        hasher.update(candidate.reason_digest.as_bytes());
    }
    let digest = format!("sha256:{:x}", hasher.finalize());
    Ok(ContextPlan {
        intent,
        candidates,
        estimated_tokens: used,
        plan_digest: digest,
    })
}

/// Materialize only the IDs selected by a plan through TypeSec's visibility gate.
pub fn materialize_context_plan<G: GraphMutationStore>(
    vault: &MemoryVault<GraphStoreMemoryStore<G>>,
    space: &MemorySpace,
    capability: &Capability<CanRead, MemorySpace>,
    plan: &ContextPlan,
    ceiling: Label,
    context: &RequestContext,
) -> Result<ContextBundle, MemoryError> {
    let ids = plan.candidates.iter().map(|candidate| candidate.id.clone());
    let (memories, redacted) =
        vault.recall_ids_at(space, capability, ids, plan.intent.as_of, ceiling, context)?;
    Ok(ContextBundle {
        plan_digest: plan.plan_digest.clone(),
        estimated_tokens: plan.estimated_tokens,
        memories,
        redacted,
    })
}
