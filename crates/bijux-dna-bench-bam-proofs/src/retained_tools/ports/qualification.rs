use super::super::observations::MicroSmokeObservation;
use super::super::observations::ScienceThresholdsObservation;
use anyhow::Result;
use bijux_dna_bench_bam_evidence::consistency::BamActiveRowConsistencyReport;
use std::path::Path;
/// Acquire canonical evidence at the requested root; failures retain their source identity.
pub trait QualificationEvidence {
    /// Acquire active row consistency evidence.
    ///
    /// # Errors
    /// Returns the original provider acquisition or publication failure.
    fn active_row_consistency(&self, root: &Path) -> Result<BamActiveRowConsistencyReport>;
    /// Acquire micro smoke summary evidence.
    ///
    /// # Errors
    /// Returns the original provider acquisition or publication failure.
    fn micro_smoke_summary(&self, root: &Path) -> Result<MicroSmokeObservation>;
    /// Acquire science thresholds evidence.
    ///
    /// # Errors
    /// Returns the original provider acquisition or publication failure.
    fn science_thresholds(&self, root: &Path) -> Result<ScienceThresholdsObservation>;
}
