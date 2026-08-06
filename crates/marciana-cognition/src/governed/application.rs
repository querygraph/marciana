//! End-to-end governed proposal binding, application, and receipt issuance.

use std::fmt;
use std::sync::{Arc, OnceLock};

use crate::{CognitionBindingError, CognitionEngineBinding, CognitionMemoryError};
use chrono::TimeDelta;
use lakecat_core::governed_scan::GovernedScanProof;
use querygraph_memory::cognition::{CognitionError, CognitionFieldMapping, CognitionRequest};
use tokio::sync::Mutex;
use typesec_core::{CanRead, CanWrite, Capability};
use typesec_integrations::{
    CognitionCommitReceipt, ReceiptError, ReceiptIssuer, VerifiedTypeDidContext,
};
use typesec_memory::{
    CognitionAuthorityError, CognitionCommitOutcome, CognitionCommitStore, CognitionProposal,
    Label, MemoryError, MemoryId, MemorySpace, MemoryVault,
};

use super::authority::{
    LakeCatAuthorityError, LakeCatCognitionAuthority, PrimedAuthorityVerifier,
    validate_fresh_authority,
};
use super::binding::BindingBasis;
use super::clock::{CognitionClock, SystemCognitionClock};
use super::proposal::{exact_manifest_ids, validate_planned_proposal, validate_proposal_intent};
use super::receipt::{CognitionReceiptSigner, ReceiptBasis};

/// Governed cognition failed before or during the authoritative commit.
#[derive(Debug, thiserror::Error)]
pub enum CognitionApplicationError {
    /// Verified identity, intent, proof, projection, or source binding was invalid.
    #[error(transparent)]
    Binding(#[from] CognitionBindingError),
    /// TypeSec denied, rejected, or could not complete protected memory work.
    #[error(transparent)]
    Memory(#[from] CognitionMemoryError),
    /// LakeCat made a typed denial or could not answer the authority check.
    #[error(transparent)]
    Authority(#[from] LakeCatAuthorityError),
    /// The one-use TypeSec authority bridge became unavailable.
    #[error("cognition authority bridge failed: {0}")]
    AuthorityState(#[from] CognitionAuthorityError),
    /// Grust or Sail rejected the exact governed planning input.
    #[error(transparent)]
    Cognition(#[from] CognitionError),
    /// Commit receipt claims could not be validated or signed.
    #[error(transparent)]
    Receipt(#[from] ReceiptError),
}

impl From<MemoryError> for CognitionApplicationError {
    fn from(error: MemoryError) -> Self {
        Self::Memory(error.into())
    }
}

/// Authoritative application result plus its portable signed evidence.
pub struct GovernedCognitionResult {
    /// Applied or idempotently recovered TypeSec commit.
    pub outcome: CognitionCommitOutcome,
    /// Claims signed into `receipt_token`.
    pub receipt: CognitionCommitReceipt,
    /// Deterministic Ed25519-signed commit receipt.
    pub receipt_token: String,
}

impl fmt::Debug for GovernedCognitionResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GovernedCognitionResult")
            .field("status", &self.outcome.status)
            .field("effect", &self.outcome.effect)
            .field("evidence", &"<redacted>")
            .finish_non_exhaustive()
    }
}

/// Immutable inputs selecting one governed cognition application context.
#[derive(Debug, Clone)]
pub struct GovernedCognitionConfig {
    /// Exact TypeSec memory space that the proposal may mutate.
    pub space: MemorySpace,
    /// Integrity-bound LakeCat scan grant presented by the worker.
    pub proof: GovernedScanProof,
    /// Exact source IDs, canonicalized and bound by the signed TypeDID intent.
    /// Every record must already have passed TypeSec governed ingestion under
    /// the LakeCat source scope derived from `proof`; local or mixed records
    /// fail before protected content reaches the engine.
    pub source_ids: Vec<MemoryId>,
    /// Exact LakeCat columns staged as cognition ID, text, and validity time.
    pub field_mapping: CognitionFieldMapping,
    /// Maximum clearance granted by trusted deployment policy or credentials.
    /// The sender-signed privacy label may narrow, never widen, this ceiling.
    pub authorized_clearance: Label,
    /// Validity interval for the signed commit evidence.
    pub receipt_ttl: TimeDelta,
    /// Maximum accepted age of authenticated LakeCat revalidation evidence.
    pub authority_max_age: TimeDelta,
    /// Maximum accepted positive clock skew on LakeCat revalidation evidence.
    pub authority_future_skew: TimeDelta,
}

/// Per-request composition boundary for LakeCat, TypeSec, and Grust cognition.
pub struct GovernedCognitionApplication<S, A>
where
    S: CognitionCommitStore,
    A: LakeCatCognitionAuthority,
{
    vault: MemoryVault<S>,
    lakecat: A,
    basis: BindingBasis,
    engine: CognitionEngineBinding,
    space: MemorySpace,
    verifier: Arc<PrimedAuthorityVerifier>,
    receipts: CognitionReceiptSigner,
    clock: Arc<dyn CognitionClock>,
    receipt_clock: Arc<dyn CognitionClock>,
    authorized_clearance: Label,
    authority_max_age: TimeDelta,
    authority_future_skew: TimeDelta,
    planned_proposal_digest: OnceLock<String>,
    apply_lock: Mutex<()>,
}

impl<S, A> GovernedCognitionApplication<S, A>
where
    S: CognitionCommitStore,
    A: LakeCatCognitionAuthority,
{
    /// Bind one verified TypeDID intent to one LakeCat grant and memory space.
    pub fn new(
        vault: MemoryVault<S>,
        lakecat: A,
        engine: CognitionEngineBinding,
        verified: VerifiedTypeDidContext<'_>,
        config: GovernedCognitionConfig,
        receipt_issuer: ReceiptIssuer,
    ) -> Result<Self, CognitionBindingError> {
        let clock: Arc<dyn CognitionClock> = Arc::new(SystemCognitionClock);
        Self::build(
            vault,
            lakecat,
            engine,
            verified,
            config,
            receipt_issuer,
            Arc::clone(&clock),
            clock,
        )
    }

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    pub fn new_with_clock(
        vault: MemoryVault<S>,
        lakecat: A,
        engine: CognitionEngineBinding,
        verified: VerifiedTypeDidContext<'_>,
        config: GovernedCognitionConfig,
        receipt_issuer: ReceiptIssuer,
        clock: Arc<dyn CognitionClock>,
    ) -> Result<Self, CognitionBindingError> {
        Self::build(
            vault,
            lakecat,
            engine,
            verified,
            config,
            receipt_issuer,
            clock,
            Arc::new(SystemCognitionClock),
        )
    }

    #[cfg(any(test, feature = "test-support"))]
    #[allow(clippy::too_many_arguments)]
    #[doc(hidden)]
    pub fn new_with_clocks(
        vault: MemoryVault<S>,
        lakecat: A,
        engine: CognitionEngineBinding,
        verified: VerifiedTypeDidContext<'_>,
        config: GovernedCognitionConfig,
        receipt_issuer: ReceiptIssuer,
        clock: Arc<dyn CognitionClock>,
        receipt_clock: Arc<dyn CognitionClock>,
    ) -> Result<Self, CognitionBindingError> {
        Self::build(
            vault,
            lakecat,
            engine,
            verified,
            config,
            receipt_issuer,
            clock,
            receipt_clock,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        vault: MemoryVault<S>,
        lakecat: A,
        engine: CognitionEngineBinding,
        verified: VerifiedTypeDidContext<'_>,
        config: GovernedCognitionConfig,
        receipt_issuer: ReceiptIssuer,
        clock: Arc<dyn CognitionClock>,
        receipt_clock: Arc<dyn CognitionClock>,
    ) -> Result<Self, CognitionBindingError> {
        if config.receipt_ttl <= TimeDelta::zero() {
            return Err(CognitionBindingError::InvalidReceiptTtl);
        }
        if config.authority_max_age <= TimeDelta::zero()
            || config.authority_future_skew < TimeDelta::zero()
        {
            return Err(CognitionBindingError::InvalidAuthorityFreshness);
        }
        let basis = BindingBasis::new(
            lakecat.catalog_identity(),
            &config.space,
            verified,
            config.proof,
            &config.source_ids,
            &config.field_mapping,
            clock.now(),
        )?;
        if !engine
            .profile(basis.intent.operation)
            .matches(&basis.intent.algorithm, &basis.intent.algorithm_version)
        {
            return Err(CognitionBindingError::EngineProfileMismatch);
        }
        let authority_verifier = Arc::new(PrimedAuthorityVerifier::new(Arc::clone(&clock)));
        Ok(Self {
            vault: vault.with_cognition_authority(authority_verifier.clone()),
            lakecat,
            basis,
            engine,
            space: config.space,
            verifier: authority_verifier,
            receipts: CognitionReceiptSigner::new(receipt_issuer, config.receipt_ttl),
            clock,
            receipt_clock,
            authorized_clearance: config.authorized_clearance,
            authority_max_age: config.authority_max_age,
            authority_future_skew: config.authority_future_skew,
            planned_proposal_digest: OnceLock::new(),
            apply_lock: Mutex::new(()),
        })
    }

    /// Run the complete governed cognition operation.
    ///
    /// The proposal is transient and never crosses this product boundary.
    /// Planning, post-engine LakeCat revalidation, TypeSec application, and
    /// receipt issuance remain one authenticated operation.
    pub async fn improve(
        &self,
        read: &Capability<CanRead, MemorySpace>,
        write: &Capability<CanWrite, MemorySpace>,
    ) -> Result<GovernedCognitionResult, CognitionApplicationError> {
        let proposal = self.plan(read).await?;
        self.apply(write, &proposal).await
    }

    /// Load one internally consistent authorized input and invoke the engine.
    ///
    /// This is an internal orchestration seam for the native `improve`
    /// operation and its focused tests. It must not become a caller-driven
    /// proposal API.
    async fn plan(
        &self,
        read: &Capability<CanRead, MemorySpace>,
    ) -> Result<CognitionProposal, CognitionApplicationError> {
        self.ensure_request_active()?;
        if read.subject().as_str() != self.basis.intent.subject {
            return Err(CognitionBindingError::ReadSubjectMismatch.into());
        }
        self.revalidate_authority().await?;
        self.ensure_request_active()?;
        let input = self.vault.governed_cognition_input_at(
            &self.space,
            read,
            &self.basis.source_ids,
            &self.request_context(),
            self.basis
                .intent
                .requested_clearance
                .min(self.authorized_clearance),
            self.basis.governed_source_scope(),
        )?;
        if input.governed_source_scope() != Some(self.basis.governed_source_scope())
            || exact_manifest_ids(input.manifest()) != self.basis.source_ids
        {
            return Err(CognitionBindingError::EngineOutputMismatch.into());
        }
        let binding = self.basis.binding(input.manifest());
        self.ensure_request_active()?;
        let proposal = self
            .engine
            .propose(CognitionRequest {
                job_id: &self.basis.intent.job_id,
                source: self.basis.planning_source(),
                binding: &binding,
                input: &input,
                field_mapping: self.basis.field_mapping(),
                operation: self.basis.intent.operation,
            })
            .await?;
        self.ensure_request_active()?;
        let identity =
            validate_planned_proposal(&proposal, &self.basis, &binding, input.manifest())?;
        self.bind_planned_proposal(identity.proposal_digest())?;
        Ok(proposal)
    }

    /// Freshly revalidate LakeCat, atomically apply through TypeSec, and sign
    /// only evidence that exactly matches this application's basis and the
    /// canonical proposal digest previously returned by [`Self::plan`].
    ///
    /// This internal seam exists to test authoritative failure modes. Public
    /// callers use [`Self::improve`] and never receive a proposal.
    async fn apply(
        &self,
        write: &Capability<CanWrite, MemorySpace>,
        proposal: &CognitionProposal,
    ) -> Result<GovernedCognitionResult, CognitionApplicationError> {
        if write.subject().as_str() != self.basis.intent.subject {
            return Err(CognitionBindingError::WriteSubjectMismatch.into());
        }
        self.ensure_request_active()?;
        let proposal_identity = validate_proposal_intent(proposal, &self.basis)?;
        self.verify_planned_proposal(proposal_identity.proposal_digest())?;
        let _guard = self.apply_lock.lock().await;
        self.ensure_request_active()?;
        let policy_decision_id = self.revalidate_authority().await?;
        let evidence = self.basis.authority_evidence(policy_decision_id);
        let receipt_basis =
            ReceiptBasis::new(proposal, &self.basis, &proposal_identity, &evidence)?;
        self.ensure_request_active()?;
        // Consuming this primed decision inside `apply_cognition` is the
        // authorization linearization point: TypeSec rechecks request expiry,
        // context, binding, capability, and policy before its authoritative
        // source reload. The store never treats an engine or backend timestamp
        // as authorization.
        self.verifier
            .prime(evidence, self.basis.intent.effective_expires_at)?;
        let outcome =
            self.vault
                .apply_cognition(&self.space, write, proposal, &self.request_context());
        let clear = self.verifier.clear();
        let outcome = outcome?;
        clear?;
        let (receipt, receipt_token) =
            self.receipts
                .sign(&outcome, &receipt_basis, self.receipt_clock.now())?;
        Ok(GovernedCognitionResult {
            outcome,
            receipt,
            receipt_token,
        })
    }

    /// Test-only planning seam for focused authoritative failure tests.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub async fn plan_for_test(
        &self,
        read: &Capability<CanRead, MemorySpace>,
    ) -> Result<CognitionProposal, CognitionApplicationError> {
        self.plan(read).await
    }

    /// Test-only application seam for focused authoritative failure tests.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub async fn apply_for_test(
        &self,
        write: &Capability<CanWrite, MemorySpace>,
        proposal: &CognitionProposal,
    ) -> Result<GovernedCognitionResult, CognitionApplicationError> {
        self.apply(write, proposal).await
    }

    async fn revalidate_authority(&self) -> Result<String, CognitionApplicationError> {
        self.ensure_request_active()?;
        let fresh = self.lakecat.revalidate(&self.basis.proof).await?;
        let now = self.clock.now();
        self.basis.intent.ensure_active(now)?;
        validate_fresh_authority(
            &self.basis,
            &fresh,
            now,
            self.authority_max_age,
            self.authority_future_skew,
        )
        .map_err(Into::into)
    }

    fn ensure_request_active(&self) -> Result<(), CognitionBindingError> {
        self.basis.intent.ensure_active(self.clock.now())
    }

    fn request_context(&self) -> typesec_core::policy::RequestContext {
        self.basis.intent.policy_context()
    }

    fn bind_planned_proposal(&self, digest: &str) -> Result<(), CognitionBindingError> {
        if self.planned_proposal_digest.set(digest.to_owned()).is_ok() {
            return Ok(());
        }
        self.verify_planned_proposal(digest)
    }

    fn verify_planned_proposal(&self, digest: &str) -> Result<(), CognitionBindingError> {
        match self.planned_proposal_digest.get() {
            Some(expected) if expected == digest => Ok(()),
            Some(_) => Err(CognitionBindingError::PlannedProposalMismatch),
            None => Err(CognitionBindingError::ProposalNotPlanned),
        }
    }
}
