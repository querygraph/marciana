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
fn metric_expressions_are_bound_into_the_binding_digest() {
    let binding = OssieAdapter::import_json("lakecat:agstack/coffee/v1", MODEL).expect("binding");
    let reexpressed = MODEL.replace("avg(price_usd_per_kg)", "max(price_usd_per_kg)");
    let other =
        OssieAdapter::import_json("lakecat:agstack/coffee/v1", &reexpressed).expect("binding");
    assert_ne!(binding.digest(), other.digest());
}

#[test]
fn section_boundaries_are_bound_into_the_binding_digest() {
    let metric_heavy = r#"
{
  "namespace": "agstack",
  "name": "boundary",
  "version": 1,
  "metrics": [{"name": "alpha", "expression": ""}, {"name": "beta", "expression": ""}],
  "dimensions": []
}
"#;
    let split = r#"
{
  "namespace": "agstack",
  "name": "boundary",
  "version": 1,
  "metrics": [{"name": "alpha", "expression": ""}],
  "dimensions": [{"name": "beta", "role": ""}]
}
"#;
    let first = OssieAdapter::import_json("lakecat:agstack/boundary/v1", metric_heavy)
        .expect("metric-heavy binding");
    let second =
        OssieAdapter::import_json("lakecat:agstack/boundary/v1", split).expect("split binding");
    assert_ne!(first.digest(), second.digest());
}

#[test]
fn rejects_control_characters_in_the_source_manifest() {
    assert_eq!(
        OssieAdapter::import_json("lakecat:agstack\tcoffee", MODEL).expect_err("control character"),
        OssieError::InvalidSourceManifest
    );
}

#[test]
fn rejects_unknown_semantics_and_duplicate_names() {
    let binding = OssieAdapter::import_json("lakecat:agstack/coffee/v1", MODEL).expect("binding");
    assert_eq!(
        OssieAdapter::plan_query(&binding, "unknown", vec!["market".into()])
            .expect_err("unknown metric"),
        OssieError::UnknownMetric
    );
    for dimension in ["aardvark", "lane", "zulu"] {
        assert_eq!(
            OssieAdapter::plan_query(&binding, "price_usd_per_kg", vec![dimension.to_owned()],)
                .expect_err("unknown dimension"),
            OssieError::UnknownDimension
        );
    }
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
