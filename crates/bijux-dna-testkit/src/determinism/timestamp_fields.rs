use serde_json::Value;

#[must_use]
pub fn strip_timestamp_fields(value: &Value, fields: &[&str]) -> Value {
    match value {
        Value::Object(map) => {
            let mut next = serde_json::Map::new();
            for (key, nested) in map {
                if fields.iter().any(|field| field == key) {
                    continue;
                }
                next.insert(key.clone(), strip_timestamp_fields(nested, fields));
            }
            Value::Object(next)
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| strip_timestamp_fields(item, fields))
                .collect(),
        ),
        _ => value.clone(),
    }
}
