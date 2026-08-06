//! Stable validation-only request contracts for Marciana's four verbs.

use serde::{Deserialize, Serialize};
use typesec_memory::{
    ForgetSelector, MemoryContent, MemoryDraft, MemoryKind, Provenance, RecallQuery,
};

const MAX_ID: usize = 256;
const MAX_TEXT: usize = 16 * 1024;

/// The four authoritative memory lifecycle verbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryVerb {
    Remember,
    Recall,
    Improve,
    Forget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RememberRequest {
    pub space_id: String,
    pub text: String,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecallRequest {
    pub space_id: String,
    pub query: String,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImproveRequest {
    pub space_id: String,
    pub memory_id: String,
    pub replacement: RememberRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForgetRequest {
    pub space_id: String,
    pub memory_ids: Vec<String>,
    pub purpose: String,
}

/// Fixed validation failures that do not echo caller values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ApiError {
    #[error("memory API identity is invalid")]
    InvalidIdentity,
    #[error("memory API text is invalid")]
    InvalidText,
    #[error("memory API id collection is invalid")]
    InvalidIds,
}

fn identity(value: &str) -> Result<(), ApiError> {
    if value.is_empty()
        || value.len() > MAX_ID
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_:/.-".contains(&b))
    {
        Err(ApiError::InvalidIdentity)
    } else {
        Ok(())
    }
}

fn text(value: &str) -> Result<(), ApiError> {
    if value.is_empty() || value.len() > MAX_TEXT {
        Err(ApiError::InvalidText)
    } else {
        Ok(())
    }
}

impl RememberRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        identity(&self.space_id)?;
        identity(&self.purpose)?;
        text(&self.text)
    }

    /// Lower into the existing TypeSec draft without authorizing it.
    pub fn to_draft(&self) -> Result<MemoryDraft, ApiError> {
        self.validate()?;
        Ok(MemoryDraft::new(
            MemoryKind::Semantic,
            MemoryContent::text(&self.text),
            Provenance::Operator,
        )
        .for_purposes([self.purpose.clone()]))
    }
}
impl RecallRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        identity(&self.space_id)?;
        identity(&self.purpose)?;
        text(&self.query)
    }

    /// Lower into the existing TypeSec query without authorizing it.
    pub fn to_query(&self) -> Result<RecallQuery, ApiError> {
        self.validate()?;
        Ok(RecallQuery::text(&self.query))
    }
}
impl ImproveRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        identity(&self.space_id)?;
        identity(&self.memory_id)?;
        self.replacement.validate()
    }

    /// Lower the replacement into the existing TypeSec draft contract.
    pub fn replacement_draft(&self) -> Result<MemoryDraft, ApiError> {
        self.validate()?;
        self.replacement.to_draft()
    }
}
impl ForgetRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        identity(&self.space_id)?;
        identity(&self.purpose)?;
        if self.memory_ids.is_empty() || self.memory_ids.len() > 256 {
            return Err(ApiError::InvalidIds);
        }
        self.memory_ids.iter().try_for_each(|id| identity(id))
    }

    /// Lower into the existing TypeSec scoped selector without authorizing it.
    pub fn to_selector(&self) -> Result<ForgetSelector, ApiError> {
        self.validate()?;
        Ok(ForgetSelector::Ids(
            self.memory_ids
                .iter()
                .cloned()
                .map(typesec_memory::MemoryId::from_string)
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests;
