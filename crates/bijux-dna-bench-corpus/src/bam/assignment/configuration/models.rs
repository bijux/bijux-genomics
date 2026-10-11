use serde::Deserialize;
use std::path::PathBuf;
#[derive(Debug, Deserialize)]
pub(in crate::bam::assignment) struct LocalAuthenticityConfig {
    pub(in crate::bam::assignment) cases: Vec<LocalAuthenticityCase>,
}
#[derive(Debug, Deserialize)]
pub(in crate::bam::assignment) struct LocalAuthenticityCase {
    pub(in crate::bam::assignment) sample_id: String,
    pub(in crate::bam::assignment) complexity_min_reads: u64,
    pub(in crate::bam::assignment) coverage_depth_thresholds: Vec<u64>,
}
#[derive(Debug, Deserialize)]
pub(in crate::bam::assignment) struct LocalContaminationConfig {
    pub(in crate::bam::assignment) sample_id: String,
    pub(in crate::bam::assignment) minimum_mean_coverage: f64,
    pub(in crate::bam::assignment) reference_fasta: PathBuf,
    pub(in crate::bam::assignment) reference_panels: Vec<PathBuf>,
}
#[derive(Debug, Deserialize)]
pub(in crate::bam::assignment) struct LocalDamageConfig {
    pub(in crate::bam::assignment) cases: Vec<LocalDamageCase>,
}
#[derive(Debug, Deserialize)]
pub(in crate::bam::assignment) struct LocalDamageCase {
    pub(in crate::bam::assignment) sample_id: String,
}
#[derive(Debug, Deserialize)]
pub(in crate::bam::assignment) struct LocalHaplogroupsConfig {
    pub(in crate::bam::assignment) sample_id: String,
    pub(in crate::bam::assignment) min_coverage: f64,
    pub(in crate::bam::assignment) reference_fasta: PathBuf,
    pub(in crate::bam::assignment) reference_panel_id: String,
}
#[derive(Debug, Deserialize)]
pub(in crate::bam::assignment) struct LocalGenotypingConfig {
    pub(in crate::bam::assignment) sample_id: String,
    pub(in crate::bam::assignment) reference_fasta: PathBuf,
    pub(in crate::bam::assignment) sites_vcf: PathBuf,
    pub(in crate::bam::assignment) regions: PathBuf,
    pub(in crate::bam::assignment) min_posterior: f64,
    pub(in crate::bam::assignment) min_call_rate: f64,
}
#[derive(Debug, Deserialize)]
pub(in crate::bam::assignment) struct LocalKinshipConfig {
    pub(in crate::bam::assignment) cases: Vec<LocalKinshipCase>,
}
#[derive(Debug, Deserialize)]
pub(in crate::bam::assignment) struct LocalKinshipCase {
    pub(in crate::bam::assignment) sample_id: String,
    pub(in crate::bam::assignment) bam: PathBuf,
    pub(in crate::bam::assignment) reference_panel: String,
    pub(in crate::bam::assignment) reference_build: String,
    pub(in crate::bam::assignment) population_scope: String,
    pub(in crate::bam::assignment) min_overlap_snps: u32,
    pub(in crate::bam::assignment) expected_status: String,
    pub(in crate::bam::assignment) expected_observed_max_overlap_snps: u32,
    #[serde(default)]
    pub(in crate::bam::assignment) expected_pairwise_results: Vec<LocalKinshipPairwiseResult>,
}
#[derive(Debug, Deserialize)]
pub(in crate::bam::assignment) struct LocalKinshipPairwiseResult {
    pub(in crate::bam::assignment) relationship_label: String,
}
#[derive(Debug, Deserialize)]
pub(in crate::bam::assignment) struct LocalSexConfig {
    pub(in crate::bam::assignment) cases: Vec<LocalSexCase>,
}
#[derive(Debug, Deserialize)]
pub(in crate::bam::assignment) struct LocalSexCase {
    pub(in crate::bam::assignment) sample_id: String,
    pub(in crate::bam::assignment) minimum_y_sites: u64,
    pub(in crate::bam::assignment) expected_x_coverage: f64,
    pub(in crate::bam::assignment) expected_y_coverage: f64,
    pub(in crate::bam::assignment) expected_autosomal_coverage: f64,
    pub(in crate::bam::assignment) reference: PathBuf,
}
