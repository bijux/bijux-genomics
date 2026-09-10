//! Owner: bijux-dna-bench-vcf
//! bcftools adapter model contracts.
use super::Serialize;

pub const DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH: &str =
    "artifacts/benchmark-smoke/readiness/adapters/bcftools.vcf.json";

pub const VCF_BCFTOOLS_ADAPTER_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_bcftools_adapter.v1";

pub(super) const GOVERNED_BCFTOOLS_TOOL_ID: &str = "bcftools";

pub(super) const GOVERNED_BAM_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam";

pub(super) const GOVERNED_BAM_INDEX_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam.bai";

pub(super) const GOVERNED_REFERENCE_FASTA_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta";

pub(super) const GOVERNED_RAW_SINGLE_SAMPLE_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf";

pub(super) const GOVERNED_FILTERED_SINGLE_SAMPLE_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_filtered_single_sample.vcf";

pub(super) const GOVERNED_MULTISAMPLE_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf";

pub(super) const GOVERNED_REFERENCE_PANEL_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_reference_panel.vcf";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfBcftoolsAdapterArtifact {
    pub artifact_id: String,
    pub role: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfBcftoolsAdapterCommandStep {
    pub step_id: String,
    pub step_kind: String,
    pub argv: Vec<String>,
    pub consumes_previous_stdout: bool,
    pub declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfBcftoolsAdapterRow {
    pub tool_id: String,
    pub stage_id: String,
    pub support_status: String,
    pub benchmark_status: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub output_root: String,
    pub stage_output_ids: Vec<String>,
    pub raw_output_ids: Vec<String>,
    pub parser_output_ids: Vec<String>,
    pub required_inputs: Vec<VcfBcftoolsAdapterArtifact>,
    pub declared_outputs: Vec<VcfBcftoolsAdapterArtifact>,
    pub command_steps: Vec<VcfBcftoolsAdapterCommandStep>,
    pub argv_validation_passed: bool,
    pub missing_input_probe_artifact_id: String,
    pub missing_input_expected_error_fragment: String,
    pub missing_input_observed_error: String,
    pub missing_input_test_passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfBcftoolsAdapterReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub tool_id: &'static str,
    pub output_path: String,
    pub row_count: usize,
    pub supported_row_count: usize,
    pub planned_row_count: usize,
    pub argv_valid_row_count: usize,
    pub missing_input_test_passed_row_count: usize,
    pub indexed_row_count: usize,
    pub rows: Vec<VcfBcftoolsAdapterRow>,
}
