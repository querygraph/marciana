//! Typed fail-closed errors for cognition intent and evidence binding.

/// A governed binding could not be assembled without widening authority.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CognitionBindingError {
    #[error("verified TypeDID action is not memory:improve")]
    ActionMismatch,
    #[error("verified TypeDID resource does not match the target memory space")]
    ResourceMismatch,
    #[error("verified TypeDID privacy is not a canonical TypeSec label")]
    InvalidPrivacy,
    #[error("verified TypeDID request is missing its required purpose claim")]
    MissingPurpose,
    #[error("verified TypeDID cognition identity is not canonical")]
    InvalidIdentity,
    #[error("verified TypeDID cognition claims are not canonical")]
    InvalidClaims,
    #[error("verified TypeDID cognition claim does not match: {0}")]
    IntentClaimMismatch(&'static str),
    #[error("verified TypeDID cognition operation is unsupported")]
    InvalidOperation,
    #[error("verified TypeDID cognition algorithm is unsupported")]
    InvalidAlgorithm,
    #[error("selected cognition engine differs from signed intent")]
    EngineProfileMismatch,
    #[error("verified TypeDID cognition request has expired")]
    RequestExpired,
    #[error("LakeCat proof subject does not match verified TypeDID subject")]
    SubjectMismatch,
    #[error("LakeCat proof purpose does not match verified TypeDID purpose")]
    PurposeMismatch,
    #[error("cognition projection or field mapping is not canonical")]
    InvalidProjection,
    #[error("LakeCat projection differs from the exact algorithm projection")]
    ProjectionMismatch,
    #[error("invalid LakeCat governed scan evidence")]
    InvalidProof,
    #[error("fresh LakeCat catalog identity differs from the bound catalog")]
    CatalogMismatch,
    #[error("fresh LakeCat grant differs from the bound grant")]
    GrantMismatch,
    #[error("fresh LakeCat snapshot differs from the bound snapshot")]
    SnapshotMismatch,
    #[error("fresh LakeCat proof evidence differs from the bound proof")]
    FreshProofMismatch,
    #[error("canonical cognition evidence could not be produced")]
    Digest,
    #[error("cognition receipt TTL must be positive")]
    InvalidReceiptTtl,
    #[error("LakeCat authority freshness limits are invalid")]
    InvalidAuthorityFreshness,
    #[error("cognition source selection is not canonical")]
    InvalidSourceSelection,
    #[error("source-read capability subject does not match verified TypeDID subject")]
    ReadSubjectMismatch,
    #[error("write capability subject does not match verified TypeDID subject")]
    WriteSubjectMismatch,
    #[error("cognition proposal is missing its governed TypeSec binding")]
    MissingProposalBinding,
    #[error("cognition proposal does not preserve the signed intent")]
    ProposalIntentMismatch,
    #[error("cognition proposal was not produced by the bound engine")]
    ProposalNotPlanned,
    #[error("cognition proposal differs from the bound engine plan")]
    PlannedProposalMismatch,
    #[error("cognition engine output does not preserve its governed input")]
    EngineOutputMismatch,
    #[error("fresh LakeCat authority evidence is stale")]
    StaleAuthorityEvidence,
    #[error("fresh LakeCat authority evidence is dated in the future")]
    FutureAuthorityEvidence,
    #[error("fresh LakeCat authority evidence contains an invalid digest")]
    InvalidAuthorityDigest,
    #[error("cognition proposal contains an invalid canonical digest")]
    InvalidProposalDigest,
}
