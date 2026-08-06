//! Bounded schema-family version windows for compatibility checks.

const MAX_FAMILY: usize = 128;

/// Supported inclusive versions for one deployment-owned schema family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaWindow {
    family: String,
    minimum: u32,
    maximum: u32,
}

impl SchemaWindow {
    /// Create an inclusive version window such as `querygraph-memory-v1..v2`.
    ///
    /// # Errors
    /// Returns a fixed error for malformed families, zero versions, or a
    /// reversed range.
    pub fn new(family: String, minimum: u32, maximum: u32) -> Result<Self, SchemaWindowError> {
        if !is_valid_family(&family) {
            return Err(SchemaWindowError::InvalidFamily);
        }
        if minimum == 0 || maximum == 0 || minimum > maximum {
            return Err(SchemaWindowError::InvalidRange);
        }
        Ok(Self {
            family,
            minimum,
            maximum,
        })
    }

    /// Schema family covered by this window.
    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }

    /// Inclusive minimum supported version.
    #[must_use]
    pub const fn minimum(&self) -> u32 {
        self.minimum
    }

    /// Inclusive maximum supported version.
    #[must_use]
    pub const fn maximum(&self) -> u32 {
        self.maximum
    }

    /// Whether a canonical `family-vN` schema identity is supported.
    #[must_use]
    pub fn accepts(&self, schema: &str) -> bool {
        let Some(version_text) = schema
            .strip_prefix(&self.family)
            .and_then(|value| value.strip_prefix("-v"))
        else {
            return false;
        };
        let Ok(version) = version_text.parse::<u32>() else {
            return false;
        };
        version >= self.minimum && version <= self.maximum
    }
}

/// Fixed schema-window construction failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SchemaWindowError {
    #[error("schema family is invalid")]
    InvalidFamily,
    #[error("schema version range is invalid")]
    InvalidRange,
}

fn is_valid_family(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_FAMILY
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-.:/".contains(&byte))
}
