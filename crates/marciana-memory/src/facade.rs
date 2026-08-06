//! Capability-bound execution of the validated four-verb API contracts.

use grust_core::prelude::GraphMutationStore;
use typesec_core::policy::RequestContext;
use typesec_core::{CanDelete, CanRead, CanWrite, Capability, Resource};
use typesec_memory::{
    ConsolidationPlan, ConsolidationStep, Label, MemoryError, MemoryId, MemorySpace, MemoryStore,
    MemoryVault,
};

use crate::GraphStoreMemoryStore;
use crate::api::{ApiError, ForgetRequest, ImproveRequest, RecallRequest, RememberRequest};
use crate::context::{
    ContextBundle, ContextCandidate, ContextError, ContextPlan, RecallIntent, plan_context,
};
use crate::session::{RecallContextMetadata, SessionMetadata, ThreadMetadata};

/// Public facade failures keep validation separate from vault failures.
#[derive(Debug, thiserror::Error)]
pub enum FacadeError {
    #[error(transparent)]
    Validation(#[from] ApiError),
    #[error(transparent)]
    Vault(#[from] MemoryError),
    #[error(transparent)]
    Context(#[from] ContextError),
    #[error("memory request targets another space")]
    SpaceMismatch,
}

impl<G> MemoryFacade<'_, GraphStoreMemoryStore<G>>
where
    G: GraphMutationStore,
{
    /// Materialize a verified context plan through the bound `TypeSec` vault.
    ///
    /// This method is available only for Marciana's Graph/Sail-backed store;
    /// planning remains backend-independent and authorization remains owned by
    /// [`MemoryVault`].
    ///
    /// # Errors
    ///
    /// Returns [`FacadeError`] when the intent is out of scope or invalid, or
    /// the vault denies the recall.
    pub fn materialize_context(
        &self,
        cap: &Capability<CanRead, MemorySpace>,
        plan: &ContextPlan,
        ceiling: Label,
        context: &RequestContext,
    ) -> Result<ContextBundle, FacadeError> {
        crate::context::materialize_context_plan(
            self.vault, self.space, cap, plan, ceiling, context,
        )
        .map_err(FacadeError::Context)
    }

    /// Bind product session metadata, plan, and materialize through the same
    /// capability gate. A session selects only the facade's space and recall
    /// identity; it cannot mint or widen a capability.
    ///
    /// # Errors
    ///
    /// Returns [`FacadeError`] when the session metadata or intent is
    /// invalid, out of scope, or denied by the vault.
    pub fn materialize_context_for_session(
        &self,
        cap: &Capability<CanRead, MemorySpace>,
        session: &SessionMetadata,
        intent: RecallIntent,
        candidates: Vec<ContextCandidate>,
        ceiling: Label,
        context: &RequestContext,
    ) -> Result<ContextBundle, FacadeError> {
        self.materialize_context_for_metadata(cap, session, intent, candidates, ceiling, context)
    }

    /// Bind thread metadata and materialize through the same capability gate.
    ///
    /// # Errors
    ///
    /// Returns [`FacadeError`] when the thread metadata or intent is invalid,
    /// out of scope, or denied by the vault.
    pub fn materialize_context_for_thread(
        &self,
        cap: &Capability<CanRead, MemorySpace>,
        thread: &ThreadMetadata,
        intent: RecallIntent,
        candidates: Vec<ContextCandidate>,
        ceiling: Label,
        context: &RequestContext,
    ) -> Result<ContextBundle, FacadeError> {
        self.materialize_context_for_metadata(cap, thread, intent, candidates, ceiling, context)
    }

    fn materialize_context_for_metadata<M: RecallContextMetadata>(
        &self,
        cap: &Capability<CanRead, MemorySpace>,
        metadata: &M,
        intent: RecallIntent,
        candidates: Vec<ContextCandidate>,
        ceiling: Label,
        context: &RequestContext,
    ) -> Result<ContextBundle, FacadeError> {
        if metadata.space_id() != self.space.resource_id() {
            return Err(FacadeError::SpaceMismatch);
        }
        let bound = metadata
            .bind_intent(intent)
            .map_err(|_| FacadeError::Context(ContextError::InvalidIntent))?;
        let plan = plan_context(bound, candidates).map_err(FacadeError::Context)?;
        self.materialize_context(cap, &plan, ceiling, context)
    }
}

/// Thin execution facade; all authority remains in [`MemoryVault`].
pub struct MemoryFacade<'a, S: MemoryStore> {
    vault: &'a MemoryVault<S>,
    space: &'a MemorySpace,
}

impl<'a, S: MemoryStore> MemoryFacade<'a, S> {
    fn check_space(&self, request_space: &str) -> Result<(), FacadeError> {
        if request_space == self.space.resource_id() {
            Ok(())
        } else {
            Err(FacadeError::SpaceMismatch)
        }
    }

    /// Bind one facade instance to one vault and one memory space.
    #[must_use]
    pub fn new(vault: &'a MemoryVault<S>, space: &'a MemorySpace) -> Self {
        Self { vault, space }
    }

    /// Execute remember after capability authorization in the vault.
    ///
    /// # Errors
    ///
    /// Returns [`FacadeError`] when the request is out of scope or invalid,
    /// or the vault denies the write.
    // All four verbs take owned requests so the facade signature stays
    // uniform and free to consume request fields later without breaking
    // callers.
    #[allow(clippy::needless_pass_by_value)]
    pub fn remember(
        &self,
        cap: &Capability<CanWrite, MemorySpace>,
        request: RememberRequest,
    ) -> Result<MemoryId, FacadeError> {
        self.check_space(&request.space_id)?;
        let draft = request.to_draft()?;
        self.vault
            .remember(self.space, cap, draft)
            .map_err(FacadeError::Vault)
    }

    /// Execute recall through the vault's runtime clearance gate.
    ///
    /// # Errors
    ///
    /// Returns [`FacadeError`] when the request is out of scope or invalid,
    /// or the vault denies the recall.
    pub fn recall(
        &self,
        cap: &Capability<CanRead, MemorySpace>,
        request: RecallRequest,
    ) -> Result<
        (
            Vec<typesec_memory::RecalledMemory>,
            Vec<typesec_memory::RedactedHit>,
        ),
        FacadeError,
    > {
        self.check_space(&request.space_id)?;
        let query = request.to_query()?;
        let context = RequestContext::default().with_purpose(request.purpose);
        self.vault
            .recall_at(self.space, cap, query, &context, Label::Internal)
            .map_err(FacadeError::Vault)
    }

    /// Execute improve as a vault-authorized new draft; history is retained.
    ///
    /// # Errors
    ///
    /// Returns [`FacadeError`] when the request is out of scope or invalid,
    /// or the vault denies the supersession.
    pub fn improve(
        &self,
        cap: &Capability<CanWrite, MemorySpace>,
        request: ImproveRequest,
    ) -> Result<MemoryId, FacadeError> {
        self.check_space(&request.space_id)?;
        let draft = request.replacement_draft()?;
        let report = self
            .vault
            .consolidate(
                self.space,
                cap,
                ConsolidationPlan {
                    steps: vec![ConsolidationStep::Supersede {
                        superseded: vec![MemoryId::from_string(request.memory_id)],
                        replacement: draft,
                    }],
                },
            )
            .map_err(FacadeError::Vault)?;
        report
            .created
            .into_iter()
            .next()
            .ok_or(FacadeError::Vault(MemoryError::NotFound(
                "improvement produced no replacement".into(),
            )))
    }

    /// Execute scoped forgetting through the vault tombstone path.
    ///
    /// # Errors
    ///
    /// Returns [`FacadeError`] when the request is out of scope or invalid,
    /// or the vault denies the tombstone.
    // All four verbs take owned requests so the facade signature stays
    // uniform and free to consume request fields later without breaking
    // callers.
    #[allow(clippy::needless_pass_by_value)]
    pub fn forget(
        &self,
        cap: &Capability<CanDelete, MemorySpace>,
        request: ForgetRequest,
    ) -> Result<typesec_memory::Tombstone, FacadeError> {
        self.check_space(&request.space_id)?;
        let selector = request.to_selector()?;
        self.vault
            .forget(self.space, cap, selector)
            .map_err(FacadeError::Vault)
    }
}
