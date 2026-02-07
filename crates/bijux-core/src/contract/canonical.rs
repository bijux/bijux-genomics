//! Canonical serialization helpers for contract artifacts.

use serde::Serialize;

use crate::foundation::{canonical, Result};

/// Canonical JSON format version for contract serialization.
pub const CANONICAL_JSON_VERSION: &str = "v1";

/// Canonicalize a JSON value by sorting keys and normalizing numbers/paths.
#[must_use]
pub fn canonicalize_json_value(value: &serde_json::Value) -> serde_json::Value {
    canonical::canonicalize_json_value(value)
}

/// Canonicalize parameters for stable hashing (sorted keys + normalized values).
#[must_use]
pub fn parameters_json_canonicalization(value: &serde_json::Value) -> serde_json::Value {
    canonical::parameters_json_canonicalization(value)
}

/// Canonicalize JSON for truth artifacts (manifests, records, reports).
#[must_use]
pub fn canonicalize_truth_json(value: &serde_json::Value) -> serde_json::Value {
    canonical::canonicalize_truth_json(value)
}

/// Canonical serializer for truth artifacts.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn to_canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let json = serde_json::to_value(value)?;
    let canonical = canonicalize_truth_json(&json);
    Ok(serde_json::to_vec(&canonical)?)
}
