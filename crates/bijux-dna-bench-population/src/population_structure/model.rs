//! Owner: bijux-dna-bench-population
//! Persisted population report contracts and admitted stage identities.
use serde::Serialize;
pub(crate) const DEFAULT_VCF_POPULATION_STRUCTURE_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.population_structure";
pub(crate) const LOCAL_VCF_POPULATION_STRUCTURE_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_population_structure_smoke.v1";
pub(crate) const LOCAL_VCF_POPULATION_STRUCTURE_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-population-structure-smoke";
pub(crate) const GOVERNED_VCF_POPULATION_STRUCTURE_STAGE_ID: &str =
    bijux_dna_core::id_catalog::VCF_POPULATION_STRUCTURE;
pub(crate) const GOVERNED_VCF_POPULATION_STRUCTURE_CORPUS_ID: &str = "vcf_production_regression";
pub(crate) const GOVERNED_VCF_POPULATION_STRUCTURE_ASSET_PROFILE_ID: &str = "vcf_cohort";
pub(crate) const GOVERNED_VCF_POPULATION_STRUCTURE_INPUT_FIXTURE_ID: &str =
    "vcf_mini_multisample_cohort";
pub(crate) const DEFAULT_OUTPUT_REPORT_NAME: &str = "population_structure.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_STAGE_REPORT_NAME: &str = "source_population_structure.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_PRUNED_VARIANTS_NAME: &str = "source_pruned_variants.tsv";
pub(crate) const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
pub(crate) const DEFAULT_OUTPUT_SOURCE_PCA_REPORT_NAME: &str = "source_pca.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_ADMIXTURE_REPORT_NAME: &str = "source_admixture.json";
pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfPopulationStructureSmokeContract {
    pub(crate) stage: String,
    pub(crate) tool: String,
    pub(crate) pca_tool_id: String,
    pub(crate) admixture_tool_id: String,
    pub(crate) normalization_pca_tool_id: String,
    pub(crate) corpus: String,
    pub(crate) input_fixture: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConsumedPcaReport {
    pub report_path: String,
    pub sample_count: u64,
    pub execution_mode: String,
    pub tool_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConsumedAdmixtureReport {
    pub report_path: String,
    pub sample_count: u64,
    pub selected_k: u64,
    pub execution_mode: String,
    pub tool_ok: bool,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfPopulationStructureSampleGroup {
    pub sample_id: String,
    pub population_id: String,
    pub population_label: String,
    pub sex: String,
    pub dominant_cluster: String,
    pub dominant_fraction: f64,
    pub pc1: f64,
    pub pc2: f64,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfPopulationStructureDistanceSummary {
    pub sample_count: u64,
    pub pair_count: u64,
    pub within_population_pair_count: u64,
    pub cross_population_pair_count: u64,
    pub min_pc_distance: f64,
    pub max_pc_distance: f64,
    pub mean_pc_distance: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfPopulationStructureSmokeReport {
    pub schema_version: &'static str,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub output_root: String,
    pub population_structure_json_path: String,
    pub source_population_structure_path: String,
    pub source_pruned_variants_path: String,
    pub source_logs_path: String,
    pub source_pca_report_path: String,
    pub source_admixture_report_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub selected_pca_tool_id: String,
    pub selected_pca_tool_ok: bool,
    pub selected_admixture_tool_id: String,
    pub selected_admixture_tool_ok: bool,
    pub consumed_pca: ConsumedPcaReport,
    pub consumed_admixture: ConsumedAdmixtureReport,
    pub sample_groups: Vec<LocalVcfPopulationStructureSampleGroup>,
    pub distance_summary: LocalVcfPopulationStructureDistanceSummary,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PcaSampleRow {
    pub(crate) sample_id: String,
    pub(crate) population_id: String,
    pub(crate) population_label: String,
    pub(crate) sex: String,
    pub(crate) pc1: f64,
    pub(crate) pc2: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AdmixtureSampleRow {
    pub(crate) sample_id: String,
    pub(crate) population_id: String,
    pub(crate) population_label: String,
    pub(crate) sex: String,
    pub(crate) status: String,
    pub(crate) clusters: Vec<(String, f64)>,
}
