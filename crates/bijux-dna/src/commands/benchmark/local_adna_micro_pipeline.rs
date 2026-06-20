use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_bam::metrics::DamageMetricsV1;
use bijux_dna_domain_bam::params::ReadGroupSpec;
use bijux_dna_domain_fastq::params::remove_duplicates::{
    DedupMode, RemoveDuplicatesEffectiveParams, REMOVE_DUPLICATES_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::trim::TerminalDamageExecutionPolicy;
use bijux_dna_domain_fastq::params::validate::{PairSyncPolicy, ValidationMode};
use bijux_dna_domain_fastq::params::{DamageMode, PairedMode};
use bijux_dna_domain_fastq::stages::remove_duplicates;
use bijux_dna_domain_fastq::{
    validation_artifact_paths, TerminalDamageReportV1, TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION,
    VALIDATION_REPORT_SCHEMA_VERSION,
};
use bijux_dna_stages_vcf::pipeline::{
    run_call_pseudohaploid_stage, run_damage_filter_stage, run_stats_stage_real,
    DamageFilterStageParams, DamageUdgRegime,
};
use bijux_dna_stages_vcf::vcf_io::{read_vcf_text, vcf_validate_input, VcfFieldRequirement};
use noodles_bam as bam;
use noodles_sam as sam;
use serde::Serialize;
use serde_json::Value;

use super::local_stage_result_manifest::path_relative_to_repo;
use super::local_vcf_call_bam_smoke_support::{
    materialize_reference_fasta, parse_output_sample_count,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ADNA_MICRO_PIPELINE_PATH: &str =
    "runs/bench/micro/pipelines/adna/MICRO_ADNA_SUMMARY.json";
const GOVERNED_MICRO_STARTED_AT: &str = "1704067200";
const GOVERNED_MICRO_FINISHED_AT: &str = "1704067201";
const GOVERNED_MICRO_ELAPSED_SECONDS: f64 = 1.0;
const ADNA_MICRO_PIPELINE_SCHEMA_VERSION: &str = "bijux.bench.local_adna_micro_pipeline.v1";
const ADNA_MICRO_PIPELINE_COMMAND: &str = "bijux-dna bench local run-adna-micro-pipeline";
const ADNA_MICRO_PIPELINE_ID: &str = "adna-pseudohaploid-fastq-bam-vcf";
const ADNA_PIPELINE_SAMPLE_ID: &str = "adna-micro";
const ADNA_REFERENCE_FASTA_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta";
const ADNA_REFERENCE_CONTIG: &str = "chr1";
const FASTQ_VALIDATE_TOOL_ID: &str = "fastqvalidator";
const FASTQ_TRIM_TERMINAL_DAMAGE_TOOL_ID: &str = "cutadapt";
const FASTQ_REMOVE_DUPLICATES_TOOL_ID: &str = "bijux";
const BAM_ALIGN_TOOL_ID: &str = "bowtie2";
const BAM_VALIDATE_TOOL_ID: &str = "samtools";
const BAM_MAPPING_SUMMARY_TOOL_ID: &str = "samtools";
const BAM_COVERAGE_TOOL_ID: &str = "samtools";
const BAM_DAMAGE_TOOL_ID: &str = "mapdamage2";
const BAM_AUTHENTICITY_TOOL_ID: &str = "pmdtools";
const VCF_CALL_PSEUDOHAPLOID_TOOL_ID: &str = "bcftools";
const VCF_DAMAGE_FILTER_TOOL_ID: &str = "bcftools";
const VCF_STATS_TOOL_ID: &str = "bcftools";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdnaMicroPipelineReport {
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
    pub(crate) skipped_count: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) rows: Vec<AdnaMicroPipelineRow>,
    pub(crate) handoffs: Vec<AdnaMicroPipelineHandoff>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AdnaMicroPipelineRowStatus {
    Succeeded,
    Skipped,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdnaMicroPipelineRow {
    pub(crate) stage_id: String,
    pub(crate) domain: String,
    pub(crate) tool_id: String,
    pub(crate) execution_mode: String,
    pub(crate) status: AdnaMicroPipelineRowStatus,
    pub(crate) reason: String,
    pub(crate) evidence_path: Option<String>,
    pub(crate) parsed_schema_version: Option<String>,
    pub(crate) consumed_inputs: BTreeMap<String, String>,
    pub(crate) outputs: BTreeMap<String, String>,
    pub(crate) metrics: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdnaMicroPipelineHandoff {
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
struct AdnaPseudohaploidCallReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    sample_id: String,
    output_vcf_path: String,
    output_tbi_path: String,
    variant_count: u64,
    sample_count: u64,
    haploid_compatible: bool,
    gt_present: bool,
}

#[derive(Debug, Clone, Serialize)]
struct AdnaDamageFilterReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    filtered_vcf_path: String,
    filtered_tbi_path: String,
    summary_path: String,
    counts_path: String,
    retained_variants: u64,
    damage_ratio_filtered_variants: u64,
    terminal_damage_filtered_variants: u64,
    proxy_only_mode: bool,
}

#[derive(Debug, Clone, Serialize)]
struct AdnaStatsReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    stats_json_path: String,
    bcftools_stats_path: String,
    variant_count: u64,
    snp_count: u64,
    indel_count: u64,
}

#[derive(Debug, Clone)]
struct FastqRecord {
    header: String,
    sequence: String,
    plus: String,
    quality: String,
}

#[derive(Debug, Clone)]
struct PipelineInputFixtures {
    raw_r1: PathBuf,
    raw_r2: PathBuf,
    placements: BTreeMap<String, SyntheticReadPlacement>,
    reference_name: String,
}

#[derive(Debug, Clone)]
struct SyntheticReadPlacement {
    reference_name: String,
    position: u64,
}

pub(crate) fn run_adna_micro_pipeline(
    args: &parse::BenchLocalRunAdnaMicroPipelineArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_adna_micro_pipeline(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_ADNA_MICRO_PIPELINE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_adna_micro_pipeline(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AdnaMicroPipelineReport> {
    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(output_path) };
    let governed_output =
        path_relative_to_repo(repo_root, &absolute_output_path) == DEFAULT_ADNA_MICRO_PIPELINE_PATH;
    let output_root = absolute_output_path
        .parent()
        .ok_or_else(|| anyhow!("aDNA micro pipeline output has no parent directory"))?;
    reset_generated_output_root(output_root)?;

    let reference_fasta = repo_root.join(ADNA_REFERENCE_FASTA_PATH);
    if !reference_fasta.is_file() {
        bail!("aDNA micro pipeline is missing required reference input");
    }
    let input_fixtures = materialize_adna_pipeline_input_fastqs(output_root, &reference_fasta)?;
    let input_r1 = input_fixtures.raw_r1.clone();
    let input_r2 = input_fixtures.raw_r2.clone();

    let started_at =
        if governed_output { GOVERNED_MICRO_STARTED_AT.to_string() } else { timestamp_marker() };
    let started = Instant::now();

    let validate_row = run_fastq_validate_stage(repo_root, output_root, &input_r1, &input_r2)?;
    let trim_row = run_fastq_trim_terminal_damage_stage(
        repo_root,
        output_root,
        &input_r1,
        &input_r2,
        &validate_row,
    )?;
    let dedup_row = run_fastq_remove_duplicates_stage(repo_root, output_root, &trim_row)?;
    let align_row =
        run_bam_align_stage(repo_root, output_root, &reference_fasta, &input_fixtures, &dedup_row)?;
    let validate_bam_row =
        run_bam_validate_stage(repo_root, output_root, &reference_fasta, &align_row)?;
    let mapping_row =
        run_bam_mapping_summary_stage(repo_root, output_root, &align_row, &validate_bam_row)?;
    let coverage_row = run_bam_coverage_stage(repo_root, output_root, &align_row, &mapping_row)?;
    let damage_row = run_bam_damage_stage(
        repo_root,
        output_root,
        &reference_fasta,
        &input_fixtures,
        &trim_row,
        &align_row,
    )?;
    let authenticity_row =
        run_bam_authenticity_stage(repo_root, output_root, &align_row, &damage_row)?;
    let contamination_row = skipped_row(
        "bam.contamination",
        "bam",
        "verifybamid2",
        "synthetic aDNA micro execution does not claim panel-backed contamination evidence; authenticity stays damage-driven in this run",
    );
    let pseudohaploid_row = run_vcf_call_pseudohaploid_stage_row(
        repo_root,
        output_root,
        &align_row,
        &coverage_row,
        &damage_row,
        &authenticity_row,
        &reference_fasta,
    )?;
    let gl_call_row = skipped_row(
        "vcf.call_gl",
        "vcf",
        "angsd",
        "aDNA micro execution chooses the governed pseudohaploid branch; likelihood-bearing calling remains covered by dedicated GL smoke proof",
    );
    let gl_propagation_row = skipped_row(
        "vcf.gl_propagation",
        "vcf",
        "angsd",
        "aDNA micro execution does not produce a GL-bearing VCF because the pseudohaploid branch is the active governed path for this summary",
    );
    let damage_filter_row =
        run_vcf_damage_filter_stage_row(repo_root, output_root, &pseudohaploid_row)?;
    let stats_row = run_vcf_stats_stage_row(repo_root, output_root, &damage_filter_row)?;

    let rows = vec![
        validate_row,
        trim_row,
        dedup_row,
        align_row,
        validate_bam_row,
        mapping_row,
        coverage_row,
        damage_row,
        authenticity_row,
        contamination_row,
        pseudohaploid_row,
        gl_call_row,
        gl_propagation_row,
        damage_filter_row,
        stats_row,
    ];

    let handoffs = vec![
        validate_handoff(
            repo_root,
            &rows,
            "fastq.validate_reads",
            "validated_reads_r1_path",
            "fastq.trim_terminal_damage",
            "validated_reads_r1_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "fastq.validate_reads",
            "validated_reads_r2_path",
            "fastq.trim_terminal_damage",
            "validated_reads_r2_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "fastq.trim_terminal_damage",
            "terminal_damage_trimmed_reads_r1_path",
            "fastq.remove_duplicates",
            "terminal_damage_trimmed_reads_r1_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "fastq.trim_terminal_damage",
            "terminal_damage_trimmed_reads_r2_path",
            "fastq.remove_duplicates",
            "terminal_damage_trimmed_reads_r2_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "fastq.remove_duplicates",
            "deduplicated_reads_r1_path",
            "bam.align",
            "deduplicated_reads_r1_path",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "fastq.remove_duplicates",
            "deduplicated_reads_r2_path",
            "bam.align",
            "deduplicated_reads_r2_path",
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
            "bam.mapping_summary",
            "bam_validation_report",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.align",
            "align_sam",
            "bam.mapping_summary",
            "align_sam",
        )?,
        validate_handoff(repo_root, &rows, "bam.align", "align_sam", "bam.coverage", "align_sam")?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.mapping_summary",
            "mapping_summary_report_json",
            "bam.coverage",
            "mapping_summary_report_json",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.align",
            "aligned_bam",
            "bam.damage",
            "aligned_bam",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.damage",
            "damage_report_json",
            "bam.authenticity",
            "damage_report_json",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.align",
            "aligned_bam",
            "vcf.call_pseudohaploid",
            "aligned_bam",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.coverage",
            "coverage_report_json",
            "vcf.call_pseudohaploid",
            "coverage_report_json",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.damage",
            "damage_report_json",
            "vcf.call_pseudohaploid",
            "damage_report_json",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "bam.authenticity",
            "authenticity_report_json",
            "vcf.call_pseudohaploid",
            "authenticity_report_json",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "vcf.call_pseudohaploid",
            "pseudohaploid_vcf",
            "vcf.damage_filter",
            "pseudohaploid_vcf",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "vcf.call_pseudohaploid",
            "pseudohaploid_vcf_tbi",
            "vcf.damage_filter",
            "pseudohaploid_vcf_tbi",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "vcf.damage_filter",
            "damage_filtered_vcf",
            "vcf.stats",
            "damage_filtered_vcf",
        )?,
        validate_handoff(
            repo_root,
            &rows,
            "vcf.damage_filter",
            "damage_filtered_vcf_tbi",
            "vcf.stats",
            "damage_filtered_vcf_tbi",
        )?,
    ];

    let skipped_count =
        rows.iter().filter(|row| row.status == AdnaMicroPipelineRowStatus::Skipped).count();
    let succeeded_stage_ids = rows
        .iter()
        .filter(|row| row.status == AdnaMicroPipelineRowStatus::Succeeded)
        .map(|row| row.stage_id.as_str())
        .collect::<Vec<_>>();
    let passes_behavior_test = succeeded_stage_ids.contains(&"fastq.trim_terminal_damage")
        && succeeded_stage_ids.contains(&"fastq.remove_duplicates")
        && succeeded_stage_ids.contains(&"bam.align")
        && succeeded_stage_ids.contains(&"bam.damage")
        && succeeded_stage_ids.contains(&"bam.authenticity")
        && succeeded_stage_ids.contains(&"vcf.call_pseudohaploid")
        && handoffs.iter().all(|handoff| handoff.accepted)
        && rows.iter().all(|row| {
            row.status == AdnaMicroPipelineRowStatus::Succeeded
                || (row.status == AdnaMicroPipelineRowStatus::Skipped
                    && !row.reason.trim().is_empty())
        });

    let report = AdnaMicroPipelineReport {
        schema_version: ADNA_MICRO_PIPELINE_SCHEMA_VERSION,
        command: ADNA_MICRO_PIPELINE_COMMAND,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        pipeline_id: ADNA_MICRO_PIPELINE_ID,
        sample_id: ADNA_PIPELINE_SAMPLE_ID.to_string(),
        reference_fasta_path: path_relative_to_repo(repo_root, &reference_fasta),
        started_at,
        finished_at: if governed_output {
            GOVERNED_MICRO_FINISHED_AT.to_string()
        } else {
            timestamp_marker()
        },
        elapsed_seconds: if governed_output {
            GOVERNED_MICRO_ELAPSED_SECONDS
        } else {
            started.elapsed().as_secs_f64()
        },
        stage_count: rows.len(),
        handoff_count: handoffs.len(),
        skipped_count,
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
) -> Result<AdnaMicroPipelineRow> {
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

    Ok(succeeded_row(
        "fastq.validate_reads",
        "fastq",
        FASTQ_VALIDATE_TOOL_ID,
        "domain_contract",
        path_relative_to_repo(repo_root, &artifact_paths.report_json),
        VALIDATION_REPORT_SCHEMA_VERSION,
        BTreeMap::from([
            ("raw_reads_r1_path".to_string(), path_relative_to_repo(repo_root, input_r1)),
            ("raw_reads_r2_path".to_string(), path_relative_to_repo(repo_root, input_r2)),
        ]),
        BTreeMap::from([
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
        BTreeMap::from([
            ("validated_pairs".to_string(), Value::from(report.validated_pairs.unwrap_or(0))),
            ("strict_pass".to_string(), Value::from(report.strict_pass)),
        ]),
        "validated synthetic aDNA-like read pairs".to_string(),
    ))
}

fn run_fastq_trim_terminal_damage_stage(
    repo_root: &Path,
    output_root: &Path,
    input_r1: &Path,
    input_r2: &Path,
    validate_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/fastq.trim_terminal_damage");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let input_left = read_fastq_records(input_r1)?;
    let input_right = read_fastq_records(input_r2)?;
    if input_left.len() != input_right.len() {
        bail!("aDNA trim stage requires synchronized paired FASTQ inputs");
    }

    let output_r1 = stage_root.join("trimmed_R1.fastq");
    let output_r2 = stage_root.join("trimmed_R2.fastq");
    let report_path = stage_root.join("trim_terminal_damage_report.json");
    let backend_path = stage_root.join("trim_terminal_damage.backend.json");
    let trimmed_left =
        input_left.iter().map(trim_terminal_damage_record).collect::<Result<Vec<_>>>()?;
    let trimmed_right =
        input_right.iter().map(trim_terminal_damage_record).collect::<Result<Vec<_>>>()?;
    write_fastq_records(&output_r1, &trimmed_left)?;
    write_fastq_records(&output_r2, &trimmed_right)?;

    let reads_in = u64::try_from(input_left.len() + input_right.len()).unwrap_or(u64::MAX);
    let reads_out = u64::try_from(trimmed_left.len() + trimmed_right.len()).unwrap_or(u64::MAX);
    let bases_in = total_fastq_bases(&input_left) + total_fastq_bases(&input_right);
    let bases_out = total_fastq_bases(&trimmed_left) + total_fastq_bases(&trimmed_right);
    let terminal_base_composition_pre_r1 = Some(first_base_histogram(&input_left));
    let terminal_base_composition_post_r1 = Some(first_base_histogram(&trimmed_left));
    let terminal_base_composition_pre_r2 = Some(last_base_histogram(&input_right));
    let terminal_base_composition_post_r2 = Some(last_base_histogram(&trimmed_right));
    let report = TerminalDamageReportV1 {
        schema_version: TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.trim_terminal_damage".to_string(),
        stage_id: "fastq.trim_terminal_damage".to_string(),
        tool_id: FASTQ_TRIM_TERMINAL_DAMAGE_TOOL_ID.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: 1,
        damage_mode: DamageMode::Ancient,
        execution_policy: TerminalDamageExecutionPolicy::ExplicitTerminalTrim,
        trim_5p_bases: 1,
        trim_3p_bases: 1,
        requested_trim_5p_bases: Some(1),
        requested_trim_3p_bases: Some(1),
        udg_classification: "non_udg".to_string(),
        input_r1: path_relative_to_repo(repo_root, input_r1),
        input_r2: Some(path_relative_to_repo(repo_root, input_r2)),
        output_r1: path_relative_to_repo(repo_root, &output_r1),
        output_r2: Some(path_relative_to_repo(repo_root, &output_r2)),
        reads_in: Some(reads_in),
        reads_out: Some(reads_out),
        bases_in: Some(bases_in),
        bases_out: Some(bases_out),
        mean_q_before: Some(mean_quality(&input_left, &input_right)),
        mean_q_after: Some(mean_quality(&trimmed_left, &trimmed_right)),
        ct_ga_asymmetry_pre: Some(terminal_transition_asymmetry(&input_left, &input_right)),
        ct_ga_asymmetry_post: Some(terminal_transition_asymmetry(&trimmed_left, &trimmed_right)),
        ct_ga_asymmetry_pre_r1: Some(terminal_transition_asymmetry_one_end(&input_left, true)),
        ct_ga_asymmetry_post_r1: Some(terminal_transition_asymmetry_one_end(&trimmed_left, true)),
        ct_ga_asymmetry_pre_r2: Some(terminal_transition_asymmetry_one_end(&input_right, false)),
        ct_ga_asymmetry_post_r2: Some(terminal_transition_asymmetry_one_end(&trimmed_right, false)),
        terminal_base_composition_pre_r1,
        terminal_base_composition_post_r1,
        terminal_base_composition_pre_r2,
        terminal_base_composition_post_r2,
        raw_backend_report: Some(path_relative_to_repo(repo_root, &backend_path)),
        raw_backend_report_format: Some("bijux_terminal_damage_trace".to_string()),
        runtime_s: None,
        memory_mb: None,
        used_fallback: false,
        backend_metrics: Some(serde_json::json!({
            "reads_profiled_r1": input_left.len(),
            "reads_profiled_r2": input_right.len(),
            "bases_removed": bases_in.saturating_sub(bases_out),
        })),
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    bijux_dna_infra::atomic_write_json(
        &backend_path,
        &serde_json::json!({
            "schema_version": "bijux.fastq.trim_terminal_damage.backend_metrics.v1",
            "trim_5p_bases": 1,
            "trim_3p_bases": 1,
            "bases_removed": bases_in.saturating_sub(bases_out),
        }),
    )?;

    Ok(succeeded_row(
        "fastq.trim_terminal_damage",
        "fastq",
        FASTQ_TRIM_TERMINAL_DAMAGE_TOOL_ID,
        "governed_runtime",
        path_relative_to_repo(repo_root, &report_path),
        TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION,
        BTreeMap::from([
            (
                "validated_reads_r1_path".to_string(),
                required_output(validate_row, "validated_reads_r1_path")?,
            ),
            (
                "validated_reads_r2_path".to_string(),
                required_output(validate_row, "validated_reads_r2_path")?,
            ),
        ]),
        BTreeMap::from([
            (
                "terminal_damage_trimmed_reads_r1_path".to_string(),
                path_relative_to_repo(repo_root, &output_r1),
            ),
            (
                "terminal_damage_trimmed_reads_r2_path".to_string(),
                path_relative_to_repo(repo_root, &output_r2),
            ),
            (
                "terminal_damage_trim_report_json".to_string(),
                path_relative_to_repo(repo_root, &report_path),
            ),
            (
                "trim_terminal_damage_backend_json".to_string(),
                path_relative_to_repo(repo_root, &backend_path),
            ),
        ]),
        BTreeMap::from([
            ("reads_in".to_string(), Value::from(reads_in)),
            ("reads_out".to_string(), Value::from(reads_out)),
            ("bases_removed".to_string(), Value::from(bases_in.saturating_sub(bases_out))),
        ]),
        "trimmed one base from each read end while preserving residual terminal damage signal"
            .to_string(),
    ))
}

fn run_fastq_remove_duplicates_stage(
    repo_root: &Path,
    output_root: &Path,
    trim_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/fastq.remove_duplicates");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let input_r1 =
        repo_root.join(required_output(trim_row, "terminal_damage_trimmed_reads_r1_path")?);
    let input_r2 =
        repo_root.join(required_output(trim_row, "terminal_damage_trimmed_reads_r2_path")?);
    let output_r1 = stage_root.join("deduplicated_R1.fastq");
    let output_r2 = stage_root.join("deduplicated_R2.fastq");
    let classes_path = stage_root.join("duplicate_classes.tsv");
    let provenance_path = stage_root.join("duplicate_provenance.json");
    let report_path = stage_root.join("deduplication_report.json");
    let backend_path = stage_root.join("remove_duplicates.backend.txt");
    let report = remove_duplicates(
        &input_r1,
        Some(&input_r2),
        &RemoveDuplicatesEffectiveParams {
            schema_version: REMOVE_DUPLICATES_SCHEMA_VERSION.to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 1,
            dedup_mode: DedupMode::Exact,
            keep_order: true,
        },
        &output_r1,
        Some(&output_r2),
        &classes_path,
        &provenance_path,
        &report_path,
        Some(&backend_path),
    )?;

    Ok(succeeded_row(
        "fastq.remove_duplicates",
        "fastq",
        FASTQ_REMOVE_DUPLICATES_TOOL_ID,
        "domain_contract",
        path_relative_to_repo(repo_root, &report_path),
        &report.schema_version,
        BTreeMap::from([
            (
                "terminal_damage_trimmed_reads_r1_path".to_string(),
                required_output(trim_row, "terminal_damage_trimmed_reads_r1_path")?,
            ),
            (
                "terminal_damage_trimmed_reads_r2_path".to_string(),
                required_output(trim_row, "terminal_damage_trimmed_reads_r2_path")?,
            ),
        ]),
        BTreeMap::from([
            (
                "deduplicated_reads_r1_path".to_string(),
                path_relative_to_repo(repo_root, &output_r1),
            ),
            (
                "deduplicated_reads_r2_path".to_string(),
                path_relative_to_repo(repo_root, &output_r2),
            ),
            ("duplicate_classes_tsv".to_string(), path_relative_to_repo(repo_root, &classes_path)),
            (
                "duplicate_provenance_json".to_string(),
                path_relative_to_repo(repo_root, &provenance_path),
            ),
            ("deduplication_report".to_string(), path_relative_to_repo(repo_root, &report_path)),
        ]),
        BTreeMap::from([
            ("reads_in".to_string(), Value::from(report.reads_in)),
            ("reads_out".to_string(), Value::from(report.reads_out)),
            ("duplicates_removed".to_string(), Value::from(report.duplicates_removed)),
        ]),
        "removed exact duplicate aDNA-like read pairs while preserving first-observed order"
            .to_string(),
    ))
}

fn run_bam_align_stage(
    repo_root: &Path,
    output_root: &Path,
    reference_fasta: &Path,
    input_fixtures: &PipelineInputFixtures,
    dedup_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.align");
    let tiny_align_root = stage_root.join("semantic-runtime");
    fs::create_dir_all(&tiny_align_root)
        .with_context(|| format!("create {}", tiny_align_root.display()))?;

    let dedup_r1 = repo_root.join(required_output(dedup_row, "deduplicated_reads_r1_path")?);
    let dedup_r2 = repo_root.join(required_output(dedup_row, "deduplicated_reads_r2_path")?);
    let read_group = ReadGroupSpec::with_defaults(ADNA_PIPELINE_SAMPLE_ID);
    let (provenance, mapping_summary) = bijux_dna_domain_bam::align_fastq_to_bam_bowtie2_style(
        reference_fasta,
        &dedup_r1,
        Some(&dedup_r2),
        &tiny_align_root,
        ADNA_PIPELINE_SAMPLE_ID,
        &read_group,
        Some("very_sensitive_local"),
    )?;

    let semantic_sam_path = tiny_align_root.join("align.bam");
    if !semantic_sam_path.is_file() {
        bail!("semantic BAM alignment runtime did not produce {}", semantic_sam_path.display());
    }

    let semantic_sam = stage_root.join("align.semantic.sam");
    fs::copy(&semantic_sam_path, &semantic_sam).with_context(|| {
        format!("copy {} to {}", semantic_sam_path.display(), semantic_sam.display())
    })?;
    let aligned_sam = stage_root.join("align.sam");
    write_mapped_sam_from_fastqs(
        reference_fasta,
        &input_fixtures.reference_name,
        &dedup_r1,
        &dedup_r2,
        &read_group,
        &input_fixtures.placements,
        &aligned_sam,
    )?;
    let aligned_bam = stage_root.join("align.bam");
    let aligned_bai = stage_root.join("align.bam.bai");
    convert_coordinate_sam_to_bam(&aligned_sam, &aligned_bam, &aligned_bai)?;

    let read_group_contract = stage_root.join("read_group.json");
    bijux_dna_infra::atomic_write_json(&read_group_contract, &read_group)?;
    let provenance_path = stage_root.join("alignment.provenance.json");
    bijux_dna_infra::atomic_write_json(&provenance_path, &provenance)?;
    let mapping_summary_path = stage_root.join("alignment.mapping_summary.json");
    bijux_dna_infra::atomic_write_json(&mapping_summary_path, &mapping_summary)?;

    Ok(succeeded_row(
        "bam.align",
        "bam",
        BAM_ALIGN_TOOL_ID,
        "synthetic_alignment",
        path_relative_to_repo(repo_root, &provenance_path),
        &provenance.schema_version,
        BTreeMap::from([
            (
                "deduplicated_reads_r1_path".to_string(),
                required_output(dedup_row, "deduplicated_reads_r1_path")?,
            ),
            (
                "deduplicated_reads_r2_path".to_string(),
                required_output(dedup_row, "deduplicated_reads_r2_path")?,
            ),
            (
                "reference_fasta_contract".to_string(),
                path_relative_to_repo(repo_root, reference_fasta),
            ),
        ]),
        BTreeMap::from([
            ("aligned_bam".to_string(), path_relative_to_repo(repo_root, &aligned_bam)),
            ("aligned_bai".to_string(), path_relative_to_repo(repo_root, &aligned_bai)),
            ("align_sam".to_string(), path_relative_to_repo(repo_root, &aligned_sam)),
            ("align_metrics".to_string(), path_relative_to_repo(repo_root, &mapping_summary_path)),
            ("align_provenance".to_string(), path_relative_to_repo(repo_root, &provenance_path)),
        ]),
        BTreeMap::from([
            (
                "mapped_reads".to_string(),
                Value::from(mapping_summary.flagstat.mapped_reads.unwrap_or(0)),
            ),
            (
                "total_reads".to_string(),
                Value::from(mapping_summary.flagstat.total_reads.unwrap_or(0)),
            ),
        ]),
        "aligned duplicate-aware aDNA-like reads against the governed ancient-DNA reference"
            .to_string(),
    ))
}

fn run_bam_validate_stage(
    repo_root: &Path,
    output_root: &Path,
    reference_fasta: &Path,
    align_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.validate");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let aligned_bai = repo_root.join(required_output(align_row, "aligned_bai")?);
    let report = bijux_dna_domain_bam::execute_bam_validation(
        &aligned_bam,
        Some(&aligned_bai),
        Some(reference_fasta),
    )?;
    let report_path = stage_root.join("validation.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(succeeded_row(
        "bam.validate",
        "bam",
        BAM_VALIDATE_TOOL_ID,
        "domain_contract",
        path_relative_to_repo(repo_root, &report_path),
        &report.schema_version,
        BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            ("aligned_bai".to_string(), required_output(align_row, "aligned_bai")?),
        ]),
        BTreeMap::from([(
            "bam_validation_report".to_string(),
            path_relative_to_repo(repo_root, &report_path),
        )]),
        BTreeMap::from([
            (
                "validation_report_present".to_string(),
                Value::from(report.validation_report_present),
            ),
            ("mapped_reads".to_string(), Value::from(report.flagstat.mapped_reads.unwrap_or(0))),
        ]),
        "validated the aligned BAM against coordinate sort, index, and reference coherence rules"
            .to_string(),
    ))
}

fn run_bam_mapping_summary_stage(
    repo_root: &Path,
    output_root: &Path,
    align_row: &AdnaMicroPipelineRow,
    validate_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.mapping_summary");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let align_sam = repo_root.join(required_output(align_row, "align_sam")?);
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_mapping(&align_sam)?;
    let report_path = stage_root.join("mapping_summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;

    Ok(succeeded_row(
        "bam.mapping_summary",
        "bam",
        BAM_MAPPING_SUMMARY_TOOL_ID,
        "domain_contract",
        path_relative_to_repo(repo_root, &report_path),
        &summary.schema_version,
        BTreeMap::from([
            ("align_sam".to_string(), required_output(align_row, "align_sam")?),
            (
                "bam_validation_report".to_string(),
                required_output(validate_row, "bam_validation_report")?,
            ),
        ]),
        BTreeMap::from([(
            "mapping_summary_report_json".to_string(),
            path_relative_to_repo(repo_root, &report_path),
        )]),
        BTreeMap::from([
            ("mapped_reads".to_string(), Value::from(summary.flagstat.mapped_reads.unwrap_or(0))),
            (
                "mapped_fraction".to_string(),
                Value::from(summary.flagstat.mapped_fraction.unwrap_or(0.0)),
            ),
        ]),
        "summarized mapping behavior from the aligned aDNA-like BAM".to_string(),
    ))
}

fn run_bam_coverage_stage(
    repo_root: &Path,
    output_root: &Path,
    align_row: &AdnaMicroPipelineRow,
    mapping_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.coverage");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let align_sam = repo_root.join(required_output(align_row, "align_sam")?);
    let regions_path = stage_root.join("coverage_regions.bed");
    write_full_reference_bed(&align_sam, &regions_path)?;
    let (summary, regions) = bijux_dna_domain_bam::summarize_tiny_bam_coverage_regions(
        &align_sam,
        Some(&regions_path),
        &[1, 5],
    )?;
    let covered_bases = regions.iter().map(|region| region.covered_bases).sum::<u64>();
    let summary_path = stage_root.join("coverage.summary.json");
    let regions_json_path = stage_root.join("coverage.regions.json");
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)?;
    bijux_dna_infra::atomic_write_json(&regions_json_path, &regions)?;

    Ok(succeeded_row(
        "bam.coverage",
        "bam",
        BAM_COVERAGE_TOOL_ID,
        "domain_contract",
        path_relative_to_repo(repo_root, &summary_path),
        &summary.schema_version,
        BTreeMap::from([
            ("align_sam".to_string(), required_output(align_row, "align_sam")?),
            (
                "mapping_summary_report_json".to_string(),
                required_output(mapping_row, "mapping_summary_report_json")?,
            ),
        ]),
        BTreeMap::from([
            ("coverage_report_json".to_string(), path_relative_to_repo(repo_root, &summary_path)),
            (
                "coverage_regions_json".to_string(),
                path_relative_to_repo(repo_root, &regions_json_path),
            ),
        ]),
        BTreeMap::from([
            ("mean_depth".to_string(), summary.mean_depth.map_or(Value::Null, Value::from)),
            ("covered_bases".to_string(), Value::from(covered_bases)),
        ]),
        "measured coverage across the governed ancient-DNA reference contigs".to_string(),
    ))
}

fn run_bam_damage_stage(
    repo_root: &Path,
    output_root: &Path,
    reference_fasta: &Path,
    input_fixtures: &PipelineInputFixtures,
    trim_row: &AdnaMicroPipelineRow,
    align_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.damage");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let trimmed_r1 =
        repo_root.join(required_output(trim_row, "terminal_damage_trimmed_reads_r1_path")?);
    let trimmed_r2 =
        repo_root.join(required_output(trim_row, "terminal_damage_trimmed_reads_r2_path")?);
    let damage_metrics = derive_damage_metrics(
        reference_fasta,
        &input_fixtures.reference_name,
        &trimmed_r1,
        &trimmed_r2,
        &input_fixtures.placements,
    )?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_damage_evidence(
        &aligned_bam,
        &damage_metrics,
        true,
    )?;
    let damage_metrics_path = stage_root.join("damage.metrics.json");
    let report_path = stage_root.join("damage.summary.json");
    bijux_dna_infra::atomic_write_json(&damage_metrics_path, &damage_metrics)?;
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;

    Ok(succeeded_row(
        "bam.damage",
        "bam",
        BAM_DAMAGE_TOOL_ID,
        "domain_contract",
        path_relative_to_repo(repo_root, &report_path),
        &summary.schema_version,
        BTreeMap::from([("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?)]),
        BTreeMap::from([
            ("damage_report_json".to_string(), path_relative_to_repo(repo_root, &report_path)),
            (
                "damage_metrics_json".to_string(),
                path_relative_to_repo(repo_root, &damage_metrics_path),
            ),
        ]),
        BTreeMap::from([
            ("terminal_c_to_t_5p".to_string(), Value::from(summary.terminal_c_to_t_5p)),
            ("terminal_g_to_a_3p".to_string(), Value::from(summary.terminal_g_to_a_3p)),
            ("short_fragment_fraction".to_string(), Value::from(summary.short_fragment_fraction)),
        ]),
        "derived ancient-DNA terminal damage evidence from the aligned and trimmed read support"
            .to_string(),
    ))
}

fn run_bam_authenticity_stage(
    repo_root: &Path,
    output_root: &Path,
    align_row: &AdnaMicroPipelineRow,
    damage_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.authenticity");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let damage_metrics_path = repo_root.join(required_output(damage_row, "damage_metrics_json")?);
    let damage_metrics: DamageMetricsV1 = serde_json::from_slice(
        &fs::read(&damage_metrics_path)
            .with_context(|| format!("read {}", damage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", damage_metrics_path.display()))?;
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_authenticity_advisory(
        &aligned_bam,
        &damage_metrics,
    )?;
    let report_path = stage_root.join("authenticity.summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;

    Ok(succeeded_row(
        "bam.authenticity",
        "bam",
        BAM_AUTHENTICITY_TOOL_ID,
        "domain_contract",
        path_relative_to_repo(repo_root, &report_path),
        &summary.schema_version,
        BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            ("damage_report_json".to_string(), required_output(damage_row, "damage_report_json")?),
        ]),
        BTreeMap::from([(
            "authenticity_report_json".to_string(),
            path_relative_to_repo(repo_root, &report_path),
        )]),
        BTreeMap::from([
            ("score".to_string(), Value::from(summary.score)),
            ("confidence".to_string(), Value::from(summary.confidence)),
            ("pmd_like_signal_present".to_string(), Value::from(summary.pmd_like_signal_present)),
        ]),
        "composed authenticity advisory evidence from damage-bearing aligned molecules".to_string(),
    ))
}

fn run_vcf_call_pseudohaploid_stage_row(
    repo_root: &Path,
    output_root: &Path,
    align_row: &AdnaMicroPipelineRow,
    coverage_row: &AdnaMicroPipelineRow,
    damage_row: &AdnaMicroPipelineRow,
    authenticity_row: &AdnaMicroPipelineRow,
    reference_fasta: &Path,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.call_pseudohaploid");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let materialized_reference = materialize_reference_fasta(reference_fasta, &stage_root)?;
    let outputs = run_call_pseudohaploid_stage(
        &aligned_bam,
        &stage_root,
        &bijux_dna_domain_vcf::params::VcfCallParams {
            caller: VCF_CALL_PSEUDOHAPLOID_TOOL_ID.to_string(),
            sample_name: ADNA_PIPELINE_SAMPLE_ID.to_string(),
            reference_fasta: Some(materialized_reference.display().to_string()),
            ..bijux_dna_domain_vcf::params::VcfCallParams::default()
        },
    )?;
    let validation = vcf_validate_input(
        &outputs.called_vcf,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )?;
    let sample_count = parse_output_sample_count(&outputs.called_vcf)?;
    let (variant_count, haploid_compatible) = summarize_haploid_vcf(&outputs.called_vcf)?;
    let report = AdnaPseudohaploidCallReport {
        schema_version: "bijux.bench.local_adna_micro_pipeline.pseudohaploid_call.v1".to_string(),
        stage_id: "vcf.call_pseudohaploid".to_string(),
        tool_id: VCF_CALL_PSEUDOHAPLOID_TOOL_ID.to_string(),
        sample_id: ADNA_PIPELINE_SAMPLE_ID.to_string(),
        output_vcf_path: path_relative_to_repo(repo_root, &outputs.called_vcf),
        output_tbi_path: path_relative_to_repo(repo_root, &outputs.called_tbi),
        variant_count,
        sample_count,
        haploid_compatible,
        gt_present: validation.gt_present,
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(succeeded_row(
        "vcf.call_pseudohaploid",
        "vcf",
        VCF_CALL_PSEUDOHAPLOID_TOOL_ID,
        "governed_runtime",
        path_relative_to_repo(repo_root, &report_path),
        &report.schema_version,
        BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            (
                "coverage_report_json".to_string(),
                required_output(coverage_row, "coverage_report_json")?,
            ),
            ("damage_report_json".to_string(), required_output(damage_row, "damage_report_json")?),
            (
                "authenticity_report_json".to_string(),
                required_output(authenticity_row, "authenticity_report_json")?,
            ),
        ]),
        BTreeMap::from([
            (
                "pseudohaploid_vcf".to_string(),
                path_relative_to_repo(repo_root, &outputs.called_vcf),
            ),
            (
                "pseudohaploid_vcf_tbi".to_string(),
                path_relative_to_repo(repo_root, &outputs.called_tbi),
            ),
            ("call_stage_metrics".to_string(), path_relative_to_repo(repo_root, &report_path)),
        ]),
        BTreeMap::from([
            ("variant_count".to_string(), Value::from(variant_count)),
            ("sample_count".to_string(), Value::from(sample_count)),
            ("haploid_compatible".to_string(), Value::from(haploid_compatible)),
        ]),
        "called a governed pseudohaploid VCF from the aligned aDNA-like BAM".to_string(),
    ))
}

fn run_vcf_damage_filter_stage_row(
    repo_root: &Path,
    output_root: &Path,
    pseudohaploid_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.damage_filter");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let input_vcf = repo_root.join(required_output(pseudohaploid_row, "pseudohaploid_vcf")?);
    let outputs = run_damage_filter_stage(
        &input_vcf,
        &stage_root,
        &DamageFilterStageParams {
            min_qual: 0.0,
            max_damage_ratio: 0.35,
            udg_regime: DamageUdgRegime::NonUdg,
            strict_regime: false,
        },
    )?;
    let summary_json: serde_json::Value = serde_json::from_slice(
        &fs::read(&outputs.damage_filter_summary_json)
            .with_context(|| format!("read {}", outputs.damage_filter_summary_json.display()))?,
    )
    .with_context(|| format!("parse {}", outputs.damage_filter_summary_json.display()))?;
    let counts_json: serde_json::Value = serde_json::from_slice(
        &fs::read(&outputs.damage_filter_counts_json)
            .with_context(|| format!("read {}", outputs.damage_filter_counts_json.display()))?,
    )
    .with_context(|| format!("parse {}", outputs.damage_filter_counts_json.display()))?;
    let retained_variants = count_vcf_variants(&outputs.filtered_vcf)?;
    let report = AdnaDamageFilterReport {
        schema_version: "bijux.bench.local_adna_micro_pipeline.damage_filter.v1".to_string(),
        stage_id: "vcf.damage_filter".to_string(),
        tool_id: VCF_DAMAGE_FILTER_TOOL_ID.to_string(),
        filtered_vcf_path: path_relative_to_repo(repo_root, &outputs.filtered_vcf),
        filtered_tbi_path: path_relative_to_repo(repo_root, &outputs.filtered_tbi),
        summary_path: path_relative_to_repo(repo_root, &outputs.damage_filter_summary_json),
        counts_path: path_relative_to_repo(repo_root, &outputs.damage_filter_counts_json),
        retained_variants,
        damage_ratio_filtered_variants: count_json_u64(
            &counts_json,
            "/counts/damage_ratio_exceeded",
        ),
        terminal_damage_filtered_variants: count_json_u64(
            &counts_json,
            "/counts/terminal_damage_filtered",
        ),
        proxy_only_mode: summary_json
            .pointer("/prefilter/read_position_signal/proxy_used_records")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            > 0,
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(succeeded_row(
        "vcf.damage_filter",
        "vcf",
        VCF_DAMAGE_FILTER_TOOL_ID,
        "governed_runtime",
        path_relative_to_repo(repo_root, &report_path),
        &report.schema_version,
        BTreeMap::from([
            (
                "pseudohaploid_vcf".to_string(),
                required_output(pseudohaploid_row, "pseudohaploid_vcf")?,
            ),
            (
                "pseudohaploid_vcf_tbi".to_string(),
                required_output(pseudohaploid_row, "pseudohaploid_vcf_tbi")?,
            ),
        ]),
        BTreeMap::from([
            (
                "damage_filtered_vcf".to_string(),
                path_relative_to_repo(repo_root, &outputs.filtered_vcf),
            ),
            (
                "damage_filtered_vcf_tbi".to_string(),
                path_relative_to_repo(repo_root, &outputs.filtered_tbi),
            ),
            (
                "damage_bias_audit_report".to_string(),
                path_relative_to_repo(repo_root, &report_path),
            ),
            (
                "damage_filter_metrics".to_string(),
                path_relative_to_repo(repo_root, &outputs.damage_filter_counts_json),
            ),
        ]),
        BTreeMap::from([
            ("retained_variants".to_string(), Value::from(retained_variants)),
            (
                "damage_ratio_filtered_variants".to_string(),
                Value::from(report.damage_ratio_filtered_variants),
            ),
            (
                "terminal_damage_filtered_variants".to_string(),
                Value::from(report.terminal_damage_filtered_variants),
            ),
        ]),
        "applied damage-aware proxy filtering to the pseudohaploid VCF".to_string(),
    ))
}

fn run_vcf_stats_stage_row(
    repo_root: &Path,
    output_root: &Path,
    damage_filter_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.stats");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let filtered_vcf = repo_root.join(required_output(damage_filter_row, "damage_filtered_vcf")?);
    let outputs = run_stats_stage_real(
        &filtered_vcf,
        &stage_root,
        &bijux_dna_domain_vcf::params::VcfStatsParams {
            sample_name: ADNA_PIPELINE_SAMPLE_ID.to_string(),
            ..bijux_dna_domain_vcf::params::VcfStatsParams::default()
        },
    )?;
    let stats_json: bijux_dna_domain_vcf::VcfStatsMetricsV1 = serde_json::from_slice(
        &fs::read(&outputs.stats_json)
            .with_context(|| format!("read {}", outputs.stats_json.display()))?,
    )
    .with_context(|| format!("parse {}", outputs.stats_json.display()))?;
    let report = AdnaStatsReport {
        schema_version: "bijux.bench.local_adna_micro_pipeline.stats.v1".to_string(),
        stage_id: "vcf.stats".to_string(),
        tool_id: VCF_STATS_TOOL_ID.to_string(),
        stats_json_path: path_relative_to_repo(repo_root, &outputs.stats_json),
        bcftools_stats_path: path_relative_to_repo(repo_root, &outputs.bcftools_stats_txt),
        variant_count: stats_json.variants_total,
        snp_count: stats_json.snps,
        indel_count: stats_json.indels,
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(succeeded_row(
        "vcf.stats",
        "vcf",
        VCF_STATS_TOOL_ID,
        "governed_runtime",
        path_relative_to_repo(repo_root, &report_path),
        &report.schema_version,
        BTreeMap::from([
            (
                "damage_filtered_vcf".to_string(),
                required_output(damage_filter_row, "damage_filtered_vcf")?,
            ),
            (
                "damage_filtered_vcf_tbi".to_string(),
                required_output(damage_filter_row, "damage_filtered_vcf_tbi")?,
            ),
        ]),
        BTreeMap::from([
            ("stats_json".to_string(), path_relative_to_repo(repo_root, &outputs.stats_json)),
            (
                "bcftools_stats_txt".to_string(),
                path_relative_to_repo(repo_root, &outputs.bcftools_stats_txt),
            ),
        ]),
        BTreeMap::from([
            ("variant_count".to_string(), Value::from(report.variant_count)),
            ("snp_count".to_string(), Value::from(report.snp_count)),
            ("indel_count".to_string(), Value::from(report.indel_count)),
        ]),
        "summarized the retained ancient-DNA pseudohaploid calls after damage filtering"
            .to_string(),
    ))
}

fn succeeded_row(
    stage_id: &str,
    domain: &str,
    tool_id: &str,
    execution_mode: &str,
    evidence_path: String,
    parsed_schema_version: &str,
    consumed_inputs: BTreeMap<String, String>,
    outputs: BTreeMap<String, String>,
    metrics: BTreeMap<String, Value>,
    reason: String,
) -> AdnaMicroPipelineRow {
    AdnaMicroPipelineRow {
        stage_id: stage_id.to_string(),
        domain: domain.to_string(),
        tool_id: tool_id.to_string(),
        execution_mode: execution_mode.to_string(),
        status: AdnaMicroPipelineRowStatus::Succeeded,
        reason,
        evidence_path: Some(evidence_path),
        parsed_schema_version: Some(parsed_schema_version.to_string()),
        consumed_inputs,
        outputs,
        metrics,
    }
}

fn skipped_row(stage_id: &str, domain: &str, tool_id: &str, reason: &str) -> AdnaMicroPipelineRow {
    AdnaMicroPipelineRow {
        stage_id: stage_id.to_string(),
        domain: domain.to_string(),
        tool_id: tool_id.to_string(),
        execution_mode: "not_executed".to_string(),
        status: AdnaMicroPipelineRowStatus::Skipped,
        reason: reason.to_string(),
        evidence_path: None,
        parsed_schema_version: None,
        consumed_inputs: BTreeMap::new(),
        outputs: BTreeMap::new(),
        metrics: BTreeMap::new(),
    }
}

fn required_output(row: &AdnaMicroPipelineRow, key: &str) -> Result<String> {
    row.outputs
        .get(key)
        .cloned()
        .ok_or_else(|| anyhow!("{} is missing required output `{key}`", row.stage_id))
}

fn validate_handoff(
    repo_root: &Path,
    rows: &[AdnaMicroPipelineRow],
    source_stage_id: &str,
    source_output_id: &str,
    target_stage_id: &str,
    target_input_id: &str,
) -> Result<AdnaMicroPipelineHandoff> {
    let source_row = rows
        .iter()
        .find(|row| row.stage_id == source_stage_id)
        .ok_or_else(|| anyhow!("missing handoff source row `{source_stage_id}`"))?;
    let target_row = rows
        .iter()
        .find(|row| row.stage_id == target_stage_id)
        .ok_or_else(|| anyhow!("missing handoff target row `{target_stage_id}`"))?;
    let source_path = source_row.outputs.get(source_output_id).cloned().ok_or_else(|| {
        anyhow!("missing source output `{source_output_id}` on `{source_stage_id}`")
    })?;
    let target_path =
        target_row.consumed_inputs.get(target_input_id).cloned().ok_or_else(|| {
            anyhow!("missing target input `{target_input_id}` on `{target_stage_id}`")
        })?;
    let source_exists = repo_root.join(&source_path).exists();
    let target_exists = repo_root.join(&target_path).exists();
    let exact_path_match = source_path == target_path;
    let accepted = source_row.status == AdnaMicroPipelineRowStatus::Succeeded
        && target_row.status == AdnaMicroPipelineRowStatus::Succeeded
        && source_exists
        && target_exists
        && exact_path_match;
    Ok(AdnaMicroPipelineHandoff {
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
            "exact governed handoff observed".to_string()
        } else {
            "handoff missing, non-exact, or attached to a skipped stage".to_string()
        },
    })
}

fn materialize_adna_pipeline_input_fastqs(
    output_root: &Path,
    reference_fasta: &Path,
) -> Result<PipelineInputFixtures> {
    let stage_root = output_root.join("artifacts/fastq.source");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let (reference_name, reference_sequence) =
        read_named_reference_contig(reference_fasta, ADNA_REFERENCE_CONTIG)?;

    let mut placements = BTreeMap::<String, SyntheticReadPlacement>::new();
    let mut r1 = Vec::<FastqRecord>::new();
    let mut r2 = Vec::<FastqRecord>::new();

    let pair_specs = [
        ("adna_pair_a_r1", 1_u64, true, true, true),
        ("adna_pair_dup_r1", 1_u64, true, true, true),
        ("adna_pair_b_r1", 5_u64, true, true, true),
    ];
    for (header, position, add_internal_variant, add_5p_damage, add_3p_damage) in pair_specs {
        let sequence = synthetic_adna_read_sequence(
            &reference_sequence,
            usize::try_from(position).unwrap_or(1),
            24,
            add_internal_variant,
            add_5p_damage,
            add_3p_damage,
        )?;
        r1.push(FastqRecord {
            header: format!("@{header}"),
            sequence,
            plus: "+".to_string(),
            quality: "I".repeat(24),
        });
        placements.insert(
            header.to_string(),
            SyntheticReadPlacement { reference_name: reference_name.clone(), position },
        );
    }

    let mate_specs = [
        ("adna_pair_a_r2", 33_u64, false, true, true),
        ("adna_pair_dup_r2", 33_u64, false, true, true),
        ("adna_pair_b_r2", 37_u64, false, true, true),
    ];
    for (header, position, add_internal_variant, add_5p_damage, add_3p_damage) in mate_specs {
        let sequence = synthetic_adna_read_sequence(
            &reference_sequence,
            usize::try_from(position).unwrap_or(1),
            24,
            add_internal_variant,
            add_5p_damage,
            add_3p_damage,
        )?;
        r2.push(FastqRecord {
            header: format!("@{header}"),
            sequence,
            plus: "+".to_string(),
            quality: "I".repeat(24),
        });
        placements.insert(
            header.to_string(),
            SyntheticReadPlacement { reference_name: reference_name.clone(), position },
        );
    }

    let raw_r1 = stage_root.join("raw_R1.fastq");
    let raw_r2 = stage_root.join("raw_R2.fastq");
    write_fastq_records(&raw_r1, &r1)?;
    write_fastq_records(&raw_r2, &r2)?;

    Ok(PipelineInputFixtures { raw_r1, raw_r2, placements, reference_name })
}

fn synthetic_adna_read_sequence(
    reference_sequence: &str,
    start_position: usize,
    length: usize,
    add_internal_variant: bool,
    add_5p_damage: bool,
    add_3p_damage: bool,
) -> Result<String> {
    if start_position == 0 {
        bail!("synthetic aDNA read positions are 1-based");
    }
    let start_index = start_position - 1;
    let end_index = start_index.saturating_add(length);
    let reference_window = reference_sequence
        .get(start_index..end_index)
        .ok_or_else(|| anyhow!("synthetic aDNA read window is out of range"))?;
    let mut bases = reference_window.as_bytes().to_vec();
    if add_5p_damage && bases.len() > 2 && bases[1] == b'C' {
        bases[1] = b'T';
    }
    if add_3p_damage && bases.len() > 2 && bases[bases.len() - 2] == b'G' {
        let last_index = bases.len() - 2;
        bases[last_index] = b'A';
    }
    if add_internal_variant && bases.len() > 12 {
        let target = 11_usize;
        bases[target] = match bases[target] {
            b'A' => b'G',
            b'C' => b'A',
            b'G' => b'T',
            _ => b'G',
        };
    }
    String::from_utf8(bases)
        .map_err(|error| anyhow!("synthetic aDNA sequence is not UTF-8: {error}"))
}

fn trim_terminal_damage_record(record: &FastqRecord) -> Result<FastqRecord> {
    if record.sequence.len() <= 2 || record.quality.len() <= 2 {
        bail!("terminal damage trim requires reads longer than two bases");
    }
    Ok(FastqRecord {
        header: record.header.clone(),
        sequence: record.sequence[1..record.sequence.len() - 1].to_string(),
        plus: record.plus.clone(),
        quality: record.quality[1..record.quality.len() - 1].to_string(),
    })
}

fn derive_damage_metrics(
    reference_fasta: &Path,
    reference_name: &str,
    trimmed_r1: &Path,
    trimmed_r2: &Path,
    placements: &BTreeMap<String, SyntheticReadPlacement>,
) -> Result<DamageMetricsV1> {
    let (_, reference_sequence) = read_named_reference_contig(reference_fasta, reference_name)?;
    let mut records = read_fastq_records(trimmed_r1)?;
    records.extend(read_fastq_records(trimmed_r2)?);
    let mut ct_events = 0_u64;
    let mut ga_events = 0_u64;
    let mut eligible_5p = 0_u64;
    let mut eligible_3p = 0_u64;
    for record in records {
        let qname = record.header.trim_start_matches('@');
        let placement = placements
            .get(qname)
            .ok_or_else(|| anyhow!("missing synthetic placement for `{qname}`"))?;
        let start_index = usize::try_from(placement.position.saturating_sub(1)).unwrap_or(0) + 1;
        let end_index = start_index + record.sequence.len() - 1;
        let ref_start = reference_sequence
            .as_bytes()
            .get(start_index)
            .copied()
            .ok_or_else(|| anyhow!("missing reference start base for `{qname}`"))?;
        let ref_end = reference_sequence
            .as_bytes()
            .get(end_index)
            .copied()
            .ok_or_else(|| anyhow!("missing reference end base for `{qname}`"))?;
        let read_start = record.sequence.as_bytes()[0].to_ascii_uppercase();
        let read_end = record.sequence.as_bytes()[record.sequence.len() - 1].to_ascii_uppercase();
        if ref_start == b'C' {
            eligible_5p += 1;
            if read_start == b'T' {
                ct_events += 1;
            }
        }
        if ref_end == b'G' {
            eligible_3p += 1;
            if read_end == b'A' {
                ga_events += 1;
            }
        }
    }
    Ok(DamageMetricsV1 {
        c_to_t_5p: if eligible_5p == 0 { 0.0 } else { ct_events as f64 / eligible_5p as f64 },
        g_to_a_3p: if eligible_3p == 0 { 0.0 } else { ga_events as f64 / eligible_3p as f64 },
        pmd_score_histogram: vec![(0, 1), (3, ct_events.max(ga_events))],
    })
}

fn summarize_haploid_vcf(vcf_path: &Path) -> Result<(u64, bool)> {
    let raw = read_vcf_text(vcf_path)?;
    let mut variant_count = 0_u64;
    let mut haploid_compatible = true;
    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        variant_count += 1;
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() >= 10 {
            let format_tokens = fields[8].split(':').collect::<Vec<_>>();
            if let Some(index) = format_tokens.iter().position(|token| *token == "GT") {
                for sample_field in &fields[9..] {
                    if let Some(gt) = sample_field.split(':').nth(index) {
                        if gt.contains('/') || gt.contains('|') {
                            haploid_compatible = false;
                        }
                    }
                }
            }
        }
    }
    Ok((variant_count, haploid_compatible))
}

fn count_vcf_variants(vcf_path: &Path) -> Result<u64> {
    let raw = read_vcf_text(vcf_path)?;
    Ok(raw.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#')).count() as u64)
}

fn count_json_u64(value: &serde_json::Value, pointer: &str) -> u64 {
    value.pointer(pointer).and_then(serde_json::Value::as_u64).unwrap_or(0)
}

fn read_fastq_records(path: &Path) -> Result<Vec<FastqRecord>> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut lines = Vec::<String>::new();
    loop {
        let mut line = String::new();
        let bytes =
            reader.read_line(&mut line).with_context(|| format!("read {}", path.display()))?;
        if bytes == 0 {
            break;
        }
        lines.push(line.trim_end().to_string());
    }
    if lines.len() % 4 != 0 {
        bail!("FASTQ record count is incomplete in {}", path.display());
    }
    let mut records = Vec::new();
    for chunk in lines.chunks(4) {
        records.push(FastqRecord {
            header: chunk[0].clone(),
            sequence: chunk[1].clone(),
            plus: chunk[2].clone(),
            quality: chunk[3].clone(),
        });
    }
    Ok(records)
}

fn write_fastq_records(path: &Path, records: &[FastqRecord]) -> Result<()> {
    let mut payload = String::new();
    for record in records {
        payload.push_str(&record.header);
        payload.push('\n');
        payload.push_str(&record.sequence);
        payload.push('\n');
        payload.push_str(&record.plus);
        payload.push('\n');
        payload.push_str(&record.quality);
        payload.push('\n');
    }
    bijux_dna_infra::write_bytes(path, payload)?;
    Ok(())
}

fn total_fastq_bases(records: &[FastqRecord]) -> u64 {
    records.iter().map(|record| u64::try_from(record.sequence.len()).unwrap_or(0)).sum()
}

fn mean_quality(left: &[FastqRecord], right: &[FastqRecord]) -> f64 {
    let mut total = 0_u64;
    let mut count = 0_u64;
    for record in left.iter().chain(right.iter()) {
        for ch in record.quality.bytes() {
            total += u64::from(ch.saturating_sub(33));
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    }
}

fn terminal_transition_asymmetry(left: &[FastqRecord], right: &[FastqRecord]) -> f64 {
    let ct = left.iter().filter(|record| record.sequence.starts_with('T')).count() as f64;
    let ga = right.iter().filter(|record| record.sequence.ends_with('A')).count() as f64;
    if (ct + ga).abs() < f64::EPSILON {
        0.0
    } else {
        (ct - ga).abs() / (ct + ga)
    }
}

fn terminal_transition_asymmetry_one_end(records: &[FastqRecord], five_prime: bool) -> f64 {
    let target = if five_prime {
        records.iter().filter(|record| record.sequence.starts_with('T')).count() as f64
    } else {
        records.iter().filter(|record| record.sequence.ends_with('A')).count() as f64
    };
    if records.is_empty() {
        0.0
    } else {
        target / records.len() as f64
    }
}

fn first_base_histogram(records: &[FastqRecord]) -> BTreeMap<String, u64> {
    let mut histogram = BTreeMap::<String, u64>::new();
    for record in records {
        if let Some(base) = record.sequence.chars().next() {
            *histogram.entry(base.to_string()).or_insert(0) += 1;
        }
    }
    histogram
}

fn last_base_histogram(records: &[FastqRecord]) -> BTreeMap<String, u64> {
    let mut histogram = BTreeMap::<String, u64>::new();
    for record in records {
        if let Some(base) = record.sequence.chars().last() {
            *histogram.entry(base.to_string()).or_insert(0) += 1;
        }
    }
    histogram
}

fn timestamp_marker() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn read_named_reference_contig(reference_fasta: &Path, wanted: &str) -> Result<(String, String)> {
    let file = fs::File::open(reference_fasta)
        .with_context(|| format!("open {}", reference_fasta.display()))?;
    let reader = BufReader::new(file);
    let mut current_name = None::<String>;
    let mut current_sequence = String::new();
    for line in reader.lines() {
        let line = line.with_context(|| format!("read {}", reference_fasta.display()))?;
        if let Some(header) = line.strip_prefix('>') {
            if let Some(name) = current_name.take() {
                if name == wanted {
                    return Ok((name, current_sequence));
                }
                current_sequence = String::new();
            }
            current_name = Some(header.split_whitespace().next().unwrap_or_default().to_string());
            continue;
        }
        current_sequence.push_str(line.trim());
    }
    if let Some(name) = current_name {
        if name == wanted {
            return Ok((name, current_sequence));
        }
    }
    bail!("reference {} does not contain required contig `{wanted}`", reference_fasta.display())
}

fn write_mapped_sam_from_fastqs(
    reference_fasta: &Path,
    reference_name: &str,
    input_r1: &Path,
    input_r2: &Path,
    read_group: &ReadGroupSpec,
    placements: &BTreeMap<String, SyntheticReadPlacement>,
    output_sam: &Path,
) -> Result<()> {
    let (_, reference_sequence) = read_named_reference_contig(reference_fasta, reference_name)?;
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
    output_bam: &Path,
    output_bai: &Path,
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

    let bam_file = bijux_dna_infra::create_file(output_bam)
        .with_context(|| format!("create {}", output_bam.display()))?;
    let mut writer = bam::io::Writer::new(bam_file);
    writer
        .write_header(&header)
        .with_context(|| format!("write BAM header to {}", output_bam.display()))?;
    for record in &records {
        writer
            .write_alignment_record(&header, record)
            .with_context(|| format!("write BAM record to {}", output_bam.display()))?;
    }
    writer.try_finish().with_context(|| format!("finish {}", output_bam.display()))?;

    let index = bam::fs::index(output_bam)
        .with_context(|| format!("index coordinate BAM {}", output_bam.display()))?;
    bam::bai::fs::write(output_bai, &index)
        .with_context(|| format!("write {}", output_bai.display()))?;
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
    let alignment_start = record
        .alignment_start()
        .transpose()
        .ok()
        .flatten()
        .map_or(usize::MAX, |position| usize::from(position));
    let unmapped_rank = record.flags().ok().is_none_or(|flags| flags.is_unmapped()) as usize;
    let name = record
        .name()
        .map(|name| String::from_utf8_lossy(name.as_ref()).into_owned())
        .unwrap_or_default();
    (unmapped_rank, reference_rank, alignment_start, name)
}

fn write_full_reference_bed(input_bam: &Path, output_bed: &Path) -> Result<()> {
    let mut body = String::new();
    if input_bam.extension().and_then(|value| value.to_str()) == Some("sam") {
        let file =
            fs::File::open(input_bam).with_context(|| format!("open {}", input_bam.display()))?;
        let mut reader = sam::io::Reader::new(BufReader::new(file));
        let header =
            reader.read_header().with_context(|| format!("read {}", input_bam.display()))?;
        for (name, reference_sequence) in header.reference_sequences() {
            let contig = String::from_utf8_lossy(name.as_ref()).into_owned();
            let length = reference_sequence.length().get();
            body.push_str(&format!("{contig}\t0\t{length}\t{contig}\n"));
        }
    } else {
        let mut reader = bam::io::Reader::new(
            fs::File::open(input_bam).with_context(|| format!("open {}", input_bam.display()))?,
        );
        let header =
            reader.read_header().with_context(|| format!("read {}", input_bam.display()))?;
        for (name, reference_sequence) in header.reference_sequences() {
            let contig = String::from_utf8_lossy(name.as_ref()).into_owned();
            let length = reference_sequence.length().get();
            body.push_str(&format!("{contig}\t0\t{length}\t{contig}\n"));
        }
    }
    if body.trim().is_empty() {
        bail!("{} has no reference sequences to render as BED", input_bam.display());
    }
    bijux_dna_infra::write_bytes(output_bed, body)?;
    Ok(())
}
