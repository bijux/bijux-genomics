//! Owner: bijux-dna-bench-vcf
//! eigensoft adapter model contracts.
use super::Serialize;

pub const DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH: &str =
    "artifacts/benchmark-smoke/readiness/adapters/eigensoft.vcf.json";

pub const VCF_EIGENSOFT_ADAPTER_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_eigensoft_adapter.v1";

pub(super) const GOVERNED_COHORT_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfEigensoftAdapterArtifact {
    pub artifact_id: String,
    pub role: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfEigensoftAdapterCommandStep {
    pub step_id: String,
    pub step_kind: String,
    pub argv: Vec<String>,
    pub declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfEigensoftAdapterRow {
    pub tool_id: String,
    pub tool_status: String,
    pub stage_id: String,
    pub stage_status: String,
    pub benchmark_status: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub command_contract_source: String,
    pub output_root: String,
    pub output_prefix: String,
    pub convertf_par_path: String,
    pub smartpca_par_path: String,
    pub geno_path: String,
    pub snp_path: String,
    pub ind_path: String,
    pub eigenvec_path: String,
    pub eigenval_path: String,
    pub smartpca_log_path: String,
    pub normalized_metrics_artifact_id: String,
    pub normalized_metrics_path: String,
    pub stage_output_ids: Vec<String>,
    pub raw_output_ids: Vec<String>,
    pub parser_output_ids: Vec<String>,
    pub required_inputs: Vec<VcfEigensoftAdapterArtifact>,
    pub declared_outputs: Vec<VcfEigensoftAdapterArtifact>,
    pub command_steps: Vec<VcfEigensoftAdapterCommandStep>,
    pub argv_validation_passed: bool,
    pub missing_input_probe_artifact_id: String,
    pub missing_input_expected_error_fragment: String,
    pub missing_input_observed_error: String,
    pub missing_input_test_passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfEigensoftAdapterReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub tool_id: String,
    pub tool_status: String,
    pub output_path: String,
    pub row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub parser_output_row_count: usize,
    pub normalized_metrics_row_count: usize,
    pub conversion_output_row_count: usize,
    pub pca_output_row_count: usize,
    pub missing_input_test_passed_row_count: usize,
    pub rows: Vec<VcfEigensoftAdapterRow>,
}

#[derive(Debug, Clone)]
pub(super) struct RegistryToolContract {
    pub(super) tool_id: String,
    pub(super) tool_status: String,
    pub(super) stage_ids: Vec<String>,
}

pub(super) struct StageContract {
    pub(super) command_contract_source: String,
    pub(super) convertf_par_path: String,
    pub(super) smartpca_par_path: String,
    pub(super) geno_path: String,
    pub(super) snp_path: String,
    pub(super) ind_path: String,
    pub(super) eigenvec_path: String,
    pub(super) eigenval_path: String,
    pub(super) smartpca_log_path: String,
    pub(super) normalized_metrics_artifact_id: String,
    pub(super) normalized_metrics_path: String,
    pub(super) raw_output_ids: Vec<String>,
    pub(super) parser_output_ids: Vec<String>,
    pub(super) declared_outputs: Vec<VcfEigensoftAdapterArtifact>,
    pub(super) command_steps: Vec<VcfEigensoftAdapterCommandStep>,
}
