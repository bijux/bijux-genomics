use serde_json::Value;

use crate::snapshots::stable_json;

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
