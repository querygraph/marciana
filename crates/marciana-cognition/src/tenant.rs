//! Canonical deployment-tenant identity validation shared by operational state.

const MAX_TENANT_ID: usize = 256;

/// Whether an identity is bounded and safe for content-free operational state.
pub(crate) fn is_valid_tenant_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_TENANT_ID
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_:/.-".contains(&byte))
}
