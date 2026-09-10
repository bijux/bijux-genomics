//! Owner: bijux-dna-bench-workflows
//! Core germline benchmark workflow composition.
use crate::execution::paths::path_relative_to_repo;
use crate::execution::variants::materialize_reference_fasta;
use crate::execution::variants::parse_output_sample_count;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_domain_bam::params::ReadGroupSpec;
use bijux_dna_domain_fastq::params::filter::FilterEffectiveParams;
use bijux_dna_domain_fastq::params::trim::TrimEffectiveParams;
use bijux_dna_domain_fastq::params::validate::PairSyncPolicy;
use bijux_dna_domain_fastq::params::validate::ValidationMode;
use bijux_dna_domain_fastq::validation_artifact_paths;
use bijux_dna_domain_fastq::FilterReadsReportV1;
use bijux_dna_domain_fastq::PairedMode;
use bijux_dna_domain_fastq::FILTER_READS_REPORT_SCHEMA_VERSION;
use bijux_dna_domain_fastq::VALIDATION_REPORT_SCHEMA_VERSION;
use bijux_dna_domain_vcf::params::VcfCallParams;
use bijux_dna_domain_vcf::params::VcfFilterParams;
use bijux_dna_domain_vcf::params::VcfStatsParams;
use bijux_dna_stages_vcf::metrics::parse_vcf_call_summary;
use bijux_dna_stages_vcf::pipeline::run_call_diploid_stage;
use bijux_dna_stages_vcf::pipeline::run_filter_stage_real;
use bijux_dna_stages_vcf::pipeline::run_qc_stage;
use bijux_dna_stages_vcf::pipeline::run_stats_stage_real;
use bijux_dna_stages_vcf::pipeline::QcStageParams;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use serde_json::json;
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
pub use model::{
    CoreGermlineMicroPipelineHandoff, CoreGermlineMicroPipelineReport,
    CoreGermlineMicroPipelineRow, DEFAULT_CORE_GERMLINE_MICRO_PIPELINE_PATH,
};
use model::{
    FastqLengthBin, FastqProfileStageReport, FastqRecord, FilterDecision, FilterDropReason,
    PipelineInputFixtures, SyntheticReadPlacement, VcfCallStageReport, VcfFilterStageReport,
    VcfQcStageReport, VcfStatsStageReport, BAM_ALIGN_TOOL_ID, BAM_COVERAGE_TOOL_ID,
    BAM_QC_PRE_TOOL_ID, BAM_VALIDATE_TOOL_ID, CORE_GERMLINE_MICRO_PIPELINE_COMMAND,
    CORE_GERMLINE_MICRO_PIPELINE_SCHEMA_VERSION, CORE_GERMLINE_PIPELINE_ID, FASTQ_FILTER_TOOL_ID,
    FASTQ_PROFILE_TOOL_ID, FASTQ_TRIM_TOOL_ID, FASTQ_VALIDATE_TOOL_ID,
    GOVERNED_MICRO_ELAPSED_SECONDS, GOVERNED_MICRO_FINISHED_AT, GOVERNED_MICRO_STARTED_AT,
    PIPELINE_SAMPLE_ID, REFERENCE_FASTA_PATH,
};

mod workflow;
pub use workflow::render_core_germline_micro_pipeline;
mod handoffs;
pub(crate) use handoffs::required_output;
mod stages;
use stages::{
    run_bam_align_stage, run_bam_coverage_stage, run_bam_qc_pre_stage, run_bam_validate_stage,
    run_fastq_filter_stage, run_fastq_profile_stage, run_fastq_trim_stage,
    run_fastq_validate_stage, run_vcf_call_stage, run_vcf_filter_stage, run_vcf_qc_stage,
    run_vcf_stats_stage,
};
mod observations;
use observations::{
    build_profile_report, combined_gc_percent, combined_mean_quality, length_histogram,
    read_fastq_records, required_tool_id, total_bases, write_fastq_records,
    write_filter_reads_report,
};
mod fixtures;
use fixtures::{materialize_pipeline_input_fastqs, write_mapped_sam_from_fastqs};
mod execution;
use execution::{read_json_document, timestamp_marker};

use crate::inputs::AlignmentFixtureOperations;
use crate::inputs::CoreGermlineWorkflowInputs;

use crate::inputs::AlignmentFixtureFormat;
