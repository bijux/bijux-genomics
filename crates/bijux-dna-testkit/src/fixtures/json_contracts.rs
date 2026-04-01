use serde_json::Value;

/// Assert that `value` contains all top-level keys present in `schema_like`.
///
/// # Panics
/// Panics if `schema_like` is not a JSON object or if any expected key is missing.
pub fn assert_json_schema_like(value: &Value, schema_like: &Value) {
    match (value, schema_like) {
        (Value::Object(actual), Value::Object(schema)) => {
            for key in schema.keys() {
                assert!(
                    actual.contains_key(key),
                    "missing expected key '{key}' in json payload"
                );
            }
        }
        _ => panic!("schema_like must be a JSON object"),
    }
}
