//! Owner: bijux-dna-bench-vcf
//! imputation adapter model contracts.
use super::Serialize;

pub const DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH: &str =
    "artifacts/benchmark-smoke/readiness/adapters/imputation-family.vcf.json";

pub const VCF_IMPUTATION_FAMILY_ADAPTER_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_imputation_family_adapter.v1";

pub(super) const GOVERNED_IMPUTATION_STAGE_IDS: [&str; 2] =
    ["vcf.imputation_metrics", "vcf.impute"];

pub(super) const GOVERNED_IMPUTATION_TOOL_IDS: [&str; 4] =
    ["beagle", "glimpse", "impute5", "minimac4"];

pub(super) const VCF_IMPUTATION_REGISTRY_PATHS: [&str; 2] = [
    "configs/ci/registry/tool_registry_vcf.toml",
    "configs/ci/registry/tool_registry_vcf_downstream.toml",
];

pub(super) const GOVERNED_GTCOHORT_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_phased.vcf";

pub(super) const GOVERNED_GLLIKE_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf";

pub(super) const GOVERNED_REGION_LITERAL: &str = "1:1-1000000";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfImputationFamilyAdapterArtifact {
    pub artifact_id: String,
    pub role: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfImputationFamilyAdapterCommandStep {
    pub step_id: String,
    pub step_kind: String,
    pub argv: Vec<String>,
    pub declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfImputationFamilyAdapterRow {
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
    pub panel_id: String,
    pub map_id: String,
    pub target_vcf_path: String,
    pub panel_vcf_path: String,
    pub panel_msav_path: Option<String>,
    pub genetic_map_path: String,
    pub region_literal: Option<String>,
    pub imputed_vcf_path: String,
    pub imputed_vcf_tbi_path: String,
    pub quality_output_path: String,
    pub quality_tsv_path: Option<String>,
    pub warnings_path: Option<String>,
    pub imputation_manifest_path: String,
    pub orchestration_manifest_path: Option<String>,
    pub panel_mismatch_diagnostics_path: Option<String>,
    pub log_output_path: String,
    pub stage_output_ids: Vec<String>,
    pub raw_output_ids: Vec<String>,
    pub parser_output_ids: Vec<String>,
    pub required_inputs: Vec<VcfImputationFamilyAdapterArtifact>,
    pub declared_outputs: Vec<VcfImputationFamilyAdapterArtifact>,
    pub command_steps: Vec<VcfImputationFamilyAdapterCommandStep>,
    pub argv_validation_passed: bool,
    pub missing_input_probe_artifact_id: String,
    pub missing_input_expected_error_fragment: String,
    pub missing_input_observed_error: String,
    pub missing_input_test_passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfImputationFamilyAdapterReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub output_path: String,
    pub row_count: usize,
    pub tool_count: usize,
    pub benchmark_ready_row_count: usize,
    pub parser_output_row_count: usize,
    pub missing_input_test_passed_row_count: usize,
    pub rows: Vec<VcfImputationFamilyAdapterRow>,
}

#[derive(Debug, Clone)]
pub(super) struct RegistryToolContract {
    pub(super) tool_id: String,
    pub(super) tool_status: String,
    pub(super) stage_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub(super) struct MaterializedPanelInputs {
    pub(super) panel_id: String,
    pub(super) map_id: String,
    pub(super) panel_vcf_path: String,
    pub(super) panel_msav_path: Option<String>,
    pub(super) genetic_map_path: String,
}

pub(super) struct StageContract {
    pub(super) command_contract_source: String,
    pub(super) imputed_vcf_path: String,
    pub(super) imputed_vcf_tbi_path: String,
    pub(super) quality_output_path: String,
    pub(super) quality_tsv_path: Option<String>,
    pub(super) warnings_path: Option<String>,
    pub(super) imputation_manifest_path: String,
    pub(super) orchestration_manifest_path: Option<String>,
    pub(super) panel_mismatch_diagnostics_path: Option<String>,
    pub(super) log_output_path: String,
    pub(super) raw_output_ids: Vec<String>,
    pub(super) parser_output_ids: Vec<String>,
    pub(super) declared_outputs: Vec<VcfImputationFamilyAdapterArtifact>,
    pub(super) command_steps: Vec<VcfImputationFamilyAdapterCommandStep>,
}
