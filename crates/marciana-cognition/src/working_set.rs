//! Proposal-only working-set policies for governed context assembly.

use chrono::{DateTime, Utc};
use querygraph_memory::context::{ContextRecipe, ContextView, RecallIntent};
use sha2::{Digest, Sha256};
use typesec_memory::MemoryId;

const MAX_SLOTS: usize = 64;
const MAX_POLICY_ID: usize = 256;
const MAX_TOKEN_BUDGET: u32 = 64_000;

/// Origin of a working-set proposal. Neither origin grants authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingSetSource {
    Operator,
    AgentProposal,
}

/// Lifecycle for a bounded working-set policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingSetStatus {
    Proposed,
    Approved,
    Active,
    Revoked,
}

/// One digest-only slot pointing at a governed memory object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingSetSlot {
    pub memory_id: MemoryId,
}

/// Saved context policy with bounded governed-memory slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingSet {
    pub working_set_digest: String,
    pub space_id: String,
    pub policy_digest: String,
    pub view: ContextView,
    pub recipe: ContextRecipe,
    pub token_budget: u32,
    pub slots: Vec<WorkingSetSlot>,
    pub source: WorkingSetSource,
    pub status: WorkingSetStatus,
}

impl WorkingSet {
    /// Create a digest-only proposal. It does not read or authorize memory.
    ///
    /// # Errors
    /// Returns a fixed error when policy identities, slots, or budgets are
    /// outside their bounds.
    pub fn propose(
        space_id: String,
        policy_digest: String,
        view: ContextView,
        recipe: ContextRecipe,
        token_budget: u32,
        slots: Vec<WorkingSetSlot>,
        source: WorkingSetSource,
    ) -> Result<Self, WorkingSetError> {
        let slot_ids = validate_policy(&space_id, &policy_digest, token_budget, &slots)?;
        let working_set_digest = working_set_digest(
            &space_id,
            &policy_digest,
            view,
            recipe,
            token_budget,
            &slot_ids,
            source,
        );
        Ok(Self {
            working_set_digest,
            space_id,
            policy_digest,
            view,
            recipe,
            token_budget,
            slots,
            source,
            status: WorkingSetStatus::Proposed,
        })
    }

    /// Verify that public metadata still matches the immutable policy digest.
    ///
    /// # Errors
    /// Returns a fixed error when a field or slot was modified after proposal.
    pub fn validate(&self) -> Result<(), WorkingSetError> {
        let slot_ids = validate_policy(
            &self.space_id,
            &self.policy_digest,
            self.token_budget,
            &self.slots,
        )?;
        if working_set_digest(
            &self.space_id,
            &self.policy_digest,
            self.view,
            self.recipe,
            self.token_budget,
            &slot_ids,
            self.source,
        ) != self.working_set_digest
        {
            return Err(WorkingSetError::Digest);
        }
        Ok(())
    }

    /// Approve a proposal without widening its slots or policy.
    ///
    /// # Errors
    /// Returns [`WorkingSetError`] when the policy is tampered with or is not
    /// in the proposed state.
    pub fn approve(&mut self) -> Result<(), WorkingSetError> {
        self.validate()?;
        if self.status != WorkingSetStatus::Proposed {
            return Err(WorkingSetError::Transition);
        }
        self.status = WorkingSetStatus::Approved;
        Ok(())
    }

    /// Activate an approved policy. This does not mint a capability.
    ///
    /// # Errors
    /// Returns [`WorkingSetError`] when the policy is tampered with or is not
    /// approved.
    pub fn activate(&mut self) -> Result<(), WorkingSetError> {
        self.validate()?;
        if self.status != WorkingSetStatus::Approved {
            return Err(WorkingSetError::Transition);
        }
        self.status = WorkingSetStatus::Active;
        Ok(())
    }

    /// Revoke an active policy.
    ///
    /// # Errors
    /// Returns [`WorkingSetError`] when the policy is tampered with or is not
    /// active.
    pub fn revoke(&mut self) -> Result<(), WorkingSetError> {
        self.validate()?;
        if self.status != WorkingSetStatus::Active {
            return Err(WorkingSetError::Transition);
        }
        self.status = WorkingSetStatus::Revoked;
        Ok(())
    }

    /// Compile an active policy into a content-free recall intent.
    ///
    /// The returned intent still requires a capability-bound vault
    /// materialization call; a working set is never an authorization bypass.
    ///
    /// # Errors
    /// Returns [`WorkingSetError`] when the policy is invalid, inactive, or
    /// the query identity is not a canonical digest.
    pub fn recall_intent(
        &self,
        query_digest: String,
        as_of: DateTime<Utc>,
    ) -> Result<RecallIntent, WorkingSetError> {
        self.validate()?;
        if self.status != WorkingSetStatus::Active || !is_digest(&query_digest) {
            return Err(WorkingSetError::Transition);
        }
        Ok(RecallIntent {
            query_digest,
            working_set_digest: Some(self.working_set_digest.clone()),
            pinned_memory_ids: self
                .slots
                .iter()
                .map(|slot| slot.memory_id.clone())
                .collect(),
            view: self.view,
            recipe: self.recipe,
            as_of,
            token_budget: self.token_budget,
        })
    }
}

/// Fixed working-set validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum WorkingSetError {
    #[error("working-set policy is out of bounds")]
    Bounds,
    #[error("working-set slot identity is invalid")]
    Slot,
    #[error("working-set policy digest is invalid")]
    Digest,
    #[error("working-set lifecycle transition is not permitted")]
    Transition,
}

fn validate_policy<'a>(
    space_id: &str,
    policy_digest: &str,
    token_budget: u32,
    slots: &'a [WorkingSetSlot],
) -> Result<Vec<&'a str>, WorkingSetError> {
    if space_id.is_empty()
        || space_id.len() > MAX_POLICY_ID
        || policy_digest.len() > MAX_POLICY_ID
        || token_budget == 0
        || token_budget > MAX_TOKEN_BUDGET
        || slots.len() > MAX_SLOTS
        || !is_digest(policy_digest)
        || !space_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_:/.-".contains(&byte))
    {
        return Err(WorkingSetError::Bounds);
    }
    let mut ids = slots
        .iter()
        .map(|slot| slot.memory_id.as_str())
        .collect::<Vec<_>>();
    if ids
        .iter()
        .any(|id| id.is_empty() || id.len() > MAX_POLICY_ID)
    {
        return Err(WorkingSetError::Slot);
    }
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(WorkingSetError::Slot);
    }
    Ok(ids)
}

fn working_set_digest(
    space_id: &str,
    policy_digest: &str,
    view: ContextView,
    recipe: ContextRecipe,
    token_budget: u32,
    ids: &[&str],
    source: WorkingSetSource,
) -> String {
    let canonical = format!(
        "working-set-v1|{space_id}|{policy_digest}|{view:?}|{recipe:?}|{token_budget}|{ids:?}|{source:?}"
    );
    format!("sha256:{:x}", Sha256::digest(canonical.as_bytes()))
}

fn is_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}
