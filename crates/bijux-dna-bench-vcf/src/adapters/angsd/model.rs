//! Owner: bijux-dna-bench-vcf
//! angsd adapter model contracts.
use super::Serialize;

pub const DEFAULT_VCF_ANGSD_ADAPTER_PATH: &str =
    "artifacts/benchmark-smoke/readiness/adapters/angsd.vcf.json";

pub const VCF_ANGSD_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_angsd_adapter.v1";

pub(super) const GOVERNED_ANGSD_TOOL_ID: &str = "angsd";

pub(super) const GOVERNED_ANGSD_TOOL_STATUS: &str = "external_artifact";

pub(super) const GOVERNED_CORPUS_ID: &str = "vcf_production_regression";

pub(super) const GOVERNED_BAM_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam";

pub(super) const GOVERNED_REFERENCE_FASTA_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta";

pub(super) const GOVERNED_SITES_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfAngsdAdapterArtifact {
    pub artifact_id: String,
    pub role: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfAngsdAdapterCommandStep {
    pub step_id: String,
    pub step_kind: String,
    pub argv: Vec<String>,
    pub declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfAngsdAdapterRow {
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
    pub bam_member_paths: Vec<String>,
    pub bam_list_path: Option<String>,
    pub reference_path: Option<String>,
    pub sites_path: Option<String>,
    pub likelihood_model: Option<String>,
    pub stage_output_ids: Vec<String>,
    pub raw_output_ids: Vec<String>,
    pub parser_output_ids: Vec<String>,
    pub required_inputs: Vec<VcfAngsdAdapterArtifact>,
    pub declared_outputs: Vec<VcfAngsdAdapterArtifact>,
    pub command_steps: Vec<VcfAngsdAdapterCommandStep>,
    pub argv_validation_passed: bool,
    pub missing_input_probe_artifact_id: String,
    pub missing_input_expected_error_fragment: String,
    pub missing_input_observed_error: String,
    pub missing_input_test_passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfAngsdAdapterReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub tool_id: &'static str,
    pub tool_status: &'static str,
    pub output_path: String,
    pub row_count: usize,
    pub supported_stage_row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub argv_valid_row_count: usize,
    pub missing_input_test_passed_row_count: usize,
    pub bam_list_row_count: usize,
    pub parser_output_row_count: usize,
    pub rows: Vec<VcfAngsdAdapterRow>,
}
