use super::observations::{
    AmpliconFixtureObservation, CorpusSamplesObservation, DamageFixtureObservation,
    EdnaFixtureObservation, TaxonomyFixtureObservation,
};
use anyhow::Result;
use bijux_dna_bench_model::{
    BenchmarkLocalCorpusSkipReport, BenchmarkLocalCorpusStageCompatibilityReport,
};
use std::path::Path;
/// Supply typed local corpora evidence; provider failures remain explicit readiness diagnostics.
pub trait CorpusEvidence {
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn validate_fastq_corpus_fixture_manifest_path(
        &self,
        repo_root: &Path,
        manifest_path: &Path,
    ) -> Result<CorpusSamplesObservation>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn validate_bam_corpus_fixture_manifest_path(
        &self,
        repo_root: &Path,
        manifest_path: &Path,
    ) -> Result<CorpusSamplesObservation>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn validate_bam_damage_fixture_manifest_path(
        &self,
        repo_root: &Path,
        manifest_path: &Path,
    ) -> Result<DamageFixtureObservation>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn validate_edna_corpus_fixture_manifest_path(
        &self,
        repo_root: &Path,
        manifest_path: &Path,
    ) -> Result<EdnaFixtureObservation>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn validate_amplicon_corpus_fixture_manifest_path(
        &self,
        repo_root: &Path,
        manifest_path: &Path,
    ) -> Result<AmpliconFixtureObservation>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn validate_taxonomy_database_fixture_manifest_path(
        &self,
        repo_root: &Path,
        manifest_path: &Path,
    ) -> Result<TaxonomyFixtureObservation>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn validate_corpus_stage_compatibility_path(
        &self,
        repo_root: &Path,
        matrix_path: &Path,
    ) -> Result<BenchmarkLocalCorpusStageCompatibilityReport>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn render_corpus_skip_report_path(
        &self,
        repo_root: &Path,
        matrix_path: &Path,
        output_path: &Path,
    ) -> Result<BenchmarkLocalCorpusSkipReport>;
}
