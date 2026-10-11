//! Owner: bijux-dna-bench-vcf
//! plink adapter model contracts.
use super::Serialize;

pub const DEFAULT_VCF_PLINK_ADAPTER_PATH: &str =
    "artifacts/benchmark-smoke/readiness/adapters/plink.vcf.json";

pub const DEFAULT_VCF_PLINK2_ADAPTER_PATH: &str =
    "artifacts/benchmark-smoke/readiness/adapters/plink2.vcf.json";

pub const VCF_PLINK_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_plink_adapter.v1";

pub const VCF_PLINK2_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_plink2_adapter.v1";

pub(super) const GOVERNED_COHORT_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfPlinkFamilyAdapterArtifact {
    pub artifact_id: String,
    pub role: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfPlinkFamilyAdapterCommandStep {
    pub step_id: String,
    pub step_kind: String,
    pub argv: Vec<String>,
    pub declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfPlinkFamilyAdapterRow {
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
    pub normalized_metrics_artifact_id: String,
    pub normalized_metrics_path: String,
    pub stage_output_ids: Vec<String>,
    pub raw_output_ids: Vec<String>,
    pub parser_output_ids: Vec<String>,
    pub required_inputs: Vec<VcfPlinkFamilyAdapterArtifact>,
    pub declared_outputs: Vec<VcfPlinkFamilyAdapterArtifact>,
    pub command_steps: Vec<VcfPlinkFamilyAdapterCommandStep>,
    pub argv_validation_passed: bool,
    pub missing_input_probe_artifact_id: String,
    pub missing_input_expected_error_fragment: String,
    pub missing_input_observed_error: String,
    pub missing_input_test_passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfPlinkFamilyAdapterReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub tool_id: String,
    pub tool_status: String,
    pub output_path: String,
    pub row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub parser_output_row_count: usize,
    pub normalized_metrics_row_count: usize,
    pub raw_output_declared_row_count: usize,
    pub missing_input_test_passed_row_count: usize,
    pub rows: Vec<VcfPlinkFamilyAdapterRow>,
}

#[derive(Debug, Clone)]
pub(super) struct RegistryToolContract {
    pub(super) tool_id: String,
    pub(super) tool_status: String,
    pub(super) stage_ids: Vec<String>,
}
