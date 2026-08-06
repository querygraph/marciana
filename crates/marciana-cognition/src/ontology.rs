//! Immutable, bounded schema/ontology declarations for governed cognition.

use sha2::{Digest, Sha256};

const MAX_COMPONENT: usize = 128;
const MAX_FIELDS: usize = 128;
const MAX_EDGES: usize = 128;
const MAX_SCHEMAS: usize = 64;

/// Canonical identity for one operator-owned schema version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaIdentity {
    pub namespace: String,
    pub name: String,
    pub version: u32,
}

/// Closed field types available to schema declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchemaFieldKind {
    Identifier,
    Text,
    Integer,
    Decimal,
    Timestamp,
    Boolean,
}

/// One typed entity field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaField {
    pub name: String,
    pub kind: SchemaFieldKind,
}

/// One typed relationship between declared entity kinds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaEdge {
    pub name: String,
    pub from_kind: String,
    pub to_kind: String,
}

/// Validated immutable schema definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaDefinition {
    identity: SchemaIdentity,
    fields: Vec<SchemaField>,
    edges: Vec<SchemaEdge>,
    digest: String,
}

impl SchemaIdentity {
    /// Construct a canonical schema identity.
    ///
    /// # Errors
    /// Returns a fixed error for empty, oversized, or non-canonical components.
    pub fn new(namespace: String, name: String, version: u32) -> Result<Self, OntologyError> {
        if !valid_component(&namespace) || !valid_component(&name) || version == 0 {
            return Err(OntologyError::InvalidIdentity);
        }
        Ok(Self {
            namespace,
            name,
            version,
        })
    }
}

impl SchemaDefinition {
    /// Validate and canonicalize one operator-owned schema definition.
    ///
    /// # Errors
    /// Returns a fixed error when declarations are malformed, duplicated, or
    /// exceed their fixed bounds.
    pub fn new(
        identity: SchemaIdentity,
        mut fields: Vec<SchemaField>,
        mut edges: Vec<SchemaEdge>,
    ) -> Result<Self, OntologyError> {
        if fields.is_empty() || fields.len() > MAX_FIELDS || edges.len() > MAX_EDGES {
            return Err(OntologyError::Bounds);
        }
        if fields.iter().any(|field| !valid_component(&field.name)) {
            return Err(OntologyError::InvalidField);
        }
        if edges.iter().any(|edge| {
            !valid_component(&edge.name)
                || !valid_component(&edge.from_kind)
                || !valid_component(&edge.to_kind)
        }) {
            return Err(OntologyError::InvalidEdge);
        }
        fields.sort_unstable();
        edges.sort_unstable();
        if fields.windows(2).any(|pair| pair[0].name == pair[1].name) {
            return Err(OntologyError::DuplicateField);
        }
        if edges.windows(2).any(|pair| pair[0].name == pair[1].name) {
            return Err(OntologyError::DuplicateEdge);
        }
        let digest = definition_digest(&identity, &fields, &edges);
        Ok(Self {
            identity,
            fields,
            edges,
            digest,
        })
    }

    /// Schema identity.
    #[must_use]
    pub fn identity(&self) -> &SchemaIdentity {
        &self.identity
    }

    /// Canonically ordered fields.
    #[must_use]
    pub fn fields(&self) -> &[SchemaField] {
        &self.fields
    }

    /// Canonically ordered edges.
    #[must_use]
    pub fn edges(&self) -> &[SchemaEdge] {
        &self.edges
    }

    /// Stable digest of identity and declarations.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
}

/// Immutable registry of operator-owned schema versions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaRegistry {
    schemas: Vec<SchemaDefinition>,
    digest: String,
}

impl SchemaRegistry {
    /// Build a canonical registry; model output has no registration path.
    ///
    /// # Errors
    /// Returns a fixed error for an empty, oversized, or duplicate registry.
    pub fn new(mut schemas: Vec<SchemaDefinition>) -> Result<Self, OntologyError> {
        if schemas.is_empty() || schemas.len() > MAX_SCHEMAS {
            return Err(OntologyError::RegistryBounds);
        }
        schemas.sort_unstable_by(|left, right| left.identity.cmp(&right.identity));
        if schemas
            .windows(2)
            .any(|pair| pair[0].identity == pair[1].identity)
        {
            return Err(OntologyError::DuplicateSchema);
        }
        let mut hasher = Sha256::new();
        hasher.update(b"querygraph.marciana.ontology-registry.v1\0");
        for schema in &schemas {
            hasher.update(schema.digest.as_bytes());
            hasher.update([0]);
        }
        Ok(Self {
            schemas,
            digest: format!("sha256:{:x}", hasher.finalize()),
        })
    }

    /// Canonically ordered schema definitions.
    #[must_use]
    pub fn schemas(&self) -> &[SchemaDefinition] {
        &self.schemas
    }

    /// Stable registry digest.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// Resolve an exact immutable schema identity.
    #[must_use]
    pub fn resolve(&self, identity: &SchemaIdentity) -> Option<&SchemaDefinition> {
        self.schemas
            .binary_search_by(|schema| schema.identity.cmp(identity))
            .ok()
            .map(|index| &self.schemas[index])
    }
}

/// Fixed schema/ontology validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum OntologyError {
    #[error("schema identity is invalid")]
    InvalidIdentity,
    #[error("schema field is invalid")]
    InvalidField,
    #[error("schema edge is invalid")]
    InvalidEdge,
    #[error("schema declaration bounds are invalid")]
    Bounds,
    #[error("schema field is duplicated")]
    DuplicateField,
    #[error("schema edge is duplicated")]
    DuplicateEdge,
    #[error("schema registry bounds are invalid")]
    RegistryBounds,
    #[error("schema identity is duplicated")]
    DuplicateSchema,
}

fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_COMPONENT
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-.".contains(&byte))
}

fn definition_digest(
    identity: &SchemaIdentity,
    fields: &[SchemaField],
    edges: &[SchemaEdge],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"querygraph.marciana.ontology-schema.v1\0");
    update_text(&mut hasher, &identity.namespace);
    update_text(&mut hasher, &identity.name);
    hasher.update(identity.version.to_be_bytes());
    for field in fields {
        update_text(&mut hasher, &field.name);
        hasher.update([field_kind_code(field.kind)]);
    }
    for edge in edges {
        update_text(&mut hasher, &edge.name);
        update_text(&mut hasher, &edge.from_kind);
        update_text(&mut hasher, &edge.to_kind);
    }
    format!("sha256:{:x}", hasher.finalize())
}

const fn field_kind_code(kind: SchemaFieldKind) -> u8 {
    match kind {
        SchemaFieldKind::Identifier => 1,
        SchemaFieldKind::Text => 2,
        SchemaFieldKind::Integer => 3,
        SchemaFieldKind::Decimal => 4,
        SchemaFieldKind::Timestamp => 5,
        SchemaFieldKind::Boolean => 6,
    }
}

fn update_text(hasher: &mut Sha256, value: &str) {
    hasher.update(value.as_bytes());
    hasher.update([0]);
}
