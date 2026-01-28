use anyhow::Result;
use serde_json::Value as JsonValue;

pub trait DomainAdapter {
    type Artifact;
    type Metrics;

    /// Validate inputs for a domain-specific stage.
    ///
    /// # Errors
    /// Returns an error if inputs are invalid or incomplete.
    fn validate_inputs(&self, artifacts: &[Self::Artifact]) -> Result<()>;

    /// Compute deltas between two metric sets.
    ///
    /// # Errors
    /// Returns an error if deltas cannot be computed.
    fn compute_deltas(&self, before: &Self::Metrics, after: &Self::Metrics) -> Result<JsonValue>;

    /// Assert compatibility between upstream and downstream artifacts.
    ///
    /// # Errors
    /// Returns an error if artifacts are incompatible.
    fn compatibility(&self, from: &Self::Artifact, to: &Self::Artifact) -> Result<()>;
}
