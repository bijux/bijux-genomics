use serde_json::Value;
pub(super) fn string_json(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(ToOwned::to_owned)
}
pub(super) fn optional_string_json(value: &Value, key: &str) -> Option<String> {
    string_json(value, key)
}
pub(super) fn bool_json(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}
pub(super) fn u64_json(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}
pub(super) fn f64_json(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}
