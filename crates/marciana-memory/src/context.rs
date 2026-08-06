//! Pure, content-free context planning over authorized candidate identities.

use chrono::{DateTime, Utc};
use grust_core::prelude::GraphMutationStore;
use sha2::{Digest, Sha256};
use typesec_core::policy::RequestContext;
use typesec_core::{CanRead, Capability, Resource};
use typesec_memory::{
    Label, MemoryError, MemoryId, MemoryKind, MemorySpace, MemoryVault, RecalledMemory, RedactedHit,
};

use crate::GraphStoreMemoryStore;

/// Closed views a planner may request from the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum ContextView {
    Assertions,
    Episodes,
    Summaries,
}

/// Closed retrieval policies; ranking implementations remain deployment-owned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum ContextRecipe {
    Ranked,
    Recent,
    CurrentAssertions,
}

/// Caller intent that is safe to hash and bind to a materialization receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallIntent {
    pub query_digest: String,
    /// Optional immutable working-set policy identity.
    pub working_set_digest: Option<String>,
    /// Bounded working-set slots that the planner must retain when candidates
    /// are available.
    pub pinned_memory_ids: Vec<MemoryId>,
    pub view: ContextView,
    pub recipe: ContextRecipe,
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
    pub considered_candidates: usize,
    pub estimated_tokens: u32,
    pub plan_digest: String,
}

/// Typed result of vault-authorized plan materialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBundle {
    pub plan_digest: String,
    /// Content-free digest of the authorized materialization result.
    pub receipt_digest: String,
    pub estimated_tokens: u32,
    pub token_budget: u32,
    pub truncated: bool,
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

/// Non-content explanation of how a context bundle was bounded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextExplanation {
    pub plan_digest: String,
    pub receipt_digest: String,
    pub requested_tokens: u32,
    pub estimated_tokens: u32,
    pub selected_candidates: usize,
    pub redacted_candidates: usize,
    pub truncated: bool,
}

/// A deterministic, typed view over one memory kind in a context bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSection {
    /// TypeSec memory taxonomy for this section.
    pub kind: MemoryKind,
    /// Authorized records in this section.
    pub memories: Vec<RecalledMemory>,
    /// Metadata-only records withheld by the clearance ceiling.
    pub redacted: Vec<RedactedHit>,
}

/// Fixed, non-content errors for context planning and plan verification.
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("context intent is invalid")]
    InvalidIntent,
    #[error("context candidate metadata is invalid")]
    InvalidCandidate,
    #[error("context plan exceeds its fixed candidate bound")]
    CandidateLimit,
    #[error("context plan token accounting is invalid")]
    TokenAccounting,
    #[error("context plan digest is invalid")]
    PlanDigest,
    #[error("working-set slot has no ranked context candidate")]
    PinnedCandidateMissing,
    #[error(transparent)]
    Memory(#[from] MemoryError),
}

impl ContextBundle {
    /// Group authorized and redacted results into stable TypeSec-kind sections.
    ///
    /// The fixed order is episodic, semantic, procedural, and profile. No
    /// section contains content that was not already returned by the vault.
    #[must_use]
    pub fn sections(&self) -> Vec<ContextSection> {
        [
            MemoryKind::Episodic,
            MemoryKind::Semantic,
            MemoryKind::Procedural,
            MemoryKind::Profile,
        ]
        .into_iter()
        .map(|kind| ContextSection {
            kind,
            memories: self
                .memories
                .iter()
                .filter(|memory| memory.kind == kind)
                .cloned()
                .collect(),
            redacted: self
                .redacted
                .iter()
                .filter(|memory| memory.kind == kind)
                .cloned()
                .collect(),
        })
        .collect()
    }

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

    /// Explain deterministic selection outcomes without exposing candidate content.
    #[must_use]
    pub fn explanation(&self) -> ContextExplanation {
        ContextExplanation {
            plan_digest: self.plan_digest.clone(),
            receipt_digest: self.receipt_digest.clone(),
            requested_tokens: self.token_budget,
            estimated_tokens: self.estimated_tokens,
            selected_candidates: self.memories.len() + self.redacted.len(),
            redacted_candidates: self.redacted.len(),
            truncated: self.truncated,
        }
    }
}

fn digest_serialized<T: serde::Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("provenance is serializable");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

impl RecallIntent {
    pub fn validate(&self) -> Result<(), ContextError> {
        if !self.query_digest.starts_with("sha256:") || self.query_digest.len() != 71 {
            return Err(ContextError::InvalidIntent);
        }
        if self.token_budget == 0 || self.token_budget > 64_000 {
            return Err(ContextError::InvalidIntent);
        }
        if self.pinned_memory_ids.len() > 64
            || self.pinned_memory_ids.iter().any(|id| {
                id.as_str().is_empty()
                    || id.as_str().len() > 256
                    || !id
                        .as_str()
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"_:/.-".contains(&byte))
            })
        {
            return Err(ContextError::InvalidIntent);
        }
        if self
            .working_set_digest
            .as_deref()
            .is_some_and(|digest| !is_digest(digest))
        {
            return Err(ContextError::InvalidIntent);
        }
        let mut ids = self
            .pinned_memory_ids
            .iter()
            .map(MemoryId::as_str)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        if ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(ContextError::InvalidIntent);
        }
        Ok(())
    }
}

impl ContextPlan {
    /// Verify the content-free integrity and token accounting of a plan.
    ///
    /// # Errors
    ///
    /// Returns a fixed [`ContextError`] when a caller modifies candidate
    /// metadata, ordering, accounting, or the plan digest.
    pub fn validate(&self) -> Result<(), ContextError> {
        self.intent.validate()?;
        validate_candidates(&self.candidates)?;
        validate_pinned_candidates(&self.intent, &self.candidates)?;
        if self.candidates.windows(2).any(|pair| {
            let left_pinned = is_pinned(&self.intent, &pair[0]);
            let right_pinned = is_pinned(&self.intent, &pair[1]);
            (!left_pinned && right_pinned)
                || (left_pinned == right_pinned
                    && (pair[0].score_basis_points < pair[1].score_basis_points
                        || (pair[0].score_basis_points == pair[1].score_basis_points
                            && pair[0].id > pair[1].id)))
        }) {
            return Err(ContextError::InvalidCandidate);
        }
        if self.considered_candidates < self.candidates.len() {
            return Err(ContextError::TokenAccounting);
        }
        let mut used = 0_u32;
        for candidate in &self.candidates {
            if candidate.estimated_tokens > self.intent.token_budget.saturating_sub(used) {
                return Err(ContextError::TokenAccounting);
            }
            used = used.saturating_add(candidate.estimated_tokens);
        }
        if used != self.estimated_tokens {
            return Err(ContextError::TokenAccounting);
        }
        let expected = context_plan_digest(&self.intent, &self.candidates, used);
        if expected != self.plan_digest {
            return Err(ContextError::PlanDigest);
        }
        Ok(())
    }
}

/// Plan without reading or disclosing candidate content.
pub fn plan_context(
    intent: RecallIntent,
    mut candidates: Vec<ContextCandidate>,
) -> Result<ContextPlan, ContextError> {
    intent.validate()?;
    if candidates.len() > 100_000 {
        return Err(ContextError::CandidateLimit);
    }
    validate_candidates(&candidates)?;
    validate_pinned_candidates(&intent, &candidates)?;
    candidates.sort_by(|left, right| {
        is_pinned(&intent, left)
            .cmp(&is_pinned(&intent, right))
            .reverse()
            .then_with(|| right.score_basis_points.cmp(&left.score_basis_points))
            .then_with(|| left.id.cmp(&right.id))
    });
    let considered_candidates = candidates.len();
    let mut used = 0_u32;
    candidates.retain(|candidate| {
        let fits = candidate.estimated_tokens <= intent.token_budget.saturating_sub(used);
        if fits {
            used = used.saturating_add(candidate.estimated_tokens);
        }
        fits
    });
    validate_pinned_candidates(&intent, &candidates).map_err(|_| ContextError::TokenAccounting)?;
    let digest = context_plan_digest(&intent, &candidates, used);
    Ok(ContextPlan {
        intent,
        candidates,
        considered_candidates,
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
) -> Result<ContextBundle, ContextError> {
    plan.validate()?;
    let ids = plan.candidates.iter().map(|candidate| candidate.id.clone());
    let (memories, redacted) =
        vault.recall_ids_at(space, capability, ids, plan.intent.as_of, ceiling, context)?;
    let receipt_digest = context_materialization_digest(
        &plan.plan_digest,
        space.resource_id(),
        ceiling,
        context.purpose.as_deref(),
        &memories,
        &redacted,
    );
    Ok(ContextBundle {
        plan_digest: plan.plan_digest.clone(),
        receipt_digest,
        estimated_tokens: plan.estimated_tokens,
        token_budget: plan.intent.token_budget,
        truncated: plan.candidates.len() < plan.considered_candidates,
        memories,
        redacted,
    })
}

fn context_materialization_digest(
    plan_digest: &str,
    space_id: &str,
    ceiling: Label,
    purpose: Option<&str>,
    memories: &[RecalledMemory],
    redacted: &[RedactedHit],
) -> String {
    let mut visible = memories
        .iter()
        .map(|memory| memory.id.as_str())
        .collect::<Vec<_>>();
    let mut withheld = redacted
        .iter()
        .map(|memory| memory.id.as_str())
        .collect::<Vec<_>>();
    visible.sort_unstable();
    withheld.sort_unstable();
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.context-materialization.v1\0");
    hasher.update(plan_digest.as_bytes());
    hasher.update(b"space\0");
    hasher.update(space_id.as_bytes());
    hasher.update(b"ceiling\0");
    hasher.update(ceiling.name().as_bytes());
    hasher.update(b"purpose\0");
    hasher.update(purpose.unwrap_or_default().as_bytes());
    hasher.update(b"visible\0");
    for id in visible {
        hasher.update(id.as_bytes());
        hasher.update([0]);
    }
    hasher.update(b"redacted\0");
    for id in withheld {
        hasher.update(id.as_bytes());
        hasher.update([0]);
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn validate_candidates(candidates: &[ContextCandidate]) -> Result<(), ContextError> {
    let mut ids = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if candidate.estimated_tokens == 0
            || candidate.reason_digest.len() != 71
            || !candidate.reason_digest.starts_with("sha256:")
        {
            return Err(ContextError::InvalidCandidate);
        }
        ids.push(candidate.id.as_str());
    }
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ContextError::InvalidCandidate);
    }
    Ok(())
}

fn context_plan_digest(
    intent: &RecallIntent,
    candidates: &[ContextCandidate],
    estimated_tokens: u32,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.context-plan.v1\0");
    hasher.update(intent.query_digest.as_bytes());
    hasher.update(b"working-set\0");
    hasher.update(
        intent
            .working_set_digest
            .as_deref()
            .unwrap_or_default()
            .as_bytes(),
    );
    hasher.update(b"pinned\0");
    let mut pinned = intent
        .pinned_memory_ids
        .iter()
        .map(MemoryId::as_str)
        .collect::<Vec<_>>();
    pinned.sort_unstable();
    for id in pinned {
        hasher.update(id.as_bytes());
        hasher.update([0]);
    }
    hasher.update(
        format!(
            "{:?}|{:?}|{}|{}",
            intent.view, intent.recipe, intent.as_of, estimated_tokens
        )
        .as_bytes(),
    );
    for candidate in candidates {
        hasher.update(candidate.id.as_str().as_bytes());
        hasher.update(candidate.score_basis_points.to_be_bytes());
        hasher.update(candidate.estimated_tokens.to_be_bytes());
        hasher.update(candidate.reason_digest.as_bytes());
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn validate_pinned_candidates(
    intent: &RecallIntent,
    candidates: &[ContextCandidate],
) -> Result<(), ContextError> {
    for pinned in &intent.pinned_memory_ids {
        if !candidates.iter().any(|candidate| candidate.id == *pinned) {
            return Err(ContextError::PinnedCandidateMissing);
        }
    }
    Ok(())
}

fn is_pinned(intent: &RecallIntent, candidate: &ContextCandidate) -> bool {
    intent
        .pinned_memory_ids
        .iter()
        .any(|id| id == &candidate.id)
}

fn is_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}
