//! Owner: bijux-dna-bench-vcf
//! descent adapter model contracts.
use super::Serialize;

pub const DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH: &str =
    "artifacts/benchmark-smoke/readiness/adapters/descent-family.vcf.json";

pub const VCF_DESCENT_FAMILY_ADAPTER_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_descent_family_adapter.v1";

pub(super) const GOVERNED_COHORT_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf";

pub(super) const GOVERNED_IBDHAP_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_ibdhap.vcf";

pub(super) const GOVERNED_IBDHAP_MAP_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/reference/vcf_mini_genetic_map.tsv";

pub(super) const GOVERNED_DESCENT_ROWS: [(&str, &str); 5] = [
    ("plink2", "vcf.roh"),
    ("germline", "vcf.ibd"),
    ("ibdseq", "vcf.ibd"),
    ("ibdhap", "vcf.ibd"),
    ("ibdne", "vcf.demography"),
];

pub(super) const GOVERNED_DEMOGRAPHY_SEGMENTS_CONTENT: &str =
    "sample_a\t0\tsample_b\t0\t1\t100\t100000\t8.5\nsample_a\t1\tsample_c\t0\t1\t120000\t200000\t7.0\n";

pub(super) const GOVERNED_DEMOGRAPHY_MAP_CONTENT: &str = "1\t.\t0.0\t1\n1\t.\t10.0\t250000\n";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfDescentFamilyAdapterArtifact {
    pub artifact_id: String,
    pub role: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfDescentFamilyAdapterCommandStep {
    pub step_id: String,
    pub step_kind: String,
    pub argv: Vec<String>,
    pub declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfDescentFamilyAdapterRow {
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
    pub input_vcf_path: Option<String>,
    pub input_ibd_segments_path: Option<String>,
    pub normalized_output_artifact_id: String,
    pub normalized_output_path: String,
    pub log_output_path: String,
    pub stage_output_ids: Vec<String>,
    pub raw_output_ids: Vec<String>,
    pub parser_output_ids: Vec<String>,
    pub required_inputs: Vec<VcfDescentFamilyAdapterArtifact>,
    pub declared_outputs: Vec<VcfDescentFamilyAdapterArtifact>,
    pub command_steps: Vec<VcfDescentFamilyAdapterCommandStep>,
    pub argv_validation_passed: bool,
    pub missing_input_probe_artifact_id: String,
    pub missing_input_expected_error_fragment: String,
    pub missing_input_observed_error: String,
    pub missing_input_test_passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfDescentFamilyAdapterReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub output_path: String,
    pub row_count: usize,
    pub tool_count: usize,
    pub benchmark_ready_row_count: usize,
    pub parser_output_row_count: usize,
    pub normalized_output_row_count: usize,
    pub missing_input_test_passed_row_count: usize,
    pub rows: Vec<VcfDescentFamilyAdapterRow>,
}

#[derive(Debug, Clone)]
pub(super) struct RegistryToolContract {
    pub(super) tool_id: String,
    pub(super) tool_status: String,
}
