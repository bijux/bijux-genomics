//! Owner: bijux-dna-bench-population
//! Persisted population report contracts and admitted stage identities.
use serde::Serialize;
pub(crate) const DEFAULT_VCF_PCA_SMOKE_ROOT: &str = "artifacts/runs/bench/local-smoke/vcf.pca";
pub(crate) const LOCAL_VCF_PCA_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_pca_smoke.v1";
pub(crate) const LOCAL_VCF_PCA_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-pca-smoke";
pub(crate) const GOVERNED_VCF_PCA_STAGE_ID: &str = bijux_dna_core::id_catalog::VCF_PCA;
pub(crate) const GOVERNED_VCF_PCA_CORPUS_ID: &str = "vcf_production_regression";
pub(crate) const GOVERNED_VCF_PCA_ASSET_PROFILE_ID: &str = "vcf_cohort";
pub(crate) const GOVERNED_VCF_PCA_INPUT_FIXTURE_ID: &str = "vcf_mini_multisample_cohort";
pub(crate) const DEFAULT_INPUT_VCF_NAME: &str = "pca_input.vcf";
pub(crate) const DEFAULT_INPUT_SAMPLE_METADATA_NAME: &str = "sample_metadata.tsv";
pub(crate) const DEFAULT_INPUT_POPULATION_METADATA_NAME: &str = "population_metadata.tsv";
pub(crate) const DEFAULT_INPUT_POPULATION_LABELS_NAME: &str = "population_labels.json";
pub(crate) const DEFAULT_OUTPUT_TSV_NAME: &str = "pca.tsv";
pub(crate) const DEFAULT_OUTPUT_JSON_NAME: &str = "pca.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME: &str = "source_eigenvec.tsv";
pub(crate) const DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME: &str = "source_eigenval.tsv";
pub(crate) const DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME: &str = "source_pca_manifest.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";
pub(crate) const GOVERNED_PCA_COMPONENT_COUNT: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfPcaSmokeContract {
    pub(crate) stage: String,
    pub(crate) tool: String,
    pub(crate) corpus: String,
    pub(crate) input_fixture: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfPcaSmokeRow {
    pub sample_id: String,
    pub population_id: String,
    pub population_label: String,
    pub sex: String,
    pub pc1: f64,
    pub pc2: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfPcaSmokeReport {
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
    pub pca_tsv_path: String,
    pub pca_json_path: String,
    pub source_eigenvec_path: String,
    pub source_eigenval_path: String,
    pub source_pca_manifest_path: String,
    pub source_logs_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub execution_mode: String,
    pub tool_ok: bool,
    pub variant_count: u64,
    pub sample_count: u64,
    pub excluded_samples: Vec<String>,
    pub unexpected_samples: Vec<String>,
    pub eigenvalues: Vec<f64>,
    pub rows: Vec<LocalVcfPcaSmokeRow>,
}
