use bijux_core::contract::canonical::{canonicalize_json_value, parameters_json_canonicalization};
use bijux_core::metrics::metrics_schema_for_stage;

#[test]
fn canonicalize_json_value_sorts_keys() {
    let input = serde_json::json!({
        "b": 1,
        "a": { "d": 2, "c": 1 }
    });
    let canonical = canonicalize_json_value(&input);
    let rendered = match serde_json::to_string(&canonical) {
        Ok(value) => value,
        Err(err) => panic!("serialize failed: {err}"),
    };
    assert!(
        rendered.starts_with("{\"a\":"),
        "expected canonical keys sorted: {rendered}"
    );
}

#[test]
fn parameters_json_canonicalization_normalizes_numbers() {
    let input = serde_json::json!({
        "a": 1,
        "b": 1.0
    });
    let canonical = parameters_json_canonicalization(&input);
    let rendered = match serde_json::to_string(&canonical) {
        Ok(value) => value,
        Err(err) => panic!("serialize failed: {err}"),
    };
    assert!(rendered.contains("\"a\":1"));
    assert!(rendered.contains("\"b\":1"));
}

#[test]
fn metrics_schema_resolves_stage() {
    let schema = metrics_schema_for_stage("fastq.trim");
    assert!(schema.is_some());
}
