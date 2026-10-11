//! Owner: bijux-dna-bench-vcf
//! phasing adapter model contracts.
use super::Serialize;

pub const DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH: &str =
    "artifacts/benchmark-smoke/readiness/adapters/shapeit5.vcf.json";

pub const DEFAULT_VCF_SHAPEIT_ADAPTER_PATH: &str =
    "artifacts/benchmark-smoke/readiness/adapters/shapeit.vcf.json";

pub const DEFAULT_VCF_EAGLE_ADAPTER_PATH: &str =
    "artifacts/benchmark-smoke/readiness/adapters/eagle.vcf.json";

pub const DEFAULT_VCF_BEAGLE_ADAPTER_PATH: &str =
    "artifacts/benchmark-smoke/readiness/adapters/beagle.vcf.json";

pub const VCF_SHAPEIT5_ADAPTER_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_shapeit5_adapter.v1";

pub const VCF_SHAPEIT_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_shapeit_adapter.v1";

pub const VCF_EAGLE_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_eagle_adapter.v1";

pub const VCF_BEAGLE_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_beagle_adapter.v1";

pub(super) const GOVERNED_COHORT_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfPhasingFamilyAdapterArtifact {
    pub artifact_id: String,
    pub role: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfPhasingFamilyAdapterCommandStep {
    pub step_id: String,
    pub step_kind: String,
    pub argv: Vec<String>,
    pub declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfPhasingFamilyAdapterRow {
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
    pub input_vcf_path: String,
    pub panel_vcf_path: String,
    pub genetic_map_path: String,
    pub phased_vcf_path: String,
    pub phased_vcf_tbi_path: String,
    pub phase_block_stats_path: String,
    pub switch_error_proxy_path: String,
    pub phasing_qc_path: String,
    pub phasing_manifest_path: String,
    pub logs_path: String,
    pub stage_output_ids: Vec<String>,
    pub raw_output_ids: Vec<String>,
    pub parser_output_ids: Vec<String>,
    pub required_inputs: Vec<VcfPhasingFamilyAdapterArtifact>,
    pub declared_outputs: Vec<VcfPhasingFamilyAdapterArtifact>,
    pub command_steps: Vec<VcfPhasingFamilyAdapterCommandStep>,
    pub argv_validation_passed: bool,
    pub missing_input_probe_artifact_id: String,
    pub missing_input_expected_error_fragment: String,
    pub missing_input_observed_error: String,
    pub missing_input_test_passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfPhasingFamilyAdapterReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub tool_id: String,
    pub tool_status: String,
    pub output_path: String,
    pub row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub parser_output_row_count: usize,
    pub indexed_row_count: usize,
    pub missing_input_test_passed_row_count: usize,
    pub rows: Vec<VcfPhasingFamilyAdapterRow>,
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
    pub(super) genetic_map_path: String,
}

pub(super) struct StageContract {
    pub(super) command_contract_source: String,
    pub(super) phased_vcf_path: String,
    pub(super) phased_vcf_tbi_path: String,
    pub(super) phase_block_stats_path: String,
    pub(super) switch_error_proxy_path: String,
    pub(super) phasing_qc_path: String,
    pub(super) phasing_manifest_path: String,
    pub(super) logs_path: String,
    pub(super) raw_output_ids: Vec<String>,
    pub(super) parser_output_ids: Vec<String>,
    pub(super) declared_outputs: Vec<VcfPhasingFamilyAdapterArtifact>,
    pub(super) command_steps: Vec<VcfPhasingFamilyAdapterCommandStep>,
}
