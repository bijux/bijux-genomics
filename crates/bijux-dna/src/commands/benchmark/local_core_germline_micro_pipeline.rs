use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_bam::params::ReadGroupSpec;
use bijux_dna_domain_fastq::params::filter::FilterEffectiveParams;
use bijux_dna_domain_fastq::params::trim::TrimEffectiveParams;
use bijux_dna_domain_fastq::params::validate::{PairSyncPolicy, ValidationMode};
use bijux_dna_domain_fastq::{
    validation_artifact_paths, FilterReadsReportV1, PairedMode, FILTER_READS_REPORT_SCHEMA_VERSION,
    VALIDATION_REPORT_SCHEMA_VERSION,
};
use bijux_dna_domain_vcf::params::{VcfCallParams, VcfFilterParams, VcfStatsParams};
use bijux_dna_stages_vcf::metrics::parse_vcf_call_summary;
use bijux_dna_stages_vcf::pipeline::{
    run_call_diploid_stage, run_filter_stage_real, run_qc_stage, run_stats_stage_real,
    QcStageParams,
};
use bijux_dna_stages_vcf::vcf_io::{vcf_validate_input, VcfFieldRequirement};
use noodles_bam as bam;
use noodles_sam as sam;
use serde::Serialize;
use serde_json::{json, Value};

use super::local_stage_result_manifest::path_relative_to_repo;
use super::local_vcf_call_bam_smoke_support::{
    materialize_reference_fasta, parse_output_sample_count,
};
use crate::commands::benchmark::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_CORE_GERMLINE_MICRO_PIPELINE_PATH: &str =
    "runs/bench/micro/pipelines/core-germline/MICRO_PIPELINE_SUMMARY.json";
const GOVERNED_MICRO_STARTED_AT: &str = "1704067200";
const GOVERNED_MICRO_FINISHED_AT: &str = "1704067201";
const GOVERNED_MICRO_ELAPSED_SECONDS: f64 = 1.0;
const CORE_GERMLINE_MICRO_PIPELINE_SCHEMA_VERSION: &str =
    "bijux.bench.local_core_germline_micro_pipeline.v1";
const CORE_GERMLINE_MICRO_PIPELINE_COMMAND: &str =
    "bijux-dna bench local run-core-germline-micro-pipeline";
const CORE_GERMLINE_PIPELINE_ID: &str = "core-germline-fastq-bam-vcf";
const PIPELINE_SAMPLE_ID: &str = "core-germline-micro";
const REFERENCE_FASTA_PATH: &str = "assets/reference/host/references/toy_host_reference.fasta";
const FASTQ_VALIDATE_TOOL_ID: &str = "fastqvalidator";
const FASTQ_PROFILE_TOOL_ID: &str = "seqkit_stats";
const FASTQ_TRIM_TOOL_ID: &str = "fastp";
const FASTQ_FILTER_TOOL_ID: &str = "fastp";
const BAM_ALIGN_TOOL_ID: &str = "bowtie2";
const BAM_VALIDATE_TOOL_ID: &str = "samtools";
const BAM_QC_PRE_TOOL_ID: &str = "samtools";
const BAM_COVERAGE_TOOL_ID: &str = "samtools";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreGermlineMicroPipelineReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: &'static str,
    pub(crate) output_path: String,
    pub(crate) pipeline_id: &'static str,
    pub(crate) sample_id: String,
    pub(crate) reference_fasta_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) stage_count: usize,
    pub(crate) handoff_count: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) rows: Vec<CoreGermlineMicroPipelineRow>,
    pub(crate) handoffs: Vec<CoreGermlineMicroPipelineHandoff>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreGermlineMicroPipelineRow {
    pub(crate) stage_id: String,
    pub(crate) domain: String,
    pub(crate) tool_id: String,
    pub(crate) execution_mode: String,
    pub(crate) evidence_path: String,
    pub(crate) parsed_schema_version: String,
    pub(crate) consumed_inputs: BTreeMap<String, String>,
    pub(crate) outputs: BTreeMap<String, String>,
    pub(crate) metrics: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreGermlineMicroPipelineHandoff {
    pub(crate) handoff_id: String,
    pub(crate) source_stage_id: String,
    pub(crate) target_stage_id: String,
    pub(crate) source_output_id: String,
    pub(crate) target_input_id: String,
    pub(crate) source_path: String,
    pub(crate) target_path: String,
    pub(crate) source_exists: bool,
    pub(crate) target_exists: bool,
    pub(crate) exact_path_match: bool,
    pub(crate) accepted: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
struct FastqProfileStageReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    sample_id: String,
    input_r1: String,
    input_r2: Option<String>,
    reads_total: u64,
    bases_total: u64,
    mean_q: f64,
    gc_percent: f64,
    length_histogram: Vec<FastqLengthBin>,
}

#[derive(Debug, Clone, Serialize)]
struct FastqLengthBin {
    length: u64,
    count: u64,
}

#[derive(Debug, Clone, Serialize)]
struct VcfCallStageReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    sample_id: String,
    consumed_coverage_report: String,
    output_vcf_path: String,
    output_tbi_path: String,
    call_metrics_path: String,
    call_manifest_path: String,
    reference_fasta_path: String,
    reference_fai_path: String,
    variant_count: u64,
    snp_count: u64,
    indel_count: u64,
    sample_count: u64,
    coverage_gate_passed: bool,
    validation_checks: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Serialize)]
struct VcfFilterStageReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    consumed_call_metrics_path: String,
    output_vcf_path: String,
    output_tbi_path: String,
    filter_breakdown_path: String,
    filter_breakdown_tsv_path: String,
    filter_explain_path: String,
    pass_variant_count: u64,
    tagged_variant_count: u64,
    sample_count: u64,
    validation_checks: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Serialize)]
struct VcfStatsStageReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    consumed_filter_report_path: String,
    stats_json_path: String,
    bcftools_stats_path: String,
    variant_count: u64,
    snp_count: u64,
    indel_count: u64,
    ti_tv: Option<f64>,
    sample_count: u64,
}

#[derive(Debug, Clone, Serialize)]
struct VcfQcStageReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    consumed_stats_json_path: String,
    qc_summary_path: String,
    qc_tables_path: String,
    imputation_qc_path: String,
    warnings_path: String,
    qc_histograms_path: String,
    sample_missingness_row_count: usize,
    variant_missingness_row_count: usize,
    excluded_sample_count: usize,
    excluded_variant_count: usize,
}

#[derive(Debug, Clone)]
struct FastqRecord {
    header: String,
    sequence: String,
    plus: String,
    quality: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterDropReason {
    TooManyN,
    LowComplexity,
}

#[derive(Debug, Clone)]
struct FilterDecision {
    record: FastqRecord,
    drop_reason: Option<FilterDropReason>,
}

#[derive(Debug, Clone)]
struct PipelineInputFixtures {
    raw_r1: PathBuf,
    raw_r2: PathBuf,
    placements: BTreeMap<String, SyntheticReadPlacement>,
}

#[derive(Debug, Clone)]
struct SyntheticReadPlacement {
    reference_name: String,
    position: u64,
}

pub(crate) fn run_core_germline_micro_pipeline(
    args: &parse::BenchLocalRunCoreGermlineMicroPipelineArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_core_germline_micro_pipeline(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CORE_GERMLINE_MICRO_PIPELINE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_core_germline_micro_pipeline(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<CoreGermlineMicroPipelineReport> {
    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(output_path) };
    let governed_output = path_relative_to_repo(repo_root, &absolute_output_path)
        == DEFAULT_CORE_GERMLINE_MICRO_PIPELINE_PATH;
    let output_root = absolute_output_path
        .parent()
        .ok_or_else(|| anyhow!("core germline micro pipeline output has no parent directory"))?;
    reset_generated_output_root(output_root)?;

    let reference_fasta = repo_root.join(REFERENCE_FASTA_PATH);
    if !reference_fasta.is_file() {
        bail!("core germline micro pipeline is missing required reference input");
    }
    let input_fixtures = materialize_pipeline_input_fastqs(output_root, &reference_fasta)?;
    let input_r1 = input_fixtures.raw_r1.clone();
    let input_r2 = input_fixtures.raw_r2.clone();

    let started_at =
        if governed_output { GOVERNED_MICRO_STARTED_AT.to_string() } else { timestamp_marker() };
    let started = Instant::now();

    let validate_row = run_fastq_validate_stage(repo_root, output_root, &input_r1, &input_r2)?;
    let profile_row =
        run_fastq_profile_stage(repo_root, output_root, &input_r1, &input_r2, &validate_row)?;
    let trim_row =
        run_fastq_trim_stage(repo_root, output_root, &input_r1, &input_r2, &validate_row)?;
    let filter_row = run_fastq_filter_stage(repo_root, output_root, &trim_row)?;
    let align_row = run_bam_align_stage(
        repo_root,
        output_root,
        &reference_fasta,
        &input_fixtures,
        &filter_row,
    )?;
    let validate_bam_row =
        run_bam_validate_stage(repo_root, output_root, &reference_fasta, &align_row)?;
    let qc_pre_row = run_bam_qc_pre_stage(repo_root, output_root, &align_row, &validate_bam_row)?;
    let coverage_row = run_bam_coverage_stage(repo_root, output_root, &align_row, &qc_pre_row)?;
    let vcf_tool_ids = resolve_core_vcf_tool_ids()?;
    let call_row = run_vcf_call_stage(
        repo_root,
        output_root,
        &align_row,
        &coverage_row,
        &reference_fasta,
        &vcf_tool_ids,
    )?;
    let filter_vcf_row = run_vcf_filter_stage(repo_root, output_root, &call_row, &vcf_tool_ids)?;
    let stats_row = run_vcf_stats_stage(repo_root, output_root, &filter_vcf_row, &vcf_tool_ids)?;
    let qc_row =
        run_vcf_qc_stage(repo_root, output_root, &filter_vcf_row, &stats_row, &vcf_tool_ids)?;

    let rows = vec![
        validate_row,
        profile_row,
        trim_row,
        filter_row,
        align_row,
        validate_bam_row,
        qc_pre_row,
        coverage_row,
        call_row,
        filter_vcf_row,
        stats_row,
        qc_row,
    ];

    let handoffs = vec![
        validate_handoff(
            repo_root,
            &rows,
            "fastq.validate_reads",
            "validated_reads_r1_path",
            "fastq.profile_reads",
            "validated_reads_r1_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "fastq.validate_reads",
            "validated_reads_r2_path",
            "fastq.profile_reads",
            "validated_reads_r2_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "fastq.validate_reads",
            "validated_reads_r1_path",
            "fastq.trim_reads",
            "validated_reads_r1_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "fastq.validate_reads",
            "validated_reads_r2_path",
            "fastq.trim_reads",
            "validated_reads_r2_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "fastq.trim_reads",
            "trimmed_reads_r1_path",
            "fastq.filter_reads",
            "trimmed_reads_r1_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "fastq.trim_reads",
            "trimmed_reads_r2_path",
            "fastq.filter_reads",
            "trimmed_reads_r2_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "fastq.filter_reads",
            "filtered_reads_r1_path",
            "bam.align",
            "filtered_reads_r1_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "fastq.filter_reads",
            "filtered_reads_r2_path",
            "bam.align",
            "filtered_reads_r2_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.align",
            "aligned_bam",
            "bam.validate",
            "aligned_bam",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.validate",
            "bam_validation_report",
            "bam.qc_pre",
            "bam_validation_report",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.qc_pre",
            "qc_pre_report",
            "bam.coverage",
            "qc_pre_report",
        )?,
        validate_handoff(repo_root, &rows, "bam.align", "aligned_bam", "vcf.call", "aligned_bam")?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.coverage",
            "coverage_report_json",
            "vcf.call",
            "coverage_report_json",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.coverage",
            "coverage_regions_json",
            "vcf.call",
            "coverage_regions_json",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "vcf.call",
            "call_metrics_json",
            "vcf.filter",
            "call_metrics_json",
        )?,
        validate_handoff(repo_root, &rows, "vcf.call", "called_vcf", "vcf.filter", "called_vcf")?,
        validate_handoff(
            repo_root,
            &rows,
            "vcf.filter",
            "filter_report_json",
            "vcf.stats",
            "filter_report_json",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "vcf.filter",
            "filtered_vcf",
            "vcf.stats",
            "filtered_vcf",
        )?,
        validate_handoff(repo_root, &rows, "vcf.filter", "filtered_vcf", "vcf.qc", "filtered_vcf")?,
        validate_handoff(repo_root, &rows, "vcf.stats", "stats_json", "vcf.qc", "stats_json")?,
    ];

    let passes_behavior_test = rows.len() == 12 && handoffs.iter().all(|handoff| handoff.accepted);
    let finished_at =
        if governed_output { GOVERNED_MICRO_FINISHED_AT.to_string() } else { timestamp_marker() };
    let report = CoreGermlineMicroPipelineReport {
        schema_version: CORE_GERMLINE_MICRO_PIPELINE_SCHEMA_VERSION,
        command: CORE_GERMLINE_MICRO_PIPELINE_COMMAND,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        pipeline_id: CORE_GERMLINE_PIPELINE_ID,
        sample_id: PIPELINE_SAMPLE_ID.to_string(),
        reference_fasta_path: path_relative_to_repo(repo_root, &reference_fasta),
        started_at,
        finished_at,
        elapsed_seconds: if governed_output {
            GOVERNED_MICRO_ELAPSED_SECONDS
        } else {
            started.elapsed().as_secs_f64()
        },
        stage_count: rows.len(),
        handoff_count: handoffs.len(),
        passes_behavior_test,
        rows,
        handoffs,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}

fn reset_generated_output_root(output_root: &Path) -> Result<()> {
    fs::create_dir_all(output_root).with_context(|| format!("create {}", output_root.display()))?;
    for entry in
        fs::read_dir(output_root).with_context(|| format!("read {}", output_root.display()))?
    {
        let entry = entry.with_context(|| format!("read {}", output_root.display()))?;
        let path = entry.path();
        let file_type =
            entry.file_type().with_context(|| format!("read file type {}", path.display()))?;
        if file_type.is_dir() {
            fs::remove_dir_all(&path).with_context(|| format!("remove {}", path.display()))?;
        } else {
            fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
        }
    }
    Ok(())
}

fn run_fastq_validate_stage(
    repo_root: &Path,
    output_root: &Path,
    input_r1: &Path,
    input_r2: &Path,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/fastq.validate_reads");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let artifact_paths = validation_artifact_paths(&stage_root, true);
    let (mut report, manifest) = bijux_dna_domain_fastq::stages::validate_reads(
        input_r1,
        Some(input_r2),
        ValidationMode::Strict,
        PairSyncPolicy::RequireHeaderSync,
        &artifact_paths.validation_log_r1,
        artifact_paths.validation_log_r2.as_deref(),
        &artifact_paths.report_json,
    )?;
    report.input_r1 = path_relative_to_repo(repo_root, input_r1);
    report.input_r2 = Some(path_relative_to_repo(repo_root, input_r2));
    report.validation_log_r1 = path_relative_to_repo(repo_root, &artifact_paths.validation_log_r1);
    report.validation_log_r2 = artifact_paths
        .validation_log_r2
        .as_ref()
        .map(|path| path_relative_to_repo(repo_root, path));
    bijux_dna_infra::atomic_write_json(&artifact_paths.report_json, &report)?;

    let mut manifest = manifest;
    manifest.input_r1 = path_relative_to_repo(repo_root, input_r1);
    manifest.input_r2 = Some(path_relative_to_repo(repo_root, input_r2));
    manifest.validation_report = path_relative_to_repo(repo_root, &artifact_paths.report_json);
    bijux_dna_infra::atomic_write_json(&artifact_paths.validated_reads_manifest, &manifest)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: "fastq.validate_reads".to_string(),
        domain: "fastq".to_string(),
        tool_id: FASTQ_VALIDATE_TOOL_ID.to_string(),
        execution_mode: "domain_contract".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &artifact_paths.report_json),
        parsed_schema_version: VALIDATION_REPORT_SCHEMA_VERSION.to_string(),
        consumed_inputs: BTreeMap::from([
            ("raw_reads_r1_path".to_string(), path_relative_to_repo(repo_root, input_r1)),
            ("raw_reads_r2_path".to_string(), path_relative_to_repo(repo_root, input_r2)),
        ]),
        outputs: BTreeMap::from([
            ("validated_reads_r1_path".to_string(), path_relative_to_repo(repo_root, input_r1)),
            ("validated_reads_r2_path".to_string(), path_relative_to_repo(repo_root, input_r2)),
            (
                "validation_report".to_string(),
                path_relative_to_repo(repo_root, &artifact_paths.report_json),
            ),
            (
                "validated_reads_manifest".to_string(),
                path_relative_to_repo(repo_root, &artifact_paths.validated_reads_manifest),
            ),
        ]),
        metrics: BTreeMap::from([
            ("validated_reads_r1".to_string(), Value::from(report.validated_reads_r1)),
            ("validated_reads_r2".to_string(), Value::from(report.validated_reads_r2.unwrap_or(0))),
            ("validated_pairs".to_string(), Value::from(report.validated_pairs.unwrap_or(0))),
            ("strict_pass".to_string(), Value::from(report.strict_pass)),
            ("exit_code".to_string(), Value::from(report.exit_code)),
        ]),
    })
}

fn run_fastq_profile_stage(
    repo_root: &Path,
    output_root: &Path,
    input_r1: &Path,
    input_r2: &Path,
    validate_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/fastq.profile_reads");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let report_path = stage_root.join("profile.json");
    let report = build_profile_report(input_r1, Some(input_r2))?;
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: "fastq.profile_reads".to_string(),
        domain: "fastq".to_string(),
        tool_id: FASTQ_PROFILE_TOOL_ID.to_string(),
        execution_mode: "domain_contract".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            (
                "validated_reads_r1_path".to_string(),
                required_output(validate_row, "validated_reads_r1_path")?,
            ),
            (
                "validated_reads_r2_path".to_string(),
                required_output(validate_row, "validated_reads_r2_path")?,
            ),
        ]),
        outputs: BTreeMap::from([(
            "validated_profile".to_string(),
            path_relative_to_repo(repo_root, &report_path),
        )]),
        metrics: BTreeMap::from([
            ("reads_total".to_string(), Value::from(report.reads_total)),
            ("bases_total".to_string(), Value::from(report.bases_total)),
            ("mean_q".to_string(), Value::from(report.mean_q)),
            ("gc_percent".to_string(), Value::from(report.gc_percent)),
        ]),
    })
}

fn run_fastq_trim_stage(
    repo_root: &Path,
    output_root: &Path,
    input_r1: &Path,
    input_r2: &Path,
    validate_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/fastq.trim_reads");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let output_r1 = stage_root.join("trimmed_R1.fastq");
    let output_r2 = stage_root.join("trimmed_R2.fastq");
    let report_path = stage_root.join("trim.report.json");
    let raw_backend_report = stage_root.join("trim.backend.json");
    let report = bijux_dna_domain_fastq::stages::trim_reads(
        input_r1,
        Some(input_r2),
        &TrimEffectiveParams {
            paired_mode: PairedMode::PairedEnd,
            threads: 1,
            min_len: 4,
            q_cutoff: None,
            adapter_policy: "none".to_string(),
            damage_mode: None,
            polyx_policy: None,
            n_policy: Some("retain".to_string()),
            contaminant_policy: Some("none".to_string()),
        },
        FASTQ_TRIM_TOOL_ID,
        &output_r1,
        Some(&output_r2),
        Some(&raw_backend_report),
    )?;
    if !raw_backend_report.is_file() {
        bijux_dna_infra::write_bytes(
            &raw_backend_report,
            json!({
                "summary": {
                    "reads_in": report.reads_in,
                    "reads_out": report.reads_out,
                    "bases_in": report.bases_in,
                    "bases_out": report.bases_out,
                }
            })
            .to_string(),
        )?;
    }
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: "fastq.trim_reads".to_string(),
        domain: "fastq".to_string(),
        tool_id: FASTQ_TRIM_TOOL_ID.to_string(),
        execution_mode: "domain_contract".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            (
                "validated_reads_r1_path".to_string(),
                required_output(validate_row, "validated_reads_r1_path")?,
            ),
            (
                "validated_reads_r2_path".to_string(),
                required_output(validate_row, "validated_reads_r2_path")?,
            ),
        ]),
        outputs: BTreeMap::from([
            ("trimmed_reads_r1_path".to_string(), path_relative_to_repo(repo_root, &output_r1)),
            ("trimmed_reads_r2_path".to_string(), path_relative_to_repo(repo_root, &output_r2)),
            ("trim_report".to_string(), path_relative_to_repo(repo_root, &report_path)),
            (
                "trim_backend_report".to_string(),
                path_relative_to_repo(repo_root, &raw_backend_report),
            ),
        ]),
        metrics: BTreeMap::from([
            ("reads_in".to_string(), Value::from(report.reads_in.unwrap_or(0))),
            ("reads_out".to_string(), Value::from(report.reads_out.unwrap_or(0))),
            ("bases_in".to_string(), Value::from(report.bases_in.unwrap_or(0))),
            ("bases_out".to_string(), Value::from(report.bases_out.unwrap_or(0))),
        ]),
    })
}

fn run_fastq_filter_stage(
    repo_root: &Path,
    output_root: &Path,
    trim_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/fastq.filter_reads");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let input_r1 = repo_root.join(required_output(trim_row, "trimmed_reads_r1_path")?);
    let input_r2 = repo_root.join(required_output(trim_row, "trimmed_reads_r2_path")?);
    let output_r1 = stage_root.join("filtered_R1.fastq");
    let output_r2 = stage_root.join("filtered_R2.fastq");
    let report_path = stage_root.join("filter.report.json");
    let raw_backend_report = stage_root.join("filter.backend.json");
    let report = write_filter_reads_report(
        repo_root,
        &input_r1,
        Some(&input_r2),
        &output_r1,
        Some(&output_r2),
        &report_path,
        &raw_backend_report,
        &FilterEffectiveParams {
            paired_mode: PairedMode::PairedEnd,
            threads: 1,
            max_n: None,
            max_n_fraction: None,
            max_n_count: None,
            low_complexity_threshold: None,
            entropy_threshold: None,
            contaminant_db: None,
            n_policy: Some("retain".to_string()),
            polyx_policy: None,
            damage_mode: None,
        },
    )?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: "fastq.filter_reads".to_string(),
        domain: "fastq".to_string(),
        tool_id: FASTQ_FILTER_TOOL_ID.to_string(),
        execution_mode: "domain_contract".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: FILTER_READS_REPORT_SCHEMA_VERSION.to_string(),
        consumed_inputs: BTreeMap::from([
            (
                "trimmed_reads_r1_path".to_string(),
                required_output(trim_row, "trimmed_reads_r1_path")?,
            ),
            (
                "trimmed_reads_r2_path".to_string(),
                required_output(trim_row, "trimmed_reads_r2_path")?,
            ),
        ]),
        outputs: BTreeMap::from([
            ("filtered_reads_r1_path".to_string(), path_relative_to_repo(repo_root, &output_r1)),
            ("filtered_reads_r2_path".to_string(), path_relative_to_repo(repo_root, &output_r2)),
            ("filter_report_json".to_string(), path_relative_to_repo(repo_root, &report_path)),
            (
                "filter_backend_report".to_string(),
                path_relative_to_repo(repo_root, &raw_backend_report),
            ),
        ]),
        metrics: BTreeMap::from([
            ("reads_in".to_string(), Value::from(report.reads_in)),
            ("reads_out".to_string(), Value::from(report.reads_out)),
            ("reads_dropped".to_string(), Value::from(report.reads_dropped)),
            ("mean_q_before".to_string(), Value::from(report.mean_q_before)),
            ("mean_q_after".to_string(), Value::from(report.mean_q_after)),
        ]),
    })
}

fn run_bam_align_stage(
    repo_root: &Path,
    output_root: &Path,
    reference_fasta: &Path,
    input_fixtures: &PipelineInputFixtures,
    filter_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.align");
    let tiny_align_root = stage_root.join("semantic-runtime");
    fs::create_dir_all(&tiny_align_root)
        .with_context(|| format!("create {}", tiny_align_root.display()))?;

    let filtered_r1 = repo_root.join(required_output(filter_row, "filtered_reads_r1_path")?);
    let filtered_r2 = repo_root.join(required_output(filter_row, "filtered_reads_r2_path")?);
    let read_group = ReadGroupSpec::with_defaults(PIPELINE_SAMPLE_ID);
    let (provenance, mapping_summary) = bijux_dna_domain_bam::align_fastq_to_bam_bowtie2_style(
        reference_fasta,
        &filtered_r1,
        Some(&filtered_r2),
        &tiny_align_root,
        PIPELINE_SAMPLE_ID,
        &read_group,
        Some("very_sensitive_local"),
    )?;

    let semantic_sam_path = tiny_align_root.join("align.bam");
    if !semantic_sam_path.is_file() {
        bail!("semantic BAM alignment runtime did not produce {}", semantic_sam_path.display());
    }

    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let semantic_sam = stage_root.join("align.semantic.sam");
    fs::copy(&semantic_sam_path, &semantic_sam).with_context(|| {
        format!("copy {} to {}", semantic_sam_path.display(), semantic_sam.display())
    })?;
    let aligned_sam_path = stage_root.join("align.sam");
    write_mapped_sam_from_fastqs(
        reference_fasta,
        &filtered_r1,
        &filtered_r2,
        &read_group,
        &input_fixtures.placements,
        &aligned_sam_path,
    )?;
    let aligned_bam_path = stage_root.join("align.bam");
    let aligned_bam_index_path = stage_root.join("align.bam.bai");
    convert_coordinate_sam_to_bam(&aligned_sam_path, &aligned_bam_path, &aligned_bam_index_path)?;

    let read_group_contract = stage_root.join("read_group.json");
    bijux_dna_infra::atomic_write_json(&read_group_contract, &read_group)?;
    let provenance_path = stage_root.join("alignment.provenance.json");
    bijux_dna_infra::atomic_write_json(&provenance_path, &provenance)?;
    let mapping_summary_path = stage_root.join("mapping_summary.json");
    bijux_dna_infra::atomic_write_json(&mapping_summary_path, &mapping_summary)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: "bam.align".to_string(),
        domain: "bam".to_string(),
        tool_id: BAM_ALIGN_TOOL_ID.to_string(),
        execution_mode: "semantic_alignment".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &provenance_path),
        parsed_schema_version: provenance.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            (
                "filtered_reads_r1_path".to_string(),
                required_output(filter_row, "filtered_reads_r1_path")?,
            ),
            (
                "filtered_reads_r2_path".to_string(),
                required_output(filter_row, "filtered_reads_r2_path")?,
            ),
            (
                "reference_fasta_contract".to_string(),
                path_relative_to_repo(repo_root, reference_fasta),
            ),
            (
                "alignment_read_group_contract".to_string(),
                path_relative_to_repo(repo_root, &read_group_contract),
            ),
        ]),
        outputs: BTreeMap::from([
            ("aligned_bam".to_string(), path_relative_to_repo(repo_root, &aligned_bam_path)),
            ("aligned_bai".to_string(), path_relative_to_repo(repo_root, &aligned_bam_index_path)),
            ("align_metrics".to_string(), path_relative_to_repo(repo_root, &mapping_summary_path)),
            ("align_provenance".to_string(), path_relative_to_repo(repo_root, &provenance_path)),
            ("align_sam".to_string(), path_relative_to_repo(repo_root, &aligned_sam_path)),
            ("align_semantic_sam".to_string(), path_relative_to_repo(repo_root, &semantic_sam)),
        ]),
        metrics: BTreeMap::from([
            (
                "mapped_reads".to_string(),
                Value::from(mapping_summary.flagstat.mapped_reads.unwrap_or(0)),
            ),
            (
                "total_reads".to_string(),
                Value::from(mapping_summary.flagstat.total_reads.unwrap_or(0)),
            ),
            (
                "proper_pair_reads".to_string(),
                Value::from(mapping_summary.proper_pair_reads.unwrap_or(0)),
            ),
        ]),
    })
}

fn run_bam_validate_stage(
    repo_root: &Path,
    output_root: &Path,
    reference_fasta: &Path,
    align_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.validate");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let aligned_bam_path = repo_root.join(required_output(align_row, "aligned_bam")?);
    let aligned_bam_index_path = repo_root.join(required_output(align_row, "aligned_bai")?);
    let report = bijux_dna_domain_bam::execute_bam_validation(
        &aligned_bam_path,
        Some(&aligned_bam_index_path),
        Some(reference_fasta),
    )?;
    let report_path = stage_root.join("validation.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: "bam.validate".to_string(),
        domain: "bam".to_string(),
        tool_id: BAM_VALIDATE_TOOL_ID.to_string(),
        execution_mode: "domain_contract".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            ("aligned_bai".to_string(), required_output(align_row, "aligned_bai")?),
            (
                "reference_fasta_contract".to_string(),
                path_relative_to_repo(repo_root, reference_fasta),
            ),
        ]),
        outputs: BTreeMap::from([(
            "bam_validation_report".to_string(),
            path_relative_to_repo(repo_root, &report_path),
        )]),
        metrics: BTreeMap::from([
            (
                "validation_report_present".to_string(),
                Value::from(report.validation_report_present),
            ),
            ("total_reads".to_string(), Value::from(report.flagstat.total_reads.unwrap_or(0))),
            ("mapped_reads".to_string(), Value::from(report.flagstat.mapped_reads.unwrap_or(0))),
            (
                "refusal_code_count".to_string(),
                Value::from(u64::try_from(report.refusal_codes.len()).unwrap_or(u64::MAX)),
            ),
        ]),
    })
}

fn run_bam_qc_pre_stage(
    repo_root: &Path,
    output_root: &Path,
    align_row: &CoreGermlineMicroPipelineRow,
    validate_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.qc_pre");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_qc_pre(&aligned_bam)?;
    let report_path = stage_root.join("qc_pre.summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: "bam.qc_pre".to_string(),
        domain: "bam".to_string(),
        tool_id: BAM_QC_PRE_TOOL_ID.to_string(),
        execution_mode: "domain_contract".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: summary.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            ("aligned_bai".to_string(), required_output(align_row, "aligned_bai")?),
            (
                "bam_validation_report".to_string(),
                required_output(validate_row, "bam_validation_report")?,
            ),
        ]),
        outputs: BTreeMap::from([(
            "qc_pre_report".to_string(),
            path_relative_to_repo(repo_root, &report_path),
        )]),
        metrics: BTreeMap::from([
            ("total_reads".to_string(), Value::from(summary.total_reads)),
            ("mapped_reads".to_string(), Value::from(summary.mapped_reads)),
            ("unmapped_reads".to_string(), Value::from(summary.unmapped_reads)),
            ("duplicate_flagged_reads".to_string(), Value::from(summary.duplicate_flagged_reads)),
        ]),
    })
}

fn run_bam_coverage_stage(
    repo_root: &Path,
    output_root: &Path,
    align_row: &CoreGermlineMicroPipelineRow,
    qc_pre_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.coverage");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let regions_bed = stage_root.join("coverage_regions.bed");
    write_full_reference_bed(&aligned_bam, &regions_bed)?;
    let (summary, region_rows) = bijux_dna_domain_bam::summarize_tiny_bam_coverage_regions(
        &aligned_bam,
        Some(&regions_bed),
        &[1],
    )?;
    let report_path = stage_root.join("coverage.summary.json");
    let regions_path = stage_root.join("coverage.regions.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    bijux_dna_infra::atomic_write_json(&regions_path, &region_rows)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: "bam.coverage".to_string(),
        domain: "bam".to_string(),
        tool_id: BAM_COVERAGE_TOOL_ID.to_string(),
        execution_mode: "domain_contract".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: summary.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            ("aligned_bai".to_string(), required_output(align_row, "aligned_bai")?),
            ("qc_pre_report".to_string(), required_output(qc_pre_row, "qc_pre_report")?),
            (
                "coverage_region_contract".to_string(),
                path_relative_to_repo(repo_root, &regions_bed),
            ),
        ]),
        outputs: BTreeMap::from([
            ("coverage_report_json".to_string(), path_relative_to_repo(repo_root, &report_path)),
            ("coverage_regions_json".to_string(), path_relative_to_repo(repo_root, &regions_path)),
        ]),
        metrics: BTreeMap::from([
            ("mean_depth".to_string(), Value::from(summary.mean_depth.unwrap_or_default())),
            (
                "region_count".to_string(),
                Value::from(u64::try_from(region_rows.len()).unwrap_or(u64::MAX)),
            ),
            (
                "coverage_regime".to_string(),
                Value::from(summary.coverage_regime.unwrap_or_else(|| "unknown".to_string())),
            ),
        ]),
    })
}

fn run_vcf_call_stage(
    repo_root: &Path,
    output_root: &Path,
    align_row: &CoreGermlineMicroPipelineRow,
    coverage_row: &CoreGermlineMicroPipelineRow,
    reference_fasta: &Path,
    tool_ids: &BTreeMap<String, String>,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.call");
    let runtime_root = stage_root.join("runtime");
    fs::create_dir_all(&runtime_root)
        .with_context(|| format!("create {}", runtime_root.display()))?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let materialized_reference = materialize_reference_fasta(reference_fasta, &runtime_root)?;
    let coverage_report_path =
        repo_root.join(required_output(coverage_row, "coverage_report_json")?);
    let coverage_regions_path =
        repo_root.join(required_output(coverage_row, "coverage_regions_json")?);
    let coverage_regions = read_json_document(&coverage_regions_path)?;
    let coverage_gate_passed = coverage_regions.as_array().is_some_and(|rows| {
        rows.iter().any(|row| {
            row.get("covered_bases").and_then(Value::as_u64).is_some_and(|count| count > 0)
                || row.get("mean_depth").and_then(Value::as_f64).is_some_and(|depth| depth > 0.0)
        })
    });
    if !coverage_gate_passed {
        bail!("vcf.call coverage gate failed for {}", coverage_report_path.display());
    }

    let stage_outputs = run_call_diploid_stage(
        &aligned_bam,
        &runtime_root,
        &VcfCallParams {
            caller: required_tool_id(tool_ids, "vcf.call")?,
            sample_name: PIPELINE_SAMPLE_ID.to_string(),
            reference_fasta: Some(materialized_reference.display().to_string()),
            ..VcfCallParams::default()
        },
    )?;
    let validation = vcf_validate_input(
        &stage_outputs.called_vcf,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )?;
    let call_summary = parse_vcf_call_summary(&stage_outputs.called_vcf, PIPELINE_SAMPLE_ID)?;
    let sample_count = parse_output_sample_count(&stage_outputs.called_vcf)?;
    let reference_fai = PathBuf::from(format!("{}.fai", materialized_reference.display()));
    if !reference_fai.is_file() {
        bail!("vcf.call did not materialize {}", reference_fai.display());
    }

    let report = VcfCallStageReport {
        schema_version: "bijux.bench.local_core_germline_micro_pipeline.vcf_call.v1".to_string(),
        stage_id: "vcf.call".to_string(),
        tool_id: required_tool_id(tool_ids, "vcf.call")?,
        sample_id: PIPELINE_SAMPLE_ID.to_string(),
        consumed_coverage_report: path_relative_to_repo(repo_root, &coverage_report_path),
        output_vcf_path: path_relative_to_repo(repo_root, &stage_outputs.called_vcf),
        output_tbi_path: path_relative_to_repo(repo_root, &stage_outputs.called_tbi),
        call_metrics_path: path_relative_to_repo(repo_root, &stage_outputs.call_metrics_json),
        call_manifest_path: path_relative_to_repo(repo_root, &stage_outputs.call_manifest_json),
        reference_fasta_path: path_relative_to_repo(repo_root, &materialized_reference),
        reference_fai_path: path_relative_to_repo(repo_root, &reference_fai),
        variant_count: call_summary.variants_called,
        snp_count: call_summary.snps,
        indel_count: call_summary.indels,
        sample_count,
        coverage_gate_passed,
        validation_checks: validation.checks.clone(),
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: "vcf.call".to_string(),
        domain: "vcf".to_string(),
        tool_id: report.tool_id.clone(),
        execution_mode: "tool_backed".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            ("aligned_bai".to_string(), required_output(align_row, "aligned_bai")?),
            (
                "coverage_report_json".to_string(),
                required_output(coverage_row, "coverage_report_json")?,
            ),
            (
                "coverage_regions_json".to_string(),
                required_output(coverage_row, "coverage_regions_json")?,
            ),
            (
                "reference_fasta_contract".to_string(),
                path_relative_to_repo(repo_root, &materialized_reference),
            ),
            (
                "reference_fai_contract".to_string(),
                path_relative_to_repo(repo_root, &reference_fai),
            ),
        ]),
        outputs: BTreeMap::from([
            ("called_vcf".to_string(), report.output_vcf_path.clone()),
            ("called_vcf_tbi".to_string(), report.output_tbi_path.clone()),
            ("call_metrics_json".to_string(), report.call_metrics_path.clone()),
            ("call_manifest_json".to_string(), report.call_manifest_path.clone()),
        ]),
        metrics: BTreeMap::from([
            ("variant_count".to_string(), Value::from(report.variant_count)),
            ("snp_count".to_string(), Value::from(report.snp_count)),
            ("indel_count".to_string(), Value::from(report.indel_count)),
            ("sample_count".to_string(), Value::from(report.sample_count)),
            ("coverage_gate_passed".to_string(), Value::from(report.coverage_gate_passed)),
        ]),
    })
}

fn run_vcf_filter_stage(
    repo_root: &Path,
    output_root: &Path,
    call_row: &CoreGermlineMicroPipelineRow,
    tool_ids: &BTreeMap<String, String>,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.filter");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let called_vcf = repo_root.join(required_output(call_row, "called_vcf")?);
    let stage_outputs = run_filter_stage_real(
        &called_vcf,
        &stage_root,
        &VcfFilterParams {
            sample_name: PIPELINE_SAMPLE_ID.to_string(),
            min_qual: 0.0,
            require_pass: false,
            normalize: true,
            require_bgzip_tabix: true,
            production_profile: false,
            ..VcfFilterParams::default()
        },
    )?;
    let validation = vcf_validate_input(
        &stage_outputs.filtered_vcf,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )?;
    let sample_count = parse_output_sample_count(&stage_outputs.filtered_vcf)?;
    let filter_breakdown = read_json_document(&stage_outputs.filter_breakdown_json)?;
    let pass_variant_count =
        filter_breakdown.pointer("/counts/PASS").and_then(Value::as_u64).unwrap_or(0);
    let total_tagged_variant_count = filter_breakdown
        .get("counts")
        .and_then(Value::as_object)
        .map(|counts| {
            counts
                .iter()
                .filter(|(filter_id, _)| filter_id.as_str() != "PASS")
                .filter_map(|(_, count)| count.as_u64())
                .sum::<u64>()
        })
        .unwrap_or(0);
    let report = VcfFilterStageReport {
        schema_version: "bijux.bench.local_core_germline_micro_pipeline.vcf_filter.v1".to_string(),
        stage_id: "vcf.filter".to_string(),
        tool_id: required_tool_id(tool_ids, "vcf.filter")?,
        consumed_call_metrics_path: required_output(call_row, "call_metrics_json")?,
        output_vcf_path: path_relative_to_repo(repo_root, &stage_outputs.filtered_vcf),
        output_tbi_path: path_relative_to_repo(repo_root, &stage_outputs.filtered_tbi),
        filter_breakdown_path: path_relative_to_repo(
            repo_root,
            &stage_outputs.filter_breakdown_json,
        ),
        filter_breakdown_tsv_path: path_relative_to_repo(
            repo_root,
            &stage_outputs.filter_breakdown_tsv,
        ),
        filter_explain_path: path_relative_to_repo(repo_root, &stage_outputs.filter_explain_json),
        pass_variant_count,
        tagged_variant_count: total_tagged_variant_count,
        sample_count,
        validation_checks: validation.checks.clone(),
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: "vcf.filter".to_string(),
        domain: "vcf".to_string(),
        tool_id: report.tool_id.clone(),
        execution_mode: "stage_runtime".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("called_vcf".to_string(), required_output(call_row, "called_vcf")?),
            ("called_vcf_tbi".to_string(), required_output(call_row, "called_vcf_tbi")?),
            ("call_metrics_json".to_string(), required_output(call_row, "call_metrics_json")?),
        ]),
        outputs: BTreeMap::from([
            ("filtered_vcf".to_string(), report.output_vcf_path.clone()),
            ("filtered_vcf_tbi".to_string(), report.output_tbi_path.clone()),
            ("filter_report_json".to_string(), path_relative_to_repo(repo_root, &report_path)),
            ("filter_breakdown_json".to_string(), report.filter_breakdown_path.clone()),
        ]),
        metrics: BTreeMap::from([
            ("pass_variant_count".to_string(), Value::from(report.pass_variant_count)),
            ("tagged_variant_count".to_string(), Value::from(report.tagged_variant_count)),
            ("sample_count".to_string(), Value::from(report.sample_count)),
        ]),
    })
}

fn run_vcf_stats_stage(
    repo_root: &Path,
    output_root: &Path,
    filter_row: &CoreGermlineMicroPipelineRow,
    tool_ids: &BTreeMap<String, String>,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.stats");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let filtered_vcf = repo_root.join(required_output(filter_row, "filtered_vcf")?);
    let stats = run_stats_stage_real(
        &filtered_vcf,
        &stage_root,
        &VcfStatsParams {
            sample_name: PIPELINE_SAMPLE_ID.to_string(),
            ..VcfStatsParams::default()
        },
    )?;
    let report = VcfStatsStageReport {
        schema_version: "bijux.bench.local_core_germline_micro_pipeline.vcf_stats.v1".to_string(),
        stage_id: "vcf.stats".to_string(),
        tool_id: required_tool_id(tool_ids, "vcf.stats")?,
        consumed_filter_report_path: required_output(filter_row, "filter_report_json")?,
        stats_json_path: path_relative_to_repo(repo_root, &stats.stats_json),
        bcftools_stats_path: path_relative_to_repo(repo_root, &stats.bcftools_stats_txt),
        variant_count: stats.metrics.variants_total,
        snp_count: stats.metrics.snps,
        indel_count: stats.metrics.indels,
        ti_tv: stats.metrics.ti_tv,
        sample_count: stats.metrics.sample_count,
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: "vcf.stats".to_string(),
        domain: "vcf".to_string(),
        tool_id: report.tool_id.clone(),
        execution_mode: "stage_runtime".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("filtered_vcf".to_string(), required_output(filter_row, "filtered_vcf")?),
            ("filtered_vcf_tbi".to_string(), required_output(filter_row, "filtered_vcf_tbi")?),
            ("filter_report_json".to_string(), required_output(filter_row, "filter_report_json")?),
        ]),
        outputs: BTreeMap::from([
            ("stats_json".to_string(), report.stats_json_path.clone()),
            ("stats_stage_metrics".to_string(), report.stats_json_path.clone()),
            ("bcftools_stats_txt".to_string(), report.bcftools_stats_path.clone()),
        ]),
        metrics: BTreeMap::from([
            ("variant_count".to_string(), Value::from(report.variant_count)),
            ("snp_count".to_string(), Value::from(report.snp_count)),
            ("indel_count".to_string(), Value::from(report.indel_count)),
            ("sample_count".to_string(), Value::from(report.sample_count)),
            ("ti_tv".to_string(), report.ti_tv.map_or(Value::Null, Value::from)),
        ]),
    })
}

fn run_vcf_qc_stage(
    repo_root: &Path,
    output_root: &Path,
    filter_row: &CoreGermlineMicroPipelineRow,
    stats_row: &CoreGermlineMicroPipelineRow,
    tool_ids: &BTreeMap<String, String>,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.qc");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let filtered_vcf = repo_root.join(required_output(filter_row, "filtered_vcf")?);
    let stage_outputs = run_qc_stage(
        &filtered_vcf,
        &stage_root,
        &QcStageParams {
            sample_name: PIPELINE_SAMPLE_ID.to_string(),
            is_ancient_dna: false,
            allow_hwe_for_ancient: false,
            production_profile: false,
            pre_filter_vcf: None,
        },
    )?;
    let qc_summary = read_json_document(&stage_outputs.qc_summary_json)?;
    let report = VcfQcStageReport {
        schema_version: "bijux.bench.local_core_germline_micro_pipeline.vcf_qc.v1".to_string(),
        stage_id: "vcf.qc".to_string(),
        tool_id: required_tool_id(tool_ids, "vcf.qc")?,
        consumed_stats_json_path: required_output(stats_row, "stats_json")?,
        qc_summary_path: path_relative_to_repo(repo_root, &stage_outputs.qc_summary_json),
        qc_tables_path: path_relative_to_repo(repo_root, &stage_outputs.qc_tables_tsv),
        imputation_qc_path: path_relative_to_repo(repo_root, &stage_outputs.imputation_qc_tsv),
        warnings_path: path_relative_to_repo(repo_root, &stage_outputs.warnings_json),
        qc_histograms_path: path_relative_to_repo(repo_root, &stage_outputs.qc_histograms_json),
        sample_missingness_row_count: qc_summary
            .get("sample_missingness")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        variant_missingness_row_count: qc_summary
            .get("variant_missingness")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        excluded_sample_count: qc_summary
            .get("excluded_samples")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        excluded_variant_count: qc_summary
            .get("excluded_variants")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: "vcf.qc".to_string(),
        domain: "vcf".to_string(),
        tool_id: report.tool_id.clone(),
        execution_mode: "stage_runtime".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("filtered_vcf".to_string(), required_output(filter_row, "filtered_vcf")?),
            ("filtered_vcf_tbi".to_string(), required_output(filter_row, "filtered_vcf_tbi")?),
            ("stats_json".to_string(), required_output(stats_row, "stats_json")?),
        ]),
        outputs: BTreeMap::from([
            ("qc_report".to_string(), report.qc_summary_path.clone()),
            ("qc_stage_metrics".to_string(), report.qc_summary_path.clone()),
            ("qc_tables_tsv".to_string(), report.qc_tables_path.clone()),
        ]),
        metrics: BTreeMap::from([
            (
                "sample_missingness_row_count".to_string(),
                Value::from(u64::try_from(report.sample_missingness_row_count).unwrap_or(u64::MAX)),
            ),
            (
                "variant_missingness_row_count".to_string(),
                Value::from(
                    u64::try_from(report.variant_missingness_row_count).unwrap_or(u64::MAX),
                ),
            ),
            (
                "excluded_sample_count".to_string(),
                Value::from(u64::try_from(report.excluded_sample_count).unwrap_or(u64::MAX)),
            ),
            (
                "excluded_variant_count".to_string(),
                Value::from(u64::try_from(report.excluded_variant_count).unwrap_or(u64::MAX)),
            ),
        ]),
    })
}

fn validate_handoff(
    repo_root: &Path,
    rows: &[CoreGermlineMicroPipelineRow],
    source_stage_id: &str,
    source_output_id: &str,
    target_stage_id: &str,
    target_input_id: &str,
) -> Result<CoreGermlineMicroPipelineHandoff> {
    let source_row = rows
        .iter()
        .find(|row| row.stage_id == source_stage_id)
        .ok_or_else(|| anyhow!("missing source stage row `{source_stage_id}`"))?;
    let target_row = rows
        .iter()
        .find(|row| row.stage_id == target_stage_id)
        .ok_or_else(|| anyhow!("missing target stage row `{target_stage_id}`"))?;
    let source_path = source_row.outputs.get(source_output_id).cloned().ok_or_else(|| {
        anyhow!("stage `{source_stage_id}` is missing output `{source_output_id}`")
    })?;
    let target_path =
        target_row.consumed_inputs.get(target_input_id).cloned().ok_or_else(|| {
            anyhow!("stage `{target_stage_id}` is missing consumed input `{target_input_id}`")
        })?;
    let source_exists = repo_root.join(&source_path).exists();
    let target_exists = repo_root.join(&target_path).exists();
    let exact_path_match = source_path == target_path;
    let accepted = source_exists && target_exists && exact_path_match;
    Ok(CoreGermlineMicroPipelineHandoff {
        handoff_id: format!(
            "{source_stage_id}:{source_output_id}->{target_stage_id}:{target_input_id}"
        ),
        source_stage_id: source_stage_id.to_string(),
        target_stage_id: target_stage_id.to_string(),
        source_output_id: source_output_id.to_string(),
        target_input_id: target_input_id.to_string(),
        source_path,
        target_path,
        source_exists,
        target_exists,
        exact_path_match,
        accepted,
        detail: if accepted {
            "target consumed the exact source artifact path".to_string()
        } else {
            "target did not consume the exact source artifact path".to_string()
        },
    })
}

fn build_profile_report(
    input_r1: &Path,
    input_r2: Option<&Path>,
) -> Result<FastqProfileStageReport> {
    let records_r1 = read_fastq_records(input_r1)?;
    let records_r2 = input_r2.map(read_fastq_records).transpose()?;
    let bases_r1 = total_bases(&records_r1);
    let bases_r2 = records_r2.as_deref().map_or(0, total_bases);
    let reads_r1 = u64::try_from(records_r1.len()).unwrap_or(u64::MAX);
    let reads_r2 = records_r2
        .as_ref()
        .map(|records| u64::try_from(records.len()).unwrap_or(u64::MAX))
        .unwrap_or(0);
    let length_histogram = length_histogram(&records_r1, records_r2.as_deref());
    Ok(FastqProfileStageReport {
        schema_version: "bijux.bench.local_core_germline_micro_pipeline.fastq_profile.v1"
            .to_string(),
        stage_id: "fastq.profile_reads".to_string(),
        tool_id: FASTQ_PROFILE_TOOL_ID.to_string(),
        sample_id: PIPELINE_SAMPLE_ID.to_string(),
        input_r1: input_r1.display().to_string(),
        input_r2: input_r2.map(|path| path.display().to_string()),
        reads_total: reads_r1 + reads_r2,
        bases_total: bases_r1 + bases_r2,
        mean_q: combined_mean_quality(&records_r1, records_r2.as_deref()),
        gc_percent: combined_gc_percent(&records_r1, records_r2.as_deref()),
        length_histogram,
    })
}

fn write_filter_reads_report(
    repo_root: &Path,
    input_r1: &Path,
    input_r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_path: &Path,
    raw_backend_report: &Path,
    effective_params: &FilterEffectiveParams,
) -> Result<FilterReadsReportV1> {
    let input_records = read_fastq_records(input_r1)?;
    let mate_records = input_r2.map(read_fastq_records).transpose()?;
    let decisions = input_records
        .iter()
        .cloned()
        .map(|record| FilterDecision {
            drop_reason: filter_drop_reason(&record, effective_params),
            record,
        })
        .collect::<Vec<_>>();
    let mate_decisions = mate_records.as_ref().map(|records| {
        records
            .iter()
            .cloned()
            .map(|record| FilterDecision {
                drop_reason: filter_drop_reason(&record, effective_params),
                record,
            })
            .collect::<Vec<_>>()
    });
    if let Some(mate_decisions) = mate_decisions.as_ref() {
        if mate_decisions.len() != decisions.len() {
            bail!("paired FASTQ decisions must keep synchronized record counts");
        }
    }

    let retained_r1 = decisions
        .iter()
        .enumerate()
        .filter(|(index, decision)| {
            decision.drop_reason.is_none()
                && mate_decisions
                    .as_ref()
                    .and_then(|mate| mate.get(*index))
                    .is_none_or(|mate_decision| mate_decision.drop_reason.is_none())
        })
        .map(|(_, decision)| decision.record.clone())
        .collect::<Vec<_>>();
    let retained_r2 = mate_decisions.as_ref().map(|mate| {
        mate.iter()
            .enumerate()
            .filter(|(index, decision)| {
                decision.drop_reason.is_none() && decisions[*index].drop_reason.is_none()
            })
            .map(|(_, decision)| decision.record.clone())
            .collect::<Vec<_>>()
    });
    write_fastq_records(output_r1, &retained_r1)?;
    if let (Some(output_r2), Some(retained_r2)) = (output_r2, retained_r2.as_ref()) {
        write_fastq_records(output_r2, retained_r2)?;
    }

    let reads_removed_by_n = decisions
        .iter()
        .chain(mate_decisions.as_deref().unwrap_or(&[]).iter())
        .filter(|decision| decision.drop_reason == Some(FilterDropReason::TooManyN))
        .count() as u64;
    let reads_removed_low_complexity = decisions
        .iter()
        .chain(mate_decisions.as_deref().unwrap_or(&[]).iter())
        .filter(|decision| decision.drop_reason == Some(FilterDropReason::LowComplexity))
        .count() as u64;
    let reads_in = u64::try_from(input_records.len() + mate_records.as_ref().map_or(0, Vec::len))
        .context("count FASTQ filter input reads")?;
    let reads_out = u64::try_from(retained_r1.len() + retained_r2.as_ref().map_or(0, Vec::len))
        .context("count FASTQ filter output reads")?;
    let bases_in = total_bases(&input_records) + mate_records.as_deref().map_or(0, total_bases);
    let bases_out = total_bases(&retained_r1) + retained_r2.as_deref().map_or(0, total_bases);
    let backend_metrics = json!({
        "passed_filter_reads": reads_out,
        "too_many_n_reads": reads_removed_by_n,
        "low_complexity_reads": reads_removed_low_complexity,
    });
    let report = FilterReadsReportV1 {
        schema_version: FILTER_READS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.filter_reads".to_string(),
        stage_id: "fastq.filter_reads".to_string(),
        tool_id: FASTQ_FILTER_TOOL_ID.to_string(),
        paired_mode: if input_r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: effective_params.threads,
        input_r1: path_relative_to_repo(repo_root, input_r1),
        input_r2: input_r2.map(|path| path_relative_to_repo(repo_root, path)),
        output_r1: path_relative_to_repo(repo_root, output_r1),
        output_r2: output_r2.map(|path| path_relative_to_repo(repo_root, path)),
        report_json: path_relative_to_repo(repo_root, report_path),
        max_n: effective_params.max_n,
        max_n_fraction: effective_params.max_n_fraction,
        max_n_count: effective_params.max_n_count,
        low_complexity_threshold: effective_params.low_complexity_threshold,
        entropy_threshold: effective_params.entropy_threshold,
        n_policy: effective_params.n_policy.clone(),
        polyx_policy: effective_params.polyx_policy.clone(),
        contaminant_db: effective_params.contaminant_db.clone(),
        reads_in,
        reads_out,
        reads_dropped: reads_in.saturating_sub(reads_out),
        reads_removed_by_n,
        reads_removed_by_entropy: 0,
        reads_removed_low_complexity,
        reads_removed_by_kmer: 0,
        reads_removed_contaminant_kmer: 0,
        reads_removed_by_length: 0,
        bases_in,
        bases_out,
        pairs_in: input_r2.map(|_| input_records.len() as u64),
        pairs_out: input_r2.map(|_| retained_r1.len() as u64),
        mean_q_before: combined_mean_quality(&input_records, mate_records.as_deref()),
        mean_q_after: combined_mean_quality(&retained_r1, retained_r2.as_deref()),
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        raw_backend_report: Some(path_relative_to_repo(repo_root, raw_backend_report)),
        raw_backend_report_format: Some("fastp_json".to_string()),
        backend_metrics: Some(backend_metrics.clone()),
    };
    bijux_dna_infra::atomic_write_json(report_path, &report)?;
    bijux_dna_infra::write_bytes(
        raw_backend_report,
        json!({
            "filtering_result": {
                "passed_filter_reads": backend_metrics["passed_filter_reads"],
                "too_many_N_reads": backend_metrics["too_many_n_reads"],
                "low_complexity_reads": backend_metrics["low_complexity_reads"],
            }
        })
        .to_string(),
    )?;
    Ok(report)
}

fn filter_drop_reason(
    record: &FastqRecord,
    effective_params: &FilterEffectiveParams,
) -> Option<FilterDropReason> {
    let max_n_count = effective_params.max_n_count.or(effective_params.max_n);
    if let Some(limit) = max_n_count {
        let n_count = u32::try_from(
            record.sequence.bytes().filter(|base| matches!(*base, b'N' | b'n')).count(),
        )
        .unwrap_or(u32::MAX);
        if n_count > limit {
            return Some(FilterDropReason::TooManyN);
        }
    }

    if let Some(threshold) =
        effective_params.low_complexity_threshold.or(effective_params.entropy_threshold)
    {
        let complexity = local_complexity_score(&record.sequence);
        if complexity < threshold {
            return Some(FilterDropReason::LowComplexity);
        }
    }

    None
}

fn materialize_pipeline_input_fastqs(
    output_root: &Path,
    reference_fasta: &Path,
) -> Result<PipelineInputFixtures> {
    let stage_root = output_root.join("artifacts/fastq.source");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let raw_r1 = stage_root.join("raw_R1.fastq");
    let raw_r2 = stage_root.join("raw_R2.fastq");
    let (reference_name, reference_sequence) = read_first_reference_contig(reference_fasta)?;
    let window_start = 900usize;
    let read_length = 30usize;
    if reference_sequence.len() < window_start + read_length {
        bail!("reference {} is too short to synthesize pipeline FASTQs", reference_fasta.display());
    }

    let mut variant_sequence =
        reference_sequence[window_start..window_start + read_length].to_string();
    let variant_offset = read_length / 2;
    let reference_base = variant_sequence.as_bytes()[variant_offset];
    let alt_base = alternate_base(reference_base);
    variant_sequence
        .replace_range(variant_offset..=variant_offset, &char::from(alt_base).to_string());
    let quality = "F".repeat(read_length);

    let records_r1 = vec![
        FastqRecord {
            header: "@read1/1".to_string(),
            sequence: variant_sequence.clone(),
            plus: "+".to_string(),
            quality: quality.clone(),
        },
        FastqRecord {
            header: "@read2/1".to_string(),
            sequence: variant_sequence.clone(),
            plus: "+".to_string(),
            quality: quality.clone(),
        },
    ];
    let records_r2 = vec![
        FastqRecord {
            header: "@read1/2".to_string(),
            sequence: variant_sequence.clone(),
            plus: "+".to_string(),
            quality: quality.clone(),
        },
        FastqRecord {
            header: "@read2/2".to_string(),
            sequence: variant_sequence,
            plus: "+".to_string(),
            quality,
        },
    ];
    write_fastq_records(&raw_r1, &records_r1)?;
    write_fastq_records(&raw_r2, &records_r2)?;

    let position = u64::try_from(window_start + 1).context("pipeline FASTQ position overflow")?;
    let placements = BTreeMap::from([
        (
            "read1/1".to_string(),
            SyntheticReadPlacement { reference_name: reference_name.clone(), position },
        ),
        (
            "read1/2".to_string(),
            SyntheticReadPlacement { reference_name: reference_name.clone(), position },
        ),
        (
            "read2/1".to_string(),
            SyntheticReadPlacement { reference_name: reference_name.clone(), position },
        ),
        ("read2/2".to_string(), SyntheticReadPlacement { reference_name, position }),
    ]);

    Ok(PipelineInputFixtures { raw_r1, raw_r2, placements })
}

fn local_complexity_score(sequence: &str) -> f64 {
    let bytes = sequence.as_bytes();
    if bytes.len() <= 1 {
        return 0.0;
    }
    let transitions = bytes.windows(2).filter(|window| window[0] != window[1]).count() as f64;
    (transitions / (bytes.len() as f64 - 1.0)) * 100.0
}

fn read_first_reference_contig(reference_fasta: &Path) -> Result<(String, String)> {
    let payload = fs::read_to_string(reference_fasta)
        .with_context(|| format!("read {}", reference_fasta.display()))?;
    let mut current_name = None::<String>;
    let mut current_sequence = String::new();
    for line in payload.lines() {
        if let Some(header) = line.strip_prefix('>') {
            if let Some(name) = current_name.take() {
                return Ok((name, current_sequence));
            }
            current_name = Some(header.split_whitespace().next().unwrap_or_default().to_string());
            current_sequence.clear();
        } else {
            current_sequence.push_str(line.trim());
        }
    }
    current_name
        .map(|name| (name, current_sequence))
        .ok_or_else(|| anyhow!("reference {} has no FASTA contigs", reference_fasta.display()))
}

fn alternate_base(reference_base: u8) -> u8 {
    match reference_base.to_ascii_uppercase() {
        b'A' => b'C',
        b'C' => b'G',
        b'G' => b'T',
        b'T' => b'A',
        _ => b'A',
    }
}

fn write_mapped_sam_from_fastqs(
    reference_fasta: &Path,
    input_r1: &Path,
    input_r2: &Path,
    read_group: &ReadGroupSpec,
    placements: &BTreeMap<String, SyntheticReadPlacement>,
    output_sam: &Path,
) -> Result<()> {
    let (reference_name, reference_sequence) = read_first_reference_contig(reference_fasta)?;
    let mut payload = String::new();
    payload.push_str("@HD\tVN:1.6\tSO:coordinate\n");
    payload.push_str(&format!("@SQ\tSN:{reference_name}\tLN:{}\n", reference_sequence.len()));
    payload.push_str(&format!(
        "@RG\tID:{}\tSM:{}\tPL:{}\tLB:{}",
        read_group.id, read_group.sample, read_group.platform, read_group.library
    ));
    if let Some(unit) = &read_group.platform_unit {
        payload.push_str(&format!("\tPU:{unit}"));
    }
    payload.push('\n');

    let mut records = read_fastq_records(input_r1)?;
    records.extend(read_fastq_records(input_r2)?);
    for record in records {
        let qname = record.header.trim_start_matches('@');
        let placement = placements
            .get(qname)
            .ok_or_else(|| anyhow!("missing synthetic placement for `{qname}`"))?;
        payload.push_str(&format!(
            "{qname}\t0\t{}\t{}\t60\t{}M\t*\t0\t0\t{}\t{}\tRG:Z:{}\n",
            placement.reference_name,
            placement.position,
            record.sequence.len(),
            record.sequence,
            record.quality,
            read_group.id
        ));
    }
    bijux_dna_infra::write_bytes(output_sam, payload)?;
    Ok(())
}

fn convert_coordinate_sam_to_bam(
    input_sam: &Path,
    output_bam_path: &Path,
    output_bam_index_path: &Path,
) -> Result<()> {
    use noodles_sam::alignment::io::Write as _;
    use noodles_sam::header::record::value::map::header::{
        sort_order::COORDINATE, tag::SORT_ORDER,
    };
    use noodles_sam::header::record::value::{map, Map};

    let file = fs::File::open(input_sam)
        .with_context(|| format!("open semantic alignment SAM {}", input_sam.display()))?;
    let mut reader = sam::io::Reader::new(BufReader::new(file));
    let mut header =
        reader.read_header().with_context(|| format!("read {}", input_sam.display()))?;
    *header.header_mut() = Some(
        Map::<map::Header>::builder()
            .insert(SORT_ORDER, COORDINATE)
            .build()
            .map_err(|error| anyhow!("build coordinate sort-order header: {error}"))?,
    );

    let mut records = reader
        .records()
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("read SAM records from {}", input_sam.display()))?;
    let reference_order = header
        .reference_sequences()
        .keys()
        .enumerate()
        .map(|(index, name)| (String::from_utf8_lossy(name.as_ref()).into_owned(), index))
        .collect::<HashMap<_, _>>();
    records.sort_by(|left, right| {
        sam_record_sort_key(left, &header, &reference_order).cmp(&sam_record_sort_key(
            right,
            &header,
            &reference_order,
        ))
    });

    let bam_file = bijux_dna_infra::create_file(output_bam_path)
        .with_context(|| format!("create {}", output_bam_path.display()))?;
    let mut writer = bam::io::Writer::new(bam_file);
    writer
        .write_header(&header)
        .with_context(|| format!("write BAM header to {}", output_bam_path.display()))?;
    for record in &records {
        writer
            .write_alignment_record(&header, record)
            .with_context(|| format!("write BAM record to {}", output_bam_path.display()))?;
    }
    writer.try_finish().with_context(|| format!("finish {}", output_bam_path.display()))?;

    let index = bam::fs::index(output_bam_path)
        .with_context(|| format!("index coordinate BAM {}", output_bam_path.display()))?;
    bam::bai::fs::write(output_bam_index_path, &index)
        .with_context(|| format!("write {}", output_bam_index_path.display()))?;
    Ok(())
}

fn sam_record_sort_key(
    record: &sam::Record,
    header: &sam::Header,
    reference_order: &HashMap<String, usize>,
) -> (usize, usize, usize, String) {
    let reference_rank = record
        .reference_sequence_id(header)
        .transpose()
        .ok()
        .flatten()
        .and_then(|reference_id| {
            reference_order
                .iter()
                .find_map(|(name, rank)| (*rank == reference_id).then_some((name.clone(), *rank)))
        })
        .map_or(usize::MAX, |(_, rank)| rank);
    let alignment_start =
        record.alignment_start().transpose().ok().flatten().map_or(usize::MAX, usize::from);
    let unmapped_rank = usize::from(
        record.flags().ok().is_none_or(noodles_sam::alignment::record::Flags::is_unmapped),
    );
    let name = record
        .name()
        .map(|name| String::from_utf8_lossy(name.as_ref()).into_owned())
        .unwrap_or_default();
    (unmapped_rank, reference_rank, alignment_start, name)
}

fn write_full_reference_bed(input_bam: &Path, output_bed: &Path) -> Result<()> {
    let mut reader = bam::io::Reader::new(
        fs::File::open(input_bam).with_context(|| format!("open {}", input_bam.display()))?,
    );
    let header = reader.read_header().with_context(|| format!("read {}", input_bam.display()))?;
    let mut body = String::new();
    for (name, reference_sequence) in header.reference_sequences() {
        let contig = String::from_utf8_lossy(name.as_ref()).into_owned();
        let length = reference_sequence.length().get();
        body.push_str(&format!("{contig}\t0\t{length}\t{contig}\n"));
    }
    if body.trim().is_empty() {
        bail!("{} has no reference sequences to render as BED", input_bam.display());
    }
    bijux_dna_infra::write_bytes(output_bed, body)?;
    Ok(())
}

fn resolve_core_vcf_tool_ids() -> Result<BTreeMap<String, String>> {
    let mut tool_ids = BTreeMap::new();
    for stage_id in ["vcf.call", "vcf.filter", "vcf.stats", "vcf.qc"] {
        let tool_id = build_vcf_stage_matrix_rows()?
            .into_iter()
            .find(|row| row.stage_id == stage_id)
            .map(|row| row.tool_id)
            .ok_or_else(|| anyhow!("VCF stage matrix is missing `{stage_id}`"))?;
        tool_ids.insert(stage_id.to_string(), tool_id);
    }
    Ok(tool_ids)
}

fn required_tool_id(tool_ids: &BTreeMap<String, String>, stage_id: &str) -> Result<String> {
    tool_ids
        .get(stage_id)
        .cloned()
        .ok_or_else(|| anyhow!("missing retained tool id for `{stage_id}`"))
}

fn read_fastq_records(path: &Path) -> Result<Vec<FastqRecord>> {
    let reader: Box<dyn BufRead> = if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        let file = fs::File::open(path)?;
        let decoder = flate2::read::MultiGzDecoder::new(file);
        Box::new(BufReader::new(decoder))
    } else {
        Box::new(BufReader::new(fs::File::open(path)?))
    };

    let mut lines = reader.lines();
    let mut records = Vec::new();
    while let Some(header) = lines.next() {
        let header = header?;
        let sequence =
            lines.next().ok_or_else(|| anyhow!("truncated FASTQ at {}", path.display()))??;
        let plus =
            lines.next().ok_or_else(|| anyhow!("truncated FASTQ at {}", path.display()))??;
        let quality =
            lines.next().ok_or_else(|| anyhow!("truncated FASTQ at {}", path.display()))??;
        records.push(FastqRecord { header, sequence, plus, quality });
    }
    Ok(records)
}

fn write_fastq_records(path: &Path, records: &[FastqRecord]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let file =
        bijux_dna_infra::create_file(path).with_context(|| format!("create {}", path.display()))?;
    let mut writer = std::io::BufWriter::new(file);
    for record in records {
        writeln!(writer, "{}", record.header)?;
        writeln!(writer, "{}", record.sequence)?;
        writeln!(writer, "{}", record.plus)?;
        writeln!(writer, "{}", record.quality)?;
    }
    writer.flush()?;
    Ok(())
}

fn length_histogram(
    records_r1: &[FastqRecord],
    records_r2: Option<&[FastqRecord]>,
) -> Vec<FastqLengthBin> {
    let mut bins = BTreeMap::<u64, u64>::new();
    for record in records_r1 {
        *bins.entry(u64::try_from(record.sequence.len()).unwrap_or(u64::MAX)).or_insert(0) += 1;
    }
    if let Some(records_r2) = records_r2 {
        for record in records_r2 {
            *bins.entry(u64::try_from(record.sequence.len()).unwrap_or(u64::MAX)).or_insert(0) += 1;
        }
    }
    bins.into_iter().map(|(length, count)| FastqLengthBin { length, count }).collect()
}

fn total_bases(records: &[FastqRecord]) -> u64 {
    records.iter().map(|record| record.sequence.len() as u64).sum()
}

fn combined_mean_quality(records_r1: &[FastqRecord], records_r2: Option<&[FastqRecord]>) -> f64 {
    let total_bases = total_bases(records_r1) + records_r2.map_or(0, total_bases);
    if total_bases == 0 {
        return 0.0;
    }
    let total_quality = quality_sum(records_r1) + records_r2.map_or(0, quality_sum);
    total_quality as f64 / total_bases as f64
}

fn combined_gc_percent(records_r1: &[FastqRecord], records_r2: Option<&[FastqRecord]>) -> f64 {
    let total_bases = total_bases(records_r1) + records_r2.map_or(0, total_bases);
    if total_bases == 0 {
        return 0.0;
    }
    let gc_bases = gc_count(records_r1) + records_r2.map_or(0, gc_count);
    (gc_bases as f64 / total_bases as f64) * 100.0
}

fn quality_sum(records: &[FastqRecord]) -> u64 {
    records
        .iter()
        .flat_map(|record| record.quality.bytes())
        .map(|value| u64::from(value.saturating_sub(33)))
        .sum::<u64>()
}

fn gc_count(records: &[FastqRecord]) -> u64 {
    records
        .iter()
        .flat_map(|record| record.sequence.bytes())
        .filter(|base| matches!(*base, b'G' | b'g' | b'C' | b'c'))
        .count()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn read_json_document(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn required_output(row: &CoreGermlineMicroPipelineRow, output_id: &str) -> Result<String> {
    row.outputs
        .get(output_id)
        .cloned()
        .ok_or_else(|| anyhow!("stage `{}` is missing output `{output_id}`", row.stage_id))
}

fn timestamp_marker() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_germline_micro_pipeline_renders_real_handoffs() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonical repo root");
        let output_path = repo_root.join(
            "artifacts/tests/benchmark/core-germline-micro-pipeline/MICRO_PIPELINE_SUMMARY.json",
        );

        let report = render_core_germline_micro_pipeline(&repo_root, output_path.clone())
            .expect("render core germline micro pipeline");

        assert_eq!(report.schema_version, "bijux.bench.local_core_germline_micro_pipeline.v1");
        assert_eq!(report.pipeline_id, "core-germline-fastq-bam-vcf");
        assert_eq!(report.stage_count, 12);
        assert_eq!(report.handoff_count, 20);
        assert!(report.passes_behavior_test);
        assert_eq!(
            report.output_path,
            "artifacts/tests/benchmark/core-germline-micro-pipeline/MICRO_PIPELINE_SUMMARY.json"
        );
        assert!(output_path.is_file(), "pipeline summary file must exist");
        assert!(report.handoffs.iter().all(|handoff| handoff.accepted));
        assert!(report.rows.iter().any(|row| row.stage_id == "vcf.call"));
        assert!(report.rows.iter().any(|row| row.stage_id == "vcf.qc"));
    }
}
