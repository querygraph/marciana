use marciana_cognition::{OssieAdapter, OssieError};

const MODEL: &str = r#"
{
  "namespace": "agstack",
  "name": "coffee-market",
  "version": 1,
  "metrics": [{"name": "price_usd_per_kg", "expression": "avg(price_usd_per_kg)"}],
  "dimensions": [
    {"name": "country", "role": "attribute"},
    {"name": "market", "role": "identifier"}
  ],
  "relationships": [{"name": "market_in_country", "from": "market", "to": "country"}]
}
"#;

#[test]
fn imports_ossie_into_operator_owned_ontology() {
    let binding = OssieAdapter::import_json("lakecat:agstack/coffee/v1", MODEL).expect("binding");
    assert_eq!(binding.source_manifest(), "lakecat:agstack/coffee/v1");
    assert_eq!(binding.schema().fields().len(), 3);
    assert_eq!(binding.schema().edges().len(), 1);
    assert!(binding.digest().starts_with("sha256:"));
}

#[test]
fn query_plans_are_order_independent_and_bound_to_the_import() {
    let binding = OssieAdapter::import_json("lakecat:agstack/coffee/v1", MODEL).expect("binding");
    let first = OssieAdapter::plan_query(
        &binding,
        "price_usd_per_kg",
        vec!["market".into(), "country".into()],
    )
    .expect("plan");
    let reversed = OssieAdapter::plan_query(
        &binding,
        "price_usd_per_kg",
        vec!["country".into(), "market".into()],
    )
    .expect("plan");
    assert_eq!(first, reversed);
    assert_eq!(first.binding_digest, binding.digest());
}

#[test]
fn rejects_unknown_semantics_and_duplicate_names() {
    let binding = OssieAdapter::import_json("lakecat:agstack/coffee/v1", MODEL).expect("binding");
    assert_eq!(
        OssieAdapter::plan_query(&binding, "unknown", vec!["market".into()])
            .expect_err("unknown metric"),
        OssieError::UnknownMetric
    );
    let duplicate = MODEL.replace(
        "{\"name\": \"market\", \"role\": \"identifier\"}",
        "{\"name\": \"country\", \"role\": \"identifier\"}",
    );
    assert_eq!(
        OssieAdapter::import_json("lakecat:agstack/coffee/v1", &duplicate)
            .expect_err("duplicate dimension"),
        OssieError::InvalidDocument
    );
}
