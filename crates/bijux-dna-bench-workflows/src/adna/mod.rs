//! Owner: bijux-dna-bench-workflows
//! adna benchmark workflow composition.
use crate::execution::paths::path_relative_to_repo;
use crate::execution::variants::materialize_reference_fasta;
use crate::execution::variants::parse_output_sample_count;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_domain_bam::metrics::DamageMetricsV1;
use bijux_dna_domain_bam::params::ReadGroupSpec;
use bijux_dna_domain_fastq::params::remove_duplicates::DedupMode;
use bijux_dna_domain_fastq::params::remove_duplicates::RemoveDuplicatesEffectiveParams;
use bijux_dna_domain_fastq::params::remove_duplicates::REMOVE_DUPLICATES_SCHEMA_VERSION;
use bijux_dna_domain_fastq::params::trim::TerminalDamageExecutionPolicy;
use bijux_dna_domain_fastq::params::validate::PairSyncPolicy;
use bijux_dna_domain_fastq::params::validate::ValidationMode;
use bijux_dna_domain_fastq::params::DamageMode;
use bijux_dna_domain_fastq::params::PairedMode;
use bijux_dna_domain_fastq::stages::remove_duplicates;
use bijux_dna_domain_fastq::validation_artifact_paths;
use bijux_dna_domain_fastq::TerminalDamageReportV1;
use bijux_dna_domain_fastq::TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION;
use bijux_dna_domain_fastq::VALIDATION_REPORT_SCHEMA_VERSION;
use bijux_dna_stages_vcf::pipeline::run_call_pseudohaploid_stage;
use bijux_dna_stages_vcf::pipeline::run_damage_filter_stage;
use bijux_dna_stages_vcf::pipeline::run_stats_stage_real;
use bijux_dna_stages_vcf::pipeline::DamageFilterStageParams;
use bijux_dna_stages_vcf::pipeline::DamageUdgRegime;
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
mod model;
use model::{
    AdnaDamageFilterReport, AdnaPseudohaploidCallReport, AdnaStatsReport, FastqRecord,
    PipelineInputFixtures, SyntheticReadPlacement, ADNA_MICRO_PIPELINE_COMMAND,
    ADNA_MICRO_PIPELINE_ID, ADNA_MICRO_PIPELINE_SCHEMA_VERSION, ADNA_PIPELINE_SAMPLE_ID,
    ADNA_REFERENCE_CONTIG, ADNA_REFERENCE_FASTA_PATH, BAM_ALIGN_TOOL_ID, BAM_AUTHENTICITY_TOOL_ID,
    BAM_COVERAGE_TOOL_ID, BAM_DAMAGE_TOOL_ID, BAM_MAPPING_SUMMARY_TOOL_ID, BAM_VALIDATE_TOOL_ID,
    FASTQ_REMOVE_DUPLICATES_TOOL_ID, FASTQ_TRIM_TERMINAL_DAMAGE_TOOL_ID, FASTQ_VALIDATE_TOOL_ID,
    GOVERNED_MICRO_ELAPSED_SECONDS, GOVERNED_MICRO_FINISHED_AT, GOVERNED_MICRO_STARTED_AT,
    VCF_CALL_GL_TOOL_ID, VCF_CALL_PSEUDOHAPLOID_TOOL_ID, VCF_DAMAGE_FILTER_TOOL_ID,
    VCF_GL_PROPAGATION_TOOL_ID, VCF_STATS_TOOL_ID,
};
pub use model::{
    AdnaMicroPipelineHandoff, AdnaMicroPipelineReport, AdnaMicroPipelineRow,
    AdnaMicroPipelineRowStatus, DEFAULT_ADNA_MICRO_PIPELINE_PATH,
};

mod workflow;
pub use workflow::render_adna_micro_pipeline;
mod handoffs;
pub(crate) use handoffs::{required_output, skipped_row, succeeded_row};
mod stages;
use stages::{
    run_bam_align_stage, run_bam_authenticity_stage, run_bam_coverage_stage, run_bam_damage_stage,
    run_bam_mapping_summary_stage, run_bam_validate_stage, run_fastq_remove_duplicates_stage,
    run_fastq_trim_terminal_damage_stage, run_fastq_validate_stage,
    run_vcf_call_pseudohaploid_stage_row, run_vcf_damage_filter_stage_row, run_vcf_stats_stage_row,
};
mod fixtures;
use fixtures::{
    materialize_adna_pipeline_input_fastqs, read_named_reference_contig,
    write_mapped_sam_from_fastqs,
};
mod observations;
use observations::{
    count_json_u64, count_vcf_variants, derive_damage_metrics, first_base_histogram,
    last_base_histogram, mean_quality, read_fastq_records, summarize_haploid_vcf,
    terminal_transition_asymmetry, terminal_transition_asymmetry_one_end, total_fastq_bases,
    trim_terminal_damage_record, write_fastq_records,
};
mod execution;
use execution::timestamp_marker;

use crate::inputs::AlignmentFixtureOperations;

use crate::inputs::AlignmentFixtureFormat;
