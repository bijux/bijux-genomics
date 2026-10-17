//! Canonical serialization helpers for contract artifacts.

use serde::Serialize;

use crate::foundation::{canonical, Result};

/// Canonical JSON format version for contract serialization.
pub const CANONICAL_JSON_VERSION: &str = "v1";

/// Sort JSON object keys recursively while preserving array order and scalar values.
///
/// This operation does not normalize paths, redact strings or inspect the environment.
#[must_use]
pub fn canonicalize_json_value(value: &serde_json::Value) -> serde_json::Value {
    canonical::canonicalize_json_value(value)
}

/// Normalize parameter JSON using the path and privacy rules of truth canonicalization.
///
/// String values can depend on `USER`, `USERNAME` and `HOSTNAME`; see the serialization
/// contract for normalization and environment requirements.
#[must_use]
pub fn parameters_json_canonicalization(value: &serde_json::Value) -> serde_json::Value {
    canonical::parameters_json_canonicalization(value)
}

/// Sort keys and normalize numeric and string values for truth artifacts.
///
/// This includes lexical path normalization and environment-sensitive privacy
/// substitutions. Arrays retain their input order.
#[must_use]
pub fn canonicalize_truth_json(value: &serde_json::Value) -> serde_json::Value {
    canonical::canonicalize_truth_json(value)
}

/// Serialize truth-normalized JSON using the declared path and privacy rules.
///
/// Byte identity depends on the input values and normalization environment. This is
/// not a raw-string-preserving serializer.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn to_canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let json = serde_json::to_value(value)?;
    let canonical = canonicalize_truth_json(&json);
    Ok(serde_json::to_vec(&canonical)?)
}
