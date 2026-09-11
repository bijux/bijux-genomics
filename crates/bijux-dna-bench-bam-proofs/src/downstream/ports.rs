use super::observations::{
    ParserObservation, PipelineObservation, PublishedVariantCounts, PublishedVariantMissingness,
};
use anyhow::Result;
use std::path::Path;
/// Acquire existing canonical evidence without linking its orchestration owner.
pub trait ParserEvidence {
    /// Acquire genotyping observations.
    ///
    /// # Errors
    /// Returns canonical acquisition or publication failure without fallback.
    fn genotyping(&self, root: &Path) -> Result<ParserObservation>;
    /// Acquire all observations.
    ///
    /// # Errors
    /// Returns canonical acquisition or publication failure without fallback.
    fn all(&self, root: &Path) -> Result<Vec<ParserObservation>>;
}
/// Acquire existing canonical evidence without linking its orchestration owner.
pub trait PipelineEvidence {
    /// Acquire validate observations.
    ///
    /// # Errors
    /// Returns canonical acquisition or publication failure without fallback.
    fn validate(
        &self,
        root: &Path,
        pipeline_id: &str,
        output_path: &Path,
    ) -> Result<PipelineObservation>;
}
/// Acquire existing canonical evidence without linking its orchestration owner.
pub trait VariantEvidence {
    /// Acquire counts observations.
    ///
    /// # Errors
    /// Returns canonical acquisition or publication failure without fallback.
    fn counts(&self, root: &Path) -> Result<PublishedVariantCounts>;
    /// Acquire missingness observations.
    ///
    /// # Errors
    /// Returns canonical acquisition or publication failure without fallback.
    fn missingness(&self, root: &Path) -> Result<PublishedVariantMissingness>;
}

/// Canonical kinship declaration report; pipeline proof remains a separate authority.
pub trait KinshipReadiness {
    /// Acquire the canonical kinship readiness report.
    ///
    /// # Errors
    /// Returns declaration, binding or native readiness acquisition failure.
    fn kinship(
        &self,
        root: &Path,
    ) -> Result<bijux_dna_bench_bam_evidence::readiness::kinship::BamKinshipReadyReport>;
}
