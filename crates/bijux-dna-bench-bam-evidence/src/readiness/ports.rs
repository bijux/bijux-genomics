use crate::{
    command_coverage::BamCommandAdapterCoverageReport,
    output_contract::BamAdapterOutputContractReport,
    parser_coverage::BamParserFixtureCoverageReport, report_map::BamReportMapReport,
};
use anyhow::Result;
use bijux_dna_bench_bindings::{
    expected::ExpectedBenchmarkResultsReport, serving::ToolServingMapReport,
};
use std::path::{Path, PathBuf};
/// Canonical published evidence surfaces. Providers own their local acquisition effects.
pub trait DeclarationEvidence {
    /// Acquire serving-map evidence.
    ///
    /// # Errors
    /// Returns source admission or publication failure.
    fn serving(&self, root: &Path) -> Result<ToolServingMapReport>;
    /// Acquire command coverage.
    ///
    /// # Errors
    /// Returns source admission or publication failure.
    fn commands(&self, root: &Path) -> Result<BamCommandAdapterCoverageReport>;
    /// Acquire output declarations.
    ///
    /// # Errors
    /// Returns source admission or publication failure.
    fn outputs(&self, root: &Path) -> Result<BamAdapterOutputContractReport>;
    /// Acquire parser fixture declaration coverage.
    ///
    /// # Errors
    /// Returns source admission or publication failure.
    fn parsers(&self, root: &Path) -> Result<BamParserFixtureCoverageReport>;
    /// Acquire expected result identities and artifact roles.
    ///
    /// # Errors
    /// Returns source admission or publication failure.
    fn expected(&self, root: &Path) -> Result<ExpectedBenchmarkResultsReport>;
    /// Acquire report placement evidence.
    ///
    /// # Errors
    /// Returns source admission or publication failure.
    fn report_map(&self, root: &Path) -> Result<BamReportMapReport>;
    /// Acquire the canonical normalized-metrics schema declaration.
    ///
    /// # Errors
    /// Returns source acquisition failure; the owner admits consumed schema fields.
    fn schema(&self) -> Result<serde_json::Value>;
}
/// Native proof-producing operation selected by the retained family contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalProofKind {
    ContaminationPlan,
    SexSmoke,
    HaplogroupsPlan,
    RecalibrationSmoke,
    GenotypingPlan,
    DamageSmoke,
    AuthenticitySmoke,
    InsertSizeSmoke,
    GcBiasSummary,
    EndogenousContentSmoke,
    OverlapCorrectionSmoke,
    KinshipSmoke,
}
/// Native execution lives in its existing owner, outside evidence compilation.
pub trait LocalProofWriter {
    /// Produce one canonical local proof and return its actual path.
    ///
    /// # Errors
    /// Returns native planning, execution or publication failure without substitution.
    fn write(&self, root: &Path, kind: LocalProofKind) -> Result<PathBuf>;
}
