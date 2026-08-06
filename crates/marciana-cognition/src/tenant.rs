//! Canonical deployment-tenant identity validation shared by operational state.

const MAX_TENANT_ID: usize = 256;

/// Whether an identity is bounded and safe for content-free operational state.
pub(crate) fn is_valid_tenant_id(value: &str) -> bool {
    is_valid_component(value, MAX_TENANT_ID)
}

/// Whether a deployment-owned identity component is bounded and canonical.
pub(crate) fn is_valid_component(value: &str, max_length: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_:/.-".contains(&byte))
}
