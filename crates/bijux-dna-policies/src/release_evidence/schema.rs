use schemars::schema_for;

use super::ContainerReleaseEvidencePublication;

/// Render the release-evidence schema owned by the Rust policy type.
///
/// # Panics
/// Panics only if the generated schema cannot be represented as JSON.
#[must_use]
pub fn release_evidence_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(ContainerReleaseEvidencePublication))
        .unwrap_or_else(|error| panic!("serialize release evidence schema: {error}"))
}
