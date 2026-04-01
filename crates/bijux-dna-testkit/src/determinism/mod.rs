mod clock;
mod rng;

use serde_json::Value;

use crate::snapshots::stable_json;

pub use clock::FixedClock;
pub use rng::fixed_rng;

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

/// Assert a slice is already sorted in deterministic order.
///
/// # Panics
/// Panics if `items` are not sorted.
pub fn assert_stable_ordering<T: Ord + std::fmt::Debug + Clone>(items: &[T]) {
    let mut sorted = items.to_vec();
    sorted.sort();
    assert_eq!(items, sorted, "items must be sorted deterministically");
}

/// Assert two JSON values are equal after stable ordering normalization.
///
/// # Panics
/// Panics if normalized JSON values differ.
pub fn assert_json_stable(expected: &Value, actual: &Value) {
    assert_eq!(
        stable_json(expected),
        stable_json(actual),
        "JSON must be deterministically ordered"
    );
}
