use schemars::schema_for;

use super::ContainerReleaseEvidencePublication;

/// Render the release-evidence schema owned by the Rust policy type.
///
#[must_use]
pub fn release_evidence_schema() -> serde_json::Value {
    schema_for!(ContainerReleaseEvidencePublication).into()
}
