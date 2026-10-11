use serde::Serialize;

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
// Owner: bijux-dna-bench-workflows
// core_germline workflow wire and internal artifact contracts.
pub const DEFAULT_CORE_GERMLINE_MICRO_PIPELINE_PATH: &str =
    "artifacts/benchmark-smoke/micro/pipelines/core-germline/MICRO_PIPELINE_SUMMARY.json";
pub(crate) const GOVERNED_MICRO_STARTED_AT: &str = "1704067200";
pub(crate) const GOVERNED_MICRO_FINISHED_AT: &str = "1704067201";
pub(crate) const GOVERNED_MICRO_ELAPSED_SECONDS: f64 = 1.0;
pub(crate) const CORE_GERMLINE_MICRO_PIPELINE_SCHEMA_VERSION: &str =
    "bijux.bench.local_core_germline_micro_pipeline.v1";
pub(crate) const CORE_GERMLINE_MICRO_PIPELINE_COMMAND: &str =
    "bijux-dna bench local run-core-germline-micro-pipeline";
pub(crate) const CORE_GERMLINE_PIPELINE_ID: &str = "core-germline-fastq-bam-vcf";
pub(crate) const PIPELINE_SAMPLE_ID: &str = "core-germline-micro";
pub(crate) const REFERENCE_FASTA_PATH: &str =
    "assets/reference/host/references/toy_host_reference.fasta";
pub(crate) const FASTQ_VALIDATE_TOOL_ID: &str = "fastqvalidator";
pub(crate) const FASTQ_PROFILE_TOOL_ID: &str = "seqkit_stats";
pub(crate) const FASTQ_TRIM_TOOL_ID: &str = "fastp";
pub(crate) const FASTQ_FILTER_TOOL_ID: &str = "fastp";
pub(crate) const BAM_ALIGN_TOOL_ID: &str = "bowtie2";
pub(crate) const BAM_VALIDATE_TOOL_ID: &str = "samtools";
pub(crate) const BAM_QC_PRE_TOOL_ID: &str = "samtools";
pub(crate) const BAM_COVERAGE_TOOL_ID: &str = "samtools";

#[derive(Debug, Clone, Serialize)]
pub struct CoreGermlineMicroPipelineReport {
    pub schema_version: &'static str,
    pub command: &'static str,
    pub output_path: String,
    pub pipeline_id: &'static str,
    pub sample_id: String,
    pub reference_fasta_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub stage_count: usize,
    pub handoff_count: usize,
    pub passes_behavior_test: bool,
    pub rows: Vec<CoreGermlineMicroPipelineRow>,
    pub handoffs: Vec<CoreGermlineMicroPipelineHandoff>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoreGermlineMicroPipelineRow {
    pub stage_id: String,
    pub domain: String,
    pub tool_id: String,
    pub execution_mode: String,
    pub evidence_path: String,
    pub parsed_schema_version: String,
    pub consumed_inputs: BTreeMap<String, String>,
    pub outputs: BTreeMap<String, String>,
    pub metrics: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Persisted independent artifact observations must retain their wire fields"
)]
pub struct CoreGermlineMicroPipelineHandoff {
    pub handoff_id: String,
    pub source_stage_id: String,
    pub target_stage_id: String,
    pub source_output_id: String,
    pub target_input_id: String,
    pub source_path: String,
    pub target_path: String,
    pub source_exists: bool,
    pub target_exists: bool,
    pub exact_path_match: bool,
    pub accepted: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqProfileStageReport {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) sample_id: String,
    pub(crate) input_r1: String,
    pub(crate) input_r2: Option<String>,
    pub(crate) reads_total: u64,
    pub(crate) bases_total: u64,
    pub(crate) mean_q: f64,
    pub(crate) gc_percent: f64,
    pub(crate) length_histogram: Vec<FastqLengthBin>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqLengthBin {
    pub(crate) length: u64,
    pub(crate) count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfCallStageReport {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) sample_id: String,
    pub(crate) consumed_coverage_report: String,
    pub(crate) output_vcf_path: String,
    pub(crate) output_tbi_path: String,
    pub(crate) call_metrics_path: String,
    pub(crate) call_manifest_path: String,
    pub(crate) reference_fasta_path: String,
    pub(crate) reference_fai_path: String,
    pub(crate) variant_count: u64,
    pub(crate) snp_count: u64,
    pub(crate) indel_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) coverage_gate_passed: bool,
    pub(crate) validation_checks: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfFilterStageReport {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) consumed_call_metrics_path: String,
    pub(crate) output_vcf_path: String,
    pub(crate) output_tbi_path: String,
    pub(crate) filter_breakdown_path: String,
    pub(crate) filter_breakdown_tsv_path: String,
    pub(crate) filter_explain_path: String,
    pub(crate) pass_variant_count: u64,
    pub(crate) tagged_variant_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) validation_checks: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfStatsStageReport {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) consumed_filter_report_path: String,
    pub(crate) stats_json_path: String,
    pub(crate) bcftools_stats_path: String,
    pub(crate) variant_count: u64,
    pub(crate) snp_count: u64,
    pub(crate) indel_count: u64,
    pub(crate) ti_tv: Option<f64>,
    pub(crate) sample_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfQcStageReport {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) consumed_stats_json_path: String,
    pub(crate) qc_summary_path: String,
    pub(crate) qc_tables_path: String,
    pub(crate) imputation_qc_path: String,
    pub(crate) warnings_path: String,
    pub(crate) qc_histograms_path: String,
    pub(crate) sample_missingness_row_count: usize,
    pub(crate) variant_missingness_row_count: usize,
    pub(crate) excluded_sample_count: usize,
    pub(crate) excluded_variant_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct FastqRecord {
    pub(crate) header: String,
    pub(crate) sequence: String,
    pub(crate) plus: String,
    pub(crate) quality: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FilterDropReason {
    TooManyN,
    LowComplexity,
}

#[derive(Debug, Clone)]
pub(crate) struct FilterDecision {
    pub(crate) record: FastqRecord,
    pub(crate) drop_reason: Option<FilterDropReason>,
}

#[derive(Debug, Clone)]
pub(crate) struct PipelineInputFixtures {
    pub(crate) raw_r1: PathBuf,
    pub(crate) raw_r2: PathBuf,
    pub(crate) placements: BTreeMap<String, SyntheticReadPlacement>,
}

#[derive(Debug, Clone)]
pub(crate) struct SyntheticReadPlacement {
    pub(crate) reference_name: String,
    pub(crate) position: u64,
}
