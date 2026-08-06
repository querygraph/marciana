//! Capability-bound execution of the validated four-verb API contracts.

use typesec_core::policy::RequestContext;
use typesec_core::{CanDelete, CanRead, CanWrite, Capability, Resource};
use typesec_memory::{Label, MemoryError, MemoryId, MemorySpace, MemoryStore, MemoryVault};

use crate::api::{ApiError, ForgetRequest, ImproveRequest, RecallRequest, RememberRequest};

/// Public facade failures keep validation separate from vault failures.
#[derive(Debug, thiserror::Error)]
pub enum FacadeError {
    #[error(transparent)]
    Validation(#[from] ApiError),
    #[error(transparent)]
    Vault(#[from] MemoryError),
    #[error("memory request targets another space")]
    SpaceMismatch,
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
    pub fn improve(
        &self,
        cap: &Capability<CanWrite, MemorySpace>,
        request: ImproveRequest,
    ) -> Result<MemoryId, FacadeError> {
        self.check_space(&request.space_id)?;
        let draft = request.replacement_draft()?;
        self.vault
            .remember(self.space, cap, draft)
            .map_err(FacadeError::Vault)
    }

    /// Execute scoped forgetting through the vault tombstone path.
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
