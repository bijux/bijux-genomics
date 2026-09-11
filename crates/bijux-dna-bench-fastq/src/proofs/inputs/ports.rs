use super::observations::{
    CommandArgvObservation, ComparableMetricObservation, ExpectedResultObservation, PublishedSchema,
};
use crate::readiness::{
    active::FastqActiveStageToolMatrixRow, adapters::FastqCommandAdapterCoverageRow,
    output_contract::FastqAdapterOutputContractRow, parsers::FastqParserCoverageRow,
    report_map::FastqReportMapRow,
};
use anyhow::Result;
use std::path::Path;
/// Acquire canonical evidence without changing selection, fields or failure behavior.
pub trait DeclarationEvidence {
    /// Acquire active evidence.
    ///
    /// # Errors
    /// Preserves canonical acquisition and publication failures.
    fn active(&self, root: &Path) -> Result<Vec<FastqActiveStageToolMatrixRow>>;
    /// Acquire outputs evidence.
    ///
    /// # Errors
    /// Preserves canonical acquisition and publication failures.
    fn outputs(&self, root: &Path) -> Result<Vec<FastqAdapterOutputContractRow>>;
    /// Acquire parsers evidence.
    ///
    /// # Errors
    /// Preserves canonical acquisition and publication failures.
    fn parsers(&self, root: &Path) -> Result<Vec<FastqParserCoverageRow>>;
    /// Acquire expected evidence.
    ///
    /// # Errors
    /// Preserves canonical acquisition and publication failures.
    fn expected(&self, root: &Path) -> Result<Vec<ExpectedResultObservation>>;
    /// Acquire report mapping evidence.
    ///
    /// # Errors
    /// Preserves canonical acquisition and publication failures.
    fn report_map(&self, root: &Path) -> Result<Vec<FastqReportMapRow>>;
}
/// Acquire canonical evidence without changing selection, fields or failure behavior.
pub trait SchemaEvidence {
    /// Acquire commands evidence.
    ///
    /// # Errors
    /// Preserves canonical acquisition and publication failures.
    fn commands(&self, root: &Path) -> Result<Vec<FastqCommandAdapterCoverageRow>>;
    /// Acquire publish evidence.
    ///
    /// # Errors
    /// Preserves canonical acquisition and publication failures.
    fn publish(&self, root: &Path) -> Result<PublishedSchema>;
    /// Return the canonical authored JSON Schema document, not a tool output payload.
    fn document(&self) -> serde_json::Value;
}
/// Acquire canonical evidence without changing selection, fields or failure behavior.
pub trait ValidationEvidence {
    /// Acquire metrics evidence.
    ///
    /// # Errors
    /// Preserves canonical acquisition and publication failures.
    fn metrics(&self, root: &Path) -> Result<Vec<ComparableMetricObservation>>;
    /// Acquire argv evidence.
    ///
    /// # Errors
    /// Preserves canonical acquisition and publication failures.
    fn argv(&self, root: &Path) -> Result<Vec<CommandArgvObservation>>;
    /// Acquire materialize evidence.
    ///
    /// # Errors
    /// Preserves canonical acquisition and publication failures.
    fn materialize(&self, root: &Path, stage_id: &str) -> Result<()>;
}
