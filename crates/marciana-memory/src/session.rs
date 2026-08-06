//! Content-free product metadata for binding a recall to a session.

use sha2::{Digest, Sha256};

use crate::context::{ContextError, RecallIntent};

/// Product metadata that selects a space and recall policy without becoming an
/// authorization namespace. Capabilities are still required at materialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMetadata {
    session_id: String,
    space_id: String,
    recall_policy_digest: String,
    digest: String,
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
        let digest = digest_fields(&session_id, &space_id, &recall_policy_digest);
        Ok(Self {
            session_id,
            space_id,
            recall_policy_digest,
            digest,
        })
    }

    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    #[must_use]
    pub fn space_id(&self) -> &str {
        &self.space_id
    }

    #[must_use]
    pub fn recall_policy_digest(&self) -> &str {
        &self.recall_policy_digest
    }

    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// Bind metadata into the intent identity used by the deterministic plan.
    /// This changes no capability or vault state.
    pub fn bind_intent(&self, mut intent: RecallIntent) -> Result<RecallIntent, SessionError> {
        intent
            .validate()
            .map_err(|_: ContextError| SessionError::InvalidIntent)?;
        let mut hasher = Sha256::new();
        hasher.update(b"querygraph.marciana.session-recall.v1\0");
        hasher.update(intent.query_digest.as_bytes());
        hasher.update([0]);
        hasher.update(self.digest.as_bytes());
        intent.query_digest = format!("sha256:{:x}", hasher.finalize());
        Ok(intent)
    }
}

fn digest_fields(session_id: &str, space_id: &str, policy: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.session.v1\0");
    for value in [session_id, space_id, policy] {
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
