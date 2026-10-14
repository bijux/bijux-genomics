use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
// Owner: bijux-dna-bench-workflows
// adna workflow wire and internal artifact contracts.
pub const DEFAULT_ADNA_MICRO_PIPELINE_PATH: &str =
    "artifacts/benchmark-smoke/micro/pipelines/adna/MICRO_ADNA_SUMMARY.json";
pub(crate) const GOVERNED_MICRO_STARTED_AT: &str = "1704067200";
pub(crate) const GOVERNED_MICRO_FINISHED_AT: &str = "1704067201";
pub(crate) const GOVERNED_MICRO_ELAPSED_SECONDS: f64 = 1.0;
pub(crate) const ADNA_MICRO_PIPELINE_SCHEMA_VERSION: &str =
    "bijux.bench.local_adna_micro_pipeline.v1";
pub(crate) const ADNA_MICRO_PIPELINE_COMMAND: &str =
    "bijux-dna bench local run-adna-micro-pipeline";
pub(crate) const ADNA_MICRO_PIPELINE_ID: &str = "adna-pseudohaploid-fastq-bam-vcf";
pub(crate) const ADNA_PIPELINE_SAMPLE_ID: &str = "adna-micro";
pub(crate) const ADNA_REFERENCE_FASTA_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta";
pub(crate) const ADNA_REFERENCE_CONTIG: &str = "chr1";
pub(crate) const FASTQ_VALIDATE_TOOL_ID: &str = "fastqvalidator";
pub(crate) const FASTQ_TRIM_TERMINAL_DAMAGE_TOOL_ID: &str = "cutadapt";
pub(crate) const FASTQ_REMOVE_DUPLICATES_TOOL_ID: &str = "bijux";
pub(crate) const BAM_ALIGN_TOOL_ID: &str = "bowtie2";
pub(crate) const BAM_VALIDATE_TOOL_ID: &str = "samtools";
pub(crate) const BAM_MAPPING_SUMMARY_TOOL_ID: &str = "samtools";
pub(crate) const BAM_COVERAGE_TOOL_ID: &str = "samtools";
pub(crate) const BAM_DAMAGE_TOOL_ID: &str = "mapdamage2";
pub(crate) const BAM_AUTHENTICITY_TOOL_ID: &str = "pmdtools";
pub(crate) const VCF_CALL_GL_TOOL_ID: &str = "angsd";
pub(crate) const VCF_CALL_PSEUDOHAPLOID_TOOL_ID: &str = "bcftools";
pub(crate) const VCF_GL_PROPAGATION_TOOL_ID: &str = "bcftools";
pub(crate) const VCF_DAMAGE_FILTER_TOOL_ID: &str = "bcftools";
pub(crate) const VCF_STATS_TOOL_ID: &str = "bcftools";

#[derive(Debug, Clone, Serialize)]
pub struct AdnaMicroPipelineReport {
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
    pub skipped_count: usize,
    pub passes_behavior_test: bool,
    pub rows: Vec<AdnaMicroPipelineRow>,
    pub handoffs: Vec<AdnaMicroPipelineHandoff>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdnaMicroPipelineRowStatus {
    Succeeded,
    Skipped,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdnaMicroPipelineRow {
    pub stage_id: String,
    pub domain: String,
    pub tool_id: String,
    pub execution_mode: String,
    pub status: AdnaMicroPipelineRowStatus,
    pub reason: String,
    pub evidence_path: Option<String>,
    pub parsed_schema_version: Option<String>,
    pub consumed_inputs: BTreeMap<String, String>,
    pub outputs: BTreeMap<String, String>,
    pub metrics: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
// Keep independent artifact observations as separate wire fields.
#[allow(clippy::struct_excessive_bools)]
pub struct AdnaMicroPipelineHandoff {
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
pub(crate) struct AdnaPseudohaploidCallReport {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) sample_id: String,
    pub(crate) output_vcf_path: String,
    pub(crate) output_tbi_path: String,
    pub(crate) variant_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) haploid_compatible: bool,
    pub(crate) gt_present: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdnaDamageFilterReport {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) filtered_vcf_path: String,
    pub(crate) filtered_tbi_path: String,
    pub(crate) summary_path: String,
    pub(crate) counts_path: String,
    pub(crate) retained_variants: u64,
    pub(crate) damage_ratio_filtered_variants: u64,
    pub(crate) terminal_damage_filtered_variants: u64,
    pub(crate) proxy_only_mode: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdnaStatsReport {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) stats_json_path: String,
    pub(crate) bcftools_stats_path: String,
    pub(crate) variant_count: u64,
    pub(crate) snp_count: u64,
    pub(crate) indel_count: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct FastqRecord {
    pub(crate) header: String,
    pub(crate) sequence: String,
    pub(crate) plus: String,
    pub(crate) quality: String,
}

#[derive(Debug, Clone)]
pub(crate) struct PipelineInputFixtures {
    pub(crate) raw_r1: PathBuf,
    pub(crate) raw_r2: PathBuf,
    pub(crate) placements: BTreeMap<String, SyntheticReadPlacement>,
    pub(crate) reference_name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct SyntheticReadPlacement {
    pub(crate) reference_name: String,
    pub(crate) position: u64,
}
