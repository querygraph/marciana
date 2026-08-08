//! Thin Apache Ossie semantic-model integration.
//!
//! Ossie supplies portable semantic definitions. Marciana validates and lowers
//! those definitions into its operator-owned ontology registry; it never lets
//! an Ossie document mint a capability or write memory directly.

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::ontology::{
    OntologyError, SchemaDefinition, SchemaEdge, SchemaField, SchemaFieldKind, SchemaIdentity,
};

const MAX_ITEMS: usize = 128;
const MAX_TEXT: usize = 256;

/// A bounded Ossie semantic model document accepted by the Marciana adapter.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OssieDocument {
    pub namespace: String,
    pub name: String,
    pub version: u32,
    #[serde(default)]
    pub metrics: Vec<OssieMetric>,
    #[serde(default)]
    pub dimensions: Vec<OssieDimension>,
    #[serde(default)]
    pub relationships: Vec<OssieRelationship>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OssieMetric {
    pub name: String,
    #[serde(default)]
    pub expression: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OssieDimension {
    pub name: String,
    #[serde(default)]
    pub role: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OssieRelationship {
    pub name: String,
    pub from: String,
    pub to: String,
}

/// Validated semantic binding between Ossie and Marciana's ontology layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OssieBinding {
    source_manifest: String,
    document: OssieDocumentSummary,
    schema: SchemaDefinition,
    digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OssieDocumentSummary {
    namespace: String,
    name: String,
    version: u32,
    // Metric (name, expression) pairs: the expression is accepted semantic
    // content, so it must be bound into the digest alongside the name.
    metrics: Vec<(String, String)>,
    dimensions: Vec<String>,
    relationships: Vec<String>,
}

/// Deterministic semantic query plan lowered from an Ossie binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OssieQueryPlan {
    pub binding_digest: String,
    pub metric: String,
    pub dimensions: Vec<String>,
    pub plan_digest: String,
}

/// Import and plan operations for the Apache Ossie boundary.
pub struct OssieAdapter;

impl OssieAdapter {
    /// Parse an Ossie JSON document and lower it into a Marciana schema.
    ///
    /// # Errors
    ///
    /// Returns an error when the source manifest, JSON document, semantic
    /// names, bounds, or lowered ontology are invalid.
    pub fn import_json(
        source_manifest: impl Into<String>,
        json: &str,
    ) -> Result<OssieBinding, OssieError> {
        let source_manifest = source_manifest.into();
        validate_text(&source_manifest).map_err(|()| OssieError::InvalidSourceManifest)?;
        let document: OssieDocument =
            serde_json::from_str(json).map_err(|_| OssieError::InvalidJson)?;
        validate_document(&document)?;

        let OssieDocument {
            namespace,
            name,
            version,
            metrics,
            dimensions,
            relationships,
        } = document;
        let identity = SchemaIdentity::new(namespace.clone(), name.clone(), version)?;
        let mut fields = Vec::with_capacity(metrics.len() + dimensions.len());
        let mut canonical_metrics = Vec::with_capacity(metrics.len());
        for metric in metrics {
            fields.push(SchemaField {
                name: metric.name.clone(),
                kind: SchemaFieldKind::Decimal,
            });
            canonical_metrics.push((metric.name, metric.expression));
        }
        let mut canonical_dimensions = Vec::with_capacity(dimensions.len());
        for dimension in dimensions {
            fields.push(SchemaField {
                name: dimension.name.clone(),
                kind: if dimension.role == "identifier" {
                    SchemaFieldKind::Identifier
                } else {
                    SchemaFieldKind::Text
                },
            });
            canonical_dimensions.push(dimension.name);
        }
        let mut edges = Vec::with_capacity(relationships.len());
        let mut canonical_relationships = Vec::with_capacity(relationships.len());
        for relationship in relationships {
            edges.push(SchemaEdge {
                name: relationship.name.clone(),
                from_kind: relationship.from,
                to_kind: relationship.to,
            });
            canonical_relationships.push(relationship.name);
        }
        let schema =
            SchemaDefinition::new(identity, fields, edges).map_err(|error| match error {
                OntologyError::DuplicateField => OssieError::InvalidDocument,
                error => OssieError::Ontology(error),
            })?;
        canonical_metrics.sort_unstable();
        canonical_dimensions.sort_unstable();
        canonical_relationships.sort_unstable();
        let summary = OssieDocumentSummary {
            namespace,
            name,
            version,
            metrics: canonical_metrics,
            dimensions: canonical_dimensions,
            relationships: canonical_relationships,
        };
        let digest = binding_digest(&source_manifest, &summary, schema.digest());
        Ok(OssieBinding {
            source_manifest,
            document: summary,
            schema,
            digest,
        })
    }

    /// Lower a metric/dimension request into a deterministic semantic plan.
    /// The plan is content-free and must still enter the normal `RecallIntent`
    /// and `TypeSec` authorization path before materialization.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested metric or dimensions are not in the
    /// validated binding, or when the request exceeds the bounded plan shape.
    pub fn plan_query(
        binding: &OssieBinding,
        metric: impl Into<String>,
        dimensions: impl IntoIterator<Item = String>,
    ) -> Result<OssieQueryPlan, OssieError> {
        let metric = metric.into();
        if binding
            .document
            .metrics
            .binary_search_by(|(name, _)| name.cmp(&metric))
            .is_err()
        {
            return Err(OssieError::UnknownMetric);
        }
        let mut dimensions = dimensions.into_iter().collect::<Vec<_>>();
        if dimensions.is_empty() || dimensions.len() > MAX_ITEMS {
            return Err(OssieError::InvalidQuery);
        }
        dimensions.sort_unstable();
        dimensions.dedup();
        if !is_sorted_subset(&dimensions, &binding.document.dimensions) {
            return Err(OssieError::UnknownDimension);
        }
        let mut hasher = Sha256::new();
        hasher.update(b"querygraph.marciana.ossie-query.v1\0");
        update_text(&mut hasher, &binding.digest);
        update_text(&mut hasher, &metric);
        for dimension in &dimensions {
            update_text(&mut hasher, dimension);
        }
        Ok(OssieQueryPlan {
            binding_digest: binding.digest.clone(),
            metric,
            dimensions,
            plan_digest: format!("sha256:{:x}", hasher.finalize()),
        })
    }
}

impl OssieBinding {
    #[must_use]
    pub fn source_manifest(&self) -> &str {
        &self.source_manifest
    }

    #[must_use]
    pub fn schema(&self) -> &SchemaDefinition {
        &self.schema
    }

    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum OssieError {
    #[error("Ossie JSON is invalid")]
    InvalidJson,
    #[error("Ossie source manifest is invalid")]
    InvalidSourceManifest,
    #[error("Ossie document is invalid")]
    InvalidDocument,
    #[error("Ossie document exceeds bounded limits")]
    Bounds,
    #[error("Ossie metric is invalid")]
    InvalidMetric,
    #[error("Ossie dimension is invalid")]
    InvalidDimension,
    #[error("Ossie relationship is invalid")]
    InvalidRelationship,
    #[error("Ossie metric is unknown")]
    UnknownMetric,
    #[error("Ossie dimension is unknown")]
    UnknownDimension,
    #[error("Ossie query is invalid")]
    InvalidQuery,
    #[error("lowered Ossie ontology is invalid")]
    Ontology(#[from] OntologyError),
}

fn validate_document(document: &OssieDocument) -> Result<(), OssieError> {
    if document.version == 0
        || document.metrics.len() + document.dimensions.len() > MAX_ITEMS
        || document.relationships.len() > MAX_ITEMS
        || (document.metrics.is_empty() && document.dimensions.is_empty())
    {
        return Err(OssieError::Bounds);
    }
    if !valid_component(&document.namespace) || !valid_component(&document.name) {
        return Err(OssieError::InvalidDocument);
    }
    if document
        .metrics
        .iter()
        .any(|metric| !valid_component(&metric.name) || metric.expression.len() > MAX_TEXT)
    {
        return Err(OssieError::InvalidMetric);
    }
    if document
        .dimensions
        .iter()
        .any(|dimension| !valid_component(&dimension.name) || dimension.role.len() > MAX_TEXT)
    {
        return Err(OssieError::InvalidDimension);
    }
    if document.relationships.iter().any(|relationship| {
        !valid_component(&relationship.name)
            || !valid_component(&relationship.from)
            || !valid_component(&relationship.to)
    }) {
        return Err(OssieError::InvalidRelationship);
    }
    Ok(())
}

fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_TEXT
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-.:/".contains(&byte))
}

fn validate_text(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > MAX_TEXT
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        Err(())
    } else {
        Ok(())
    }
}

fn is_sorted_subset(required: &[String], available: &[String]) -> bool {
    let mut available = available.iter();
    required.iter().all(|required| {
        loop {
            match available.next() {
                Some(candidate) if candidate < required => {}
                Some(candidate) => break candidate == required,
                None => break false,
            }
        }
    })
}

fn binding_digest(source_manifest: &str, document: &OssieDocumentSummary, schema: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.ossie-binding.v2\0");
    update_text(&mut hasher, source_manifest);
    update_text(&mut hasher, &document.namespace);
    update_text(&mut hasher, &document.name);
    hasher.update(document.version.to_be_bytes());
    update_text(&mut hasher, schema);
    update_section(&mut hasher, "metrics", document.metrics.len());
    for (name, expression) in &document.metrics {
        update_text(&mut hasher, name);
        update_text(&mut hasher, expression);
    }
    update_section(&mut hasher, "dimensions", document.dimensions.len());
    for name in &document.dimensions {
        update_text(&mut hasher, name);
    }
    update_section(&mut hasher, "relationships", document.relationships.len());
    for name in &document.relationships {
        update_text(&mut hasher, name);
    }
    format!("sha256:{:x}", hasher.finalize())
}

// Labeled, counted section boundaries keep names in one section from
// colliding with names in the next under the flat byte stream.
fn update_section(hasher: &mut Sha256, label: &str, count: usize) {
    hasher.update(label.as_bytes());
    hasher.update([0]);
    hasher.update(
        u64::try_from(count)
            .expect("bounded section count")
            .to_be_bytes(),
    );
}

fn update_text(hasher: &mut Sha256, value: &str) {
    hasher.update(value.as_bytes());
    hasher.update([0]);
}
