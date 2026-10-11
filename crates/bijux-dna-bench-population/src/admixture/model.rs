//! Owner: bijux-dna-bench-population
//! Persisted population report contracts and admitted stage identities.
use serde::Serialize;
use std::collections::BTreeMap;
pub(crate) const DEFAULT_VCF_ADMIXTURE_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.admixture";
pub(crate) const LOCAL_VCF_ADMIXTURE_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_admixture_smoke.v1";
pub(crate) const LOCAL_VCF_ADMIXTURE_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-admixture-smoke";
pub(crate) const GOVERNED_VCF_ADMIXTURE_STAGE_ID: &str = bijux_dna_core::id_catalog::VCF_ADMIXTURE;
pub(crate) const GOVERNED_VCF_ADMIXTURE_CORPUS_ID: &str = "vcf_production_regression";
pub(crate) const GOVERNED_VCF_ADMIXTURE_ASSET_PROFILE_ID: &str = "vcf_cohort";
pub(crate) const GOVERNED_VCF_ADMIXTURE_INPUT_FIXTURE_ID: &str = "vcf_mini_multisample_cohort";
pub(crate) const DEFAULT_INPUT_VCF_NAME: &str = "admixture_input.vcf";
pub(crate) const DEFAULT_INPUT_SAMPLE_METADATA_NAME: &str = "sample_metadata.tsv";
pub(crate) const DEFAULT_INPUT_POPULATION_METADATA_NAME: &str = "population_metadata.tsv";
pub(crate) const DEFAULT_INPUT_POPULATION_LABELS_NAME: &str = "population_labels.json";
pub(crate) const DEFAULT_OUTPUT_TSV_NAME: &str = "admixture.tsv";
pub(crate) const DEFAULT_OUTPUT_JSON_NAME: &str = "admixture.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_Q_MATRIX_NAME: &str = "source_admixture_q_matrix.tsv";
pub(crate) const DEFAULT_OUTPUT_SOURCE_K_SELECTION_NAME: &str = "source_admixture_k_selection.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";
pub(crate) const GOVERNED_VCF_ADMIXTURE_K_VALUES: &[usize] = &[2, 3, 4];
pub(crate) const ANCESTRY_SUM_TOLERANCE: f64 = 1e-6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfAdmixtureSmokeContract {
    pub(crate) stage: String,
    pub(crate) tool: String,
    pub(crate) corpus: String,
    pub(crate) input_fixture: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfAdmixtureSmokeRow {
    pub sample_id: String,
    pub population_id: String,
    pub population_label: String,
    pub sex: String,
    #[serde(rename = "K")]
    pub k: usize,
    pub status: String,
    #[serde(flatten)]
    pub clusters: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfAdmixtureSmokeReport {
    pub schema_version: &'static str,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub fixture_manifest_path: String,
    pub input_vcf_path: String,
    pub sample_metadata_path: String,
    pub population_metadata_path: String,
    pub population_labels_manifest_path: String,
    pub output_root: String,
    pub admixture_tsv_path: String,
    pub admixture_json_path: String,
    pub source_q_matrix_path: String,
    pub source_k_selection_path: String,
    pub source_logs_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub execution_mode: String,
    pub tool_ok: bool,
    pub selected_k: usize,
    pub status: String,
    pub insufficient_data_reason: Option<String>,
    pub sample_count: u64,
    pub population_count: u64,
    pub cluster_headers: Vec<String>,
    pub rows: Vec<LocalVcfAdmixtureSmokeRow>,
}
