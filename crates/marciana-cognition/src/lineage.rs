//! Deterministic digest-only lineage inspection over an audit export.

use crate::AuditExportRecord;

const NODE_COUNT: usize = 9;

/// One stable lineage stage retained in the audit projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineageNodeKind {
    Proposal,
    Binding,
    SourceManifest,
    TypedidRequest,
    GovernedScan,
    Snapshot,
    AuthorizationReceipt,
    PolicyDecision,
    Evidence,
}

/// Content-free lineage node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageNode {
    pub kind: LineageNodeKind,
    pub digest: String,
}

/// Directed relationship between two fixed lineage stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineageEdge {
    pub from: LineageNodeKind,
    pub to: LineageNodeKind,
}

/// Stable graph projection for operator and lineage tooling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageInspection {
    pub schema_version: &'static str,
    pub operation_id: String,
    pub space_id: String,
    pub nodes: Vec<LineageNode>,
    pub edges: Vec<LineageEdge>,
    pub affected_id_count: u32,
    pub affected_ids_digest: String,
}

impl LineageInspection {
    /// Build the fixed lineage graph without exposing protected values.
    ///
    /// # Errors
    /// Returns [`LineageError::Invalid`] when an exported identity is empty.
    pub fn from_export(export: &AuditExportRecord) -> Result<Self, LineageError> {
        if export.operation_id.is_empty()
            || export.space_id.is_empty()
            || export.affected_ids_digest.is_empty()
        {
            return Err(LineageError);
        }
        let node_values = [
            (LineageNodeKind::Proposal, &export.proposal_digest),
            (LineageNodeKind::Binding, &export.binding_digest),
            (
                LineageNodeKind::SourceManifest,
                &export.source_manifest_digest,
            ),
            (
                LineageNodeKind::TypedidRequest,
                &export.typedid_request_digest,
            ),
            (LineageNodeKind::GovernedScan, &export.governed_scan_digest),
            (LineageNodeKind::Snapshot, &export.snapshot_digest),
            (
                LineageNodeKind::AuthorizationReceipt,
                &export.authorization_receipt_digest,
            ),
            (
                LineageNodeKind::PolicyDecision,
                &export.policy_decision_digest,
            ),
            (LineageNodeKind::Evidence, &export.evidence_digest),
        ];
        if node_values.iter().any(|(_, digest)| digest.is_empty()) {
            return Err(LineageError);
        }
        let edges = node_values
            .windows(2)
            .map(|window| LineageEdge {
                from: window[0].0,
                to: window[1].0,
            })
            .collect::<Vec<_>>();
        let nodes = node_values
            .iter()
            .map(|(kind, digest)| LineageNode {
                kind: *kind,
                digest: (*digest).clone(),
            })
            .collect::<Vec<_>>();
        debug_assert_eq!(nodes.len(), NODE_COUNT);
        Ok(Self {
            schema_version: "marciana-lineage-v1",
            operation_id: export.operation_id.clone(),
            space_id: export.space_id.clone(),
            nodes,
            edges,
            affected_id_count: export.affected_id_count,
            affected_ids_digest: export.affected_ids_digest.clone(),
        })
    }
}

/// Fixed lineage projection failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("audit lineage projection is invalid")]
pub struct LineageError;
