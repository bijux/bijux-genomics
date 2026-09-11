use super::super::observations::ExpectedResultsObservation;
use super::super::observations::LocalSmokeObservation;
use super::super::observations::RenderedCommandsObservation;
use super::super::observations::ToolMapObservation;
use anyhow::Result;
use bijux_dna_bench_bam_evidence::command_coverage::BamCommandAdapterCoverageReport;
use bijux_dna_bench_bam_evidence::output_contract::BamAdapterOutputContractReport;
use bijux_dna_bench_bam_evidence::parser_coverage::BamParserFixtureCoverageReport;
use bijux_dna_bench_bam_evidence::report_map::BamReportMapReport;
use std::path::Path;
/// Acquire canonical evidence at the requested root; failures retain their source identity.
pub trait BindingEvidence {
    /// Acquire tool map evidence.
    ///
    /// # Errors
    /// Returns the original provider acquisition or publication failure.
    fn tool_map(&self, root: &Path) -> Result<ToolMapObservation>;
    /// Acquire command coverage evidence.
    ///
    /// # Errors
    /// Returns the original provider acquisition or publication failure.
    fn command_coverage(&self, root: &Path) -> Result<BamCommandAdapterCoverageReport>;
    /// Acquire parser coverage evidence.
    ///
    /// # Errors
    /// Returns the original provider acquisition or publication failure.
    fn parser_coverage(&self, root: &Path) -> Result<BamParserFixtureCoverageReport>;
    /// Acquire local smoke evidence.
    ///
    /// # Errors
    /// Returns the original provider acquisition or publication failure.
    fn local_smoke(&self, root: &Path) -> Result<LocalSmokeObservation>;
    /// Acquire output declarations evidence.
    ///
    /// # Errors
    /// Returns the original provider acquisition or publication failure.
    fn output_declarations(&self, root: &Path) -> Result<BamAdapterOutputContractReport>;
    /// Acquire rendered commands evidence.
    ///
    /// # Errors
    /// Returns the original provider acquisition or publication failure.
    fn rendered_commands(&self, root: &Path) -> Result<RenderedCommandsObservation>;
    /// Acquire expected results evidence.
    ///
    /// # Errors
    /// Returns the original provider acquisition or publication failure.
    fn expected_results(&self, root: &Path) -> Result<ExpectedResultsObservation>;
    /// Acquire report map evidence.
    ///
    /// # Errors
    /// Returns the original provider acquisition or publication failure.
    fn report_map(&self, root: &Path) -> Result<BamReportMapReport>;
}
