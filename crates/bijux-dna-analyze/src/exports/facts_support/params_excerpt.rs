pub(super) fn params_excerpt(value: &serde_json::Value, limit: usize) -> serde_json::Value {
    let Some(obj) = value.as_object() else {
        return value.clone();
    };
    let mut keys: Vec<_> = obj.keys().cloned().collect();
    keys.sort();
    let mut out = serde_json::Map::new();
    for key in keys.into_iter().take(limit) {
        if let Some(v) = obj.get(&key) {
            out.insert(key, v.clone());
        }
    }
    serde_json::Value::Object(out)
}
