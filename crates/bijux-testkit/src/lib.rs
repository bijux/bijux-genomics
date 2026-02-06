//! Shared test fixtures/helpers for bijux crates.

pub mod snapshots {
    use serde_json::Value;

    #[must_use]
    pub fn stable_json(value: &Value) -> Value {
        sort_value(value)
    }

    fn sort_value(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut entries: Vec<(String, Value)> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), sort_value(v)))
                    .collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                let mut sorted = serde_json::Map::new();
                for (k, v) in entries {
                    sorted.insert(k, v);
                }
                Value::Object(sorted)
            }
            Value::Array(items) => Value::Array(items.iter().map(sort_value).collect()),
            _ => value.clone(),
        }
    }
}
