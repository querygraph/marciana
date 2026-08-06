//! Grust guarded persistence for content-free vector-index manifests.

use std::collections::BTreeMap;

use grust_core::prelude::{GraphCommitStore, GraphMutation, Node, NodeId, Value};
use sha2::{Digest, Sha256};

use crate::vector::VectorIndexScope;
use crate::vector_manifest::{VectorIndexManifest, VectorManifestError};
use crate::{GraphStoreMemoryStore, StoreError};

const MANIFEST_LABEL: &str = "VectorIndexManifest";

impl<G: GraphCommitStore> GraphStoreMemoryStore<G> {
    /// Persist one manifest with a guarded, replay-safe graph commit.
    ///
    /// # Errors
    ///
    /// Returns [`VectorManifestError`] when the manifest is invalid or the
    /// guarded backend commit fails.
    pub fn persist_vector_manifest(
        &self,
        manifest: &VectorIndexManifest,
    ) -> Result<(), StoreError> {
        manifest.validate().map_err(|_| manifest_store_error())?;
        let json = serde_json::to_value(manifest).map_err(|_| manifest_store_error())?;
        let mut props = BTreeMap::new();
        props.insert("manifest".to_string(), Value::Json(json));
        props.insert(
            "scope_digest".to_string(),
            Value::String(manifest.scope().digest().to_owned()),
        );
        let node = Node::new(
            MANIFEST_LABEL,
            manifest_node_id(manifest.scope().digest()),
            props,
        );
        let request = persist_request(manifest);
        let commit = grust_core::prelude::GuardedGraphCommit::new(
            format!("marciana-vector-manifest:{}", &manifest.digest()[7..]),
            request,
            vec![GraphMutation::UpsertNode(node)],
        );
        self.run(self.graph().commit_guarded(&commit))
            .map_err(|_| manifest_store_error())?;
        Ok(())
    }

    /// Recover a manifest only when its persisted scope and digest match.
    ///
    /// # Errors
    ///
    /// Returns [`VectorManifestError`] when the persisted manifest is absent,
    /// undecodable, or fails scope revalidation.
    pub fn load_vector_manifest(
        &self,
        scope: &VectorIndexScope,
    ) -> Result<Option<VectorIndexManifest>, StoreError> {
        let Some(node) = self
            .run(self.graph().get_node(&manifest_node_id(scope.digest())))
            .map_err(|_| manifest_store_error())?
        else {
            return Ok(None);
        };
        let Some(Value::Json(json)) = node.props.get("manifest") else {
            return Err(manifest_store_error());
        };
        let manifest: VectorIndexManifest =
            serde_json::from_value(json.clone()).map_err(|_| manifest_store_error())?;
        if manifest.scope() != scope {
            return Err(manifest_store_error());
        }
        manifest
            .validate()
            .map_err(|_: VectorManifestError| manifest_store_error())?;
        Ok(Some(manifest))
    }
}

fn manifest_node_id(scope_digest: &str) -> NodeId {
    NodeId::from(format!("vector-manifest:{scope_digest}").as_str())
}

fn persist_request(manifest: &VectorIndexManifest) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.vector-manifest.persist.v1\0");
    hasher.update(manifest.scope().digest().as_bytes());
    hasher.update(manifest.digest().as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn manifest_store_error() -> StoreError {
    StoreError::Backend("vector manifest persistence failed".into())
}
