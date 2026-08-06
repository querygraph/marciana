//! Content-free product metadata for binding a recall to a session or thread.

use sha2::{Digest, Sha256};

use crate::context::{ContextError, RecallIntent};

/// Product metadata that selects a space and recall policy without becoming an
/// authorization namespace. Capabilities are still required at materialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMetadata {
    core: ContextMetadataCore,
}

/// Product metadata for a conversation thread. It is not an authorization
/// namespace; capabilities are still required at materialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadMetadata {
    core: ContextMetadataCore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContextMetadataCore {
    owner_id: String,
    space_id: String,
    recall_policy_digest: String,
    digest: String,
    recall_domain: &'static [u8],
}

/// Shared binding contract used by session- and thread-scoped facade paths.
pub trait RecallContextMetadata {
    fn space_id(&self) -> &str;
    fn bind_intent(&self, intent: RecallIntent) -> Result<RecallIntent, SessionError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SessionError {
    #[error("session metadata is invalid")]
    Invalid,
    #[error("recall intent is invalid")]
    InvalidIntent,
}

impl SessionMetadata {
    /// Construct bounded, content-free session metadata.
    pub fn new(
        session_id: String,
        space_id: String,
        recall_policy_digest: String,
    ) -> Result<Self, SessionError> {
        if !valid_identifier(&session_id)
            || !valid_identifier(&space_id)
            || !valid_digest(&recall_policy_digest)
        {
            return Err(SessionError::Invalid);
        }
        Ok(Self {
            core: ContextMetadataCore::new(
                session_id,
                space_id,
                recall_policy_digest,
                b"querygraph.marciana.session.v1\0",
                b"querygraph.marciana.session-recall.v1\0",
            )?,
        })
    }

    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.core.owner_id
    }

    #[must_use]
    pub fn space_id(&self) -> &str {
        &self.core.space_id
    }

    #[must_use]
    pub fn recall_policy_digest(&self) -> &str {
        &self.core.recall_policy_digest
    }

    #[must_use]
    pub fn digest(&self) -> &str {
        &self.core.digest
    }

    /// Bind metadata into the intent identity used by the deterministic plan.
    /// This changes no capability or vault state.
    pub fn bind_intent(&self, mut intent: RecallIntent) -> Result<RecallIntent, SessionError> {
        self.core.bind_intent(&mut intent)
    }
}

impl ThreadMetadata {
    pub fn new(
        thread_id: String,
        space_id: String,
        recall_policy_digest: String,
    ) -> Result<Self, SessionError> {
        Ok(Self {
            core: ContextMetadataCore::new(
                thread_id,
                space_id,
                recall_policy_digest,
                b"querygraph.marciana.thread.v1\0",
                b"querygraph.marciana.thread-recall.v1\0",
            )?,
        })
    }

    #[must_use]
    pub fn thread_id(&self) -> &str {
        &self.core.owner_id
    }

    #[must_use]
    pub fn space_id(&self) -> &str {
        &self.core.space_id
    }

    #[must_use]
    pub fn recall_policy_digest(&self) -> &str {
        &self.core.recall_policy_digest
    }

    #[must_use]
    pub fn digest(&self) -> &str {
        &self.core.digest
    }

    pub fn bind_intent(&self, mut intent: RecallIntent) -> Result<RecallIntent, SessionError> {
        self.core.bind_intent(&mut intent)
    }
}

impl RecallContextMetadata for SessionMetadata {
    fn space_id(&self) -> &str {
        self.space_id()
    }

    fn bind_intent(&self, intent: RecallIntent) -> Result<RecallIntent, SessionError> {
        self.bind_intent(intent)
    }
}

impl RecallContextMetadata for ThreadMetadata {
    fn space_id(&self) -> &str {
        self.space_id()
    }

    fn bind_intent(&self, intent: RecallIntent) -> Result<RecallIntent, SessionError> {
        self.bind_intent(intent)
    }
}

impl ContextMetadataCore {
    fn new(
        owner_id: String,
        space_id: String,
        recall_policy_digest: String,
        digest_domain: &'static [u8],
        recall_domain: &'static [u8],
    ) -> Result<Self, SessionError> {
        if !valid_identifier(&owner_id)
            || !valid_identifier(&space_id)
            || !valid_digest(&recall_policy_digest)
        {
            return Err(SessionError::Invalid);
        }
        let digest = digest_fields(digest_domain, &owner_id, &space_id, &recall_policy_digest);
        Ok(Self {
            owner_id,
            space_id,
            recall_policy_digest,
            digest,
            recall_domain,
        })
    }

    fn bind_intent(&self, intent: &mut RecallIntent) -> Result<RecallIntent, SessionError> {
        intent
            .validate()
            .map_err(|_: ContextError| SessionError::InvalidIntent)?;
        let mut hasher = Sha256::new();
        hasher.update(self.recall_domain);
        hasher.update(intent.query_digest.as_bytes());
        hasher.update([0]);
        hasher.update(self.digest.as_bytes());
        intent.query_digest = format!("sha256:{:x}", hasher.finalize());
        Ok(intent.clone())
    }
}

fn digest_fields(domain: &[u8], owner_id: &str, space_id: &str, policy: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for value in [owner_id, space_id, policy] {
        hasher.update(value.as_bytes());
        hasher.update([0]);
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_:/.-".contains(&byte))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}
