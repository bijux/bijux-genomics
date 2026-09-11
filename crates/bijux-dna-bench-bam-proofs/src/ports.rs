use anyhow::Result;
use bijux_dna_bench_bam_evidence::readiness::contamination_sex_haplogroups::BamContaminationSexHaplogroupsReadyReport;
use bijux_dna_bench_bam_evidence::readiness::damage_authenticity::BamDamageAuthenticityReadyReport;
use bijux_dna_bench_bam_evidence::readiness::overlap_endogenous::BamOverlapEndogenousReadyReport;
use bijux_dna_bench_bam_evidence::readiness::recalibration_genotyping::BamRecalibrationGenotypingReadyReport;
use std::path::{Path, PathBuf};
/// Canonical readiness reports supplied independently of native stage execution.
pub trait ReadinessEvidence {
    /// Acquire the canonical damage authenticity report.
    ///
    /// # Errors
    /// Returns evidence acquisition or publication failure.
    fn damage_authenticity(&self, root: &Path) -> Result<BamDamageAuthenticityReadyReport>;
    /// Acquire the canonical contamination sex haplogroups report.
    ///
    /// # Errors
    /// Returns evidence acquisition or publication failure.
    fn contamination_sex_haplogroups(
        &self,
        root: &Path,
    ) -> Result<BamContaminationSexHaplogroupsReadyReport>;
    /// Acquire the canonical overlap endogenous report.
    ///
    /// # Errors
    /// Returns evidence acquisition or publication failure.
    fn overlap_endogenous(&self, root: &Path) -> Result<BamOverlapEndogenousReadyReport>;
    /// Acquire the canonical recalibration genotyping report.
    ///
    /// # Errors
    /// Returns evidence acquisition or publication failure.
    fn recalibration_genotyping(
        &self,
        root: &Path,
    ) -> Result<BamRecalibrationGenotypingReadyReport>;
}
/// Native operation identity is retained; backends cannot substitute another smoke path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeProofKind {
    Authenticity,
    Contamination,
    Damage,
    Haplogroups,
    Recalibration,
    Sex,
}
/// Adapter for existing native proof publication; no process-global root changes are allowed.
pub trait NativeProofWriter {
    /// Produce the requested proof inside the admitted repository.
    ///
    /// # Errors
    /// Returns unsupported mode, root mismatch, or native execution failure.
    fn write(&self, root: &Path, kind: NativeProofKind) -> Result<PathBuf>;
}
