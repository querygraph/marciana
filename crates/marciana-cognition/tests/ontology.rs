use marciana_cognition::{
    OntologyError, SchemaDefinition, SchemaEdge, SchemaField, SchemaFieldKind, SchemaIdentity,
    SchemaRegistry,
};

fn identity(version: u32) -> SchemaIdentity {
    SchemaIdentity::new("agstack".into(), "coffee-market".into(), version).expect("identity")
}

fn definition(version: u32, reverse: bool) -> SchemaDefinition {
    let mut fields = vec![
        SchemaField {
            name: "price".into(),
            kind: SchemaFieldKind::Decimal,
        },
        SchemaField {
            name: "farm_id".into(),
            kind: SchemaFieldKind::Identifier,
        },
    ];
    let mut edges = vec![SchemaEdge {
        name: "traded_at".into(),
        from_kind: "coffee_lot".into(),
        to_kind: "market".into(),
    }];
    if reverse {
        fields.reverse();
        edges.reverse();
    }
    SchemaDefinition::new(identity(version), fields, edges).expect("definition")
}

#[test]
fn definitions_and_registries_are_order_independent_and_resolvable() {
    let first = definition(1, false);
    let reversed = definition(1, true);
    assert_eq!(first.digest(), reversed.digest());
    assert_eq!(first.fields()[0].name, "farm_id");
    let registry =
        SchemaRegistry::new(vec![definition(2, false), first.clone()]).expect("registry");
    let same_order = SchemaRegistry::new(vec![first, definition(2, false)]).expect("registry");
    assert_eq!(registry.digest(), same_order.digest());
    assert_eq!(registry.schemas()[0].identity().version, 1);
    assert!(registry.resolve(&identity(2)).is_some());
}

#[test]
fn ontology_rejects_duplicate_declarations_and_schema_identities() {
    let duplicate_field = SchemaDefinition::new(
        identity(1),
        vec![
            SchemaField {
                name: "price".into(),
                kind: SchemaFieldKind::Decimal,
            },
            SchemaField {
                name: "price".into(),
                kind: SchemaFieldKind::Text,
            },
        ],
        vec![],
    )
    .expect_err("duplicate field");
    assert_eq!(duplicate_field, OntologyError::DuplicateField);
    assert_eq!(
        SchemaRegistry::new(vec![definition(1, false), definition(1, false)])
            .expect_err("duplicate schema"),
        OntologyError::DuplicateSchema
    );
}

#[test]
fn ontology_rejects_unbounded_or_noncanonical_components() {
    assert_eq!(
        SchemaIdentity::new("ag stack".into(), "coffee".into(), 1).expect_err("identity"),
        OntologyError::InvalidIdentity
    );
    assert_eq!(
        SchemaDefinition::new(
            identity(1),
            vec![SchemaField {
                name: "bad field".into(),
                kind: SchemaFieldKind::Text,
            }],
            vec![],
        )
        .expect_err("field"),
        OntologyError::InvalidField
    );
}
