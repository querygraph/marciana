//! Shared canonical bounds for untrusted cognition identities and projections.

pub use typesec_memory::{MAX_COGNITION_IDENTITY_BYTES, MAX_COGNITION_PROJECTION_FIELDS};

/// Maximum UTF-8 bytes accepted for one caller-presented bearer token.
pub const MAX_COGNITION_BEARER_TOKEN_BYTES: usize = 256;
/// Maximum UTF-8 bytes accepted for transient worker failure diagnostics.
pub const MAX_COGNITION_FAILURE_BYTES: usize = 16 * 1024;
/// Maximum aggregate UTF-8 bytes in one LakeCat-authorized projection.
pub const MAX_COGNITION_PROJECTION_BYTES: usize = 64 * 1024;

pub(super) fn is_canonical_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_COGNITION_IDENTITY_BYTES
        && value == value.trim()
        && !value.chars().any(char::is_control)
}

pub(super) fn is_canonical_bearer_token(value: &str) -> bool {
    value.len() <= MAX_COGNITION_BEARER_TOKEN_BYTES && is_canonical_text(value)
}

pub(super) fn is_bounded_failure(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= MAX_COGNITION_FAILURE_BYTES
}

/// Reports whether a cognition projection is bounded, canonical, and unique.
#[must_use]
pub fn is_canonical_projection(projection: &[String]) -> bool {
    canonical_projection_bytes(projection).is_some()
}

pub(super) fn canonical_projection_bytes(projection: &[String]) -> Option<usize> {
    if projection.is_empty() || projection.len() > MAX_COGNITION_PROJECTION_FIELDS {
        return None;
    }
    let total_bytes = projection.iter().try_fold(0usize, |total, field| {
        if is_canonical_text(field) {
            total.checked_add(field.len())
        } else {
            None
        }
    })?;
    if total_bytes > MAX_COGNITION_PROJECTION_BYTES || !unique_projection_fields(projection) {
        return None;
    }
    Some(total_bytes)
}

fn unique_projection_fields(projection: &[String]) -> bool {
    if projection.len() <= 8 {
        return projection
            .iter()
            .enumerate()
            .all(|(index, field)| !projection[..index].contains(field));
    }

    let mut fields = projection.iter().map(String::as_str).collect::<Vec<_>>();
    fields.sort_unstable();
    !fields.windows(2).any(|pair| pair[0] == pair[1])
}
