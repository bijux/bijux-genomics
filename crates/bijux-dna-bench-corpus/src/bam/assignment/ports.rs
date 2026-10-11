use anyhow::Result;
use std::path::Path;
/// Nonserialized command observations consumed by corpus assignment.
pub struct CoverageRow {
    pub stage_id: String,
    pub tool_id: String,
    pub benchmark_ready: bool,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
}
pub struct Coverage {
    pub stage_count: usize,
    pub tool_count: usize,
    pub rows: Vec<CoverageRow>,
}
/// Consumed fields from the canonical compatibility matrix.
pub struct StageCompatibility {
    pub stage_id: String,
    pub corpus_family_id: Option<String>,
    pub fixture_id: Option<String>,
}
pub trait CoverageEvidence {
    /// Acquire canonical command coverage.
    ///
    /// # Errors
    /// Returns source acquisition failure without replacing it with another source.
    fn coverage(&self, root: &Path) -> Result<Coverage>;
}
pub trait CompatibilityEvidence {
    /// Acquire the canonical corpus compatibility matrix.
    ///
    /// # Errors
    /// Returns source admission failure.
    fn compatibility(&self, root: &Path) -> Result<Vec<StageCompatibility>>;
}
pub trait InventoryEvidence {
    /// Acquire the published stage inventory count.
    ///
    /// # Errors
    /// Returns canonical inventory acquisition failure.
    fn stage_count(&self, root: &Path) -> Result<usize>;
}
