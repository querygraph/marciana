//! Stable validation-only request contracts for Marciana's four verbs.

const MAX_ID: usize = 256;
const MAX_TEXT: usize = 16 * 1024;

/// The four authoritative memory lifecycle verbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryVerb {
    Remember,
    Recall,
    Improve,
    Forget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RememberRequest {
    pub space_id: String,
    pub text: String,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallRequest {
    pub space_id: String,
    pub query: String,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImproveRequest {
    pub space_id: String,
    pub memory_id: String,
    pub replacement: RememberRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
}
impl RecallRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        identity(&self.space_id)?;
        identity(&self.purpose)?;
        text(&self.query)
    }
}
impl ImproveRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        identity(&self.space_id)?;
        identity(&self.memory_id)?;
        self.replacement.validate()
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
}

#[cfg(test)]
mod tests;
