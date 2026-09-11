use std::collections::BTreeSet;
pub(in crate::readiness) fn collect_local_smoke_fields(
    values: &[serde_json::Value],
) -> Vec<String> {
    let mut fields = values
        .iter()
        .filter_map(serde_json::Value::as_object)
        .flat_map(|value| value.keys().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    fields.sort();
    fields
}
pub(in crate::readiness) fn collect_json_fields(values: &[serde_json::Value]) -> Vec<String> {
    fn visit(value: &serde_json::Value, fields: &mut BTreeSet<String>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    fields.insert(key.clone());
                    visit(value, fields);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    visit(item, fields);
                }
            }
            serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_) => {}
        }
    }

    let mut fields = BTreeSet::new();
    for value in values {
        visit(value, &mut fields);
    }
    fields.into_iter().collect()
}
