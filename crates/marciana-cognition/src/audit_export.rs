//! Versioned, content-free operator export of cognition audit evidence.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use typesec_memory::CognitionAuditEvidence;

const EXPORT_SCHEMA_VERSION: u32 = 1;
const MAX_AFFECTED_IDS: usize = 4_096;

/// Stable redacted audit projection for dashboards, export, and lineage tools.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditExportRecord {
    pub schema_version: u32,
    pub operation_id: String,
    pub space_id: String,
    pub effect: typesec_memory::CognitionEffect,
    pub subject_digest: String,
    pub purpose_digest: String,
    pub governed_source_scope_digest: Option<String>,
    pub proposal_digest: String,
    pub binding_digest: String,
    pub source_manifest_digest: String,
    pub typedid_request_digest: String,
    pub governed_scan_digest: String,
    pub snapshot_digest: String,
    pub authorization_receipt_digest: String,
    pub policy_decision_digest: String,
    pub evidence_digest: String,
    pub affected_id_count: u32,
    pub affected_ids_digest: String,
    pub authority_revalidated_at: chrono::DateTime<chrono::Utc>,
    pub prepared_at: chrono::DateTime<chrono::Utc>,
}

impl AuditExportRecord {
    /// Project one validated durable audit record without exposing its values.
    ///
    /// # Errors
    /// Returns a fixed error when the audit schema or affected-ID bound is
    /// unsupported.
    pub fn from_audit(audit: &CognitionAuditEvidence) -> Result<Self, AuditExportError> {
        if audit.schema_version != CognitionAuditEvidence::SCHEMA_VERSION {
            return Err(AuditExportError::UnsupportedSchema);
        }
        if audit.affected_ids.len() > MAX_AFFECTED_IDS {
            return Err(AuditExportError::Bounds);
        }
        Ok(Self {
            schema_version: EXPORT_SCHEMA_VERSION,
            operation_id: audit.operation_id.clone(),
            space_id: audit.space_id.clone(),
            effect: audit.effect,
            subject_digest: digest("subject", &audit.subject),
            purpose_digest: digest("purpose", &audit.purpose),
            governed_source_scope_digest: audit
                .governed_source_scope
                .as_ref()
                .map(ToString::to_string),
            proposal_digest: audit.proposal_digest.clone(),
            binding_digest: audit.binding_digest.clone(),
            source_manifest_digest: audit.source_manifest_digest.clone(),
            typedid_request_digest: audit.typedid_request_digest.clone(),
            governed_scan_digest: audit.governed_scan_digest.clone(),
            snapshot_digest: audit.snapshot_digest.clone(),
            authorization_receipt_digest: audit.authorization_receipt_digest.clone(),
            policy_decision_digest: digest("policy-decision", &audit.policy_decision_id),
            evidence_digest: audit.evidence_digest.clone(),
            affected_id_count: u32::try_from(audit.affected_ids.len())
                .map_err(|_| AuditExportError::Bounds)?,
            affected_ids_digest: affected_ids_digest(audit),
            authority_revalidated_at: audit.authority_revalidated_at,
            prepared_at: audit.prepared_at,
        })
    }
}

/// Fixed audit export failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AuditExportError {
    #[error("unsupported cognition audit schema")]
    UnsupportedSchema,
    #[error("cognition audit export exceeds its fixed bound")]
    Bounds,
}

fn digest(domain: &str, value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.audit-export.v1\0");
    hasher.update(domain.as_bytes());
    hasher.update([0]);
    hasher.update(value.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn affected_ids_digest(audit: &CognitionAuditEvidence) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.audit-export.affected.v1\0");
    let mut previous_id = None;
    for id in &audit.affected_ids {
        let id = id.as_str();
        if previous_id.is_some_and(|previous| previous >= id) {
            return sorted_affected_ids_digest(audit);
        }
        hasher.update(id.as_bytes());
        hasher.update([0]);
        previous_id = Some(id);
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn sorted_affected_ids_digest(audit: &CognitionAuditEvidence) -> String {
    let mut ids = audit
        .affected_ids
        .iter()
        .map(typesec_memory::MemoryId::as_str)
        .collect::<Vec<_>>();
    ids.sort_unstable();
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.audit-export.affected.v1\0");
    for id in ids {
        hasher.update(id.as_bytes());
        hasher.update([0]);
    }
    format!("sha256:{:x}", hasher.finalize())
}
