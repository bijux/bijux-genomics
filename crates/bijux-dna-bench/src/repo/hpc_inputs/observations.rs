//! Owner: typed stage hints and observed HPC source identities.
use crate::BenchmarkSourceAsset;
/// Declared source role supplied by a stage-plan adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HpcStageInputHint {
    /// Declared artifact identifier, when available.
    pub artifact_id: String,
    /// Declared artifact role, when available.
    pub artifact_role: String,
    /// Source path interpreted relative to the explicit repository root.
    pub source_path: String,
}

/// Current content identity joined to optional declared stage provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HpcDiscoveredSourceInput {
    /// Declared artifact identifier, when available.
    pub artifact_id: Option<String>,
    /// Declared artifact role, when available.
    pub artifact_role: Option<String>,
    /// Fresh file, directory or prefix-bundle content observation.
    pub source_asset: BenchmarkSourceAsset,
}
