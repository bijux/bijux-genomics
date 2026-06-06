use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_stages_vcf::pipeline::{
    run_ibd_stage, run_roh_stage, IbdStageParams, RohStageParams,
};
use serde::Serialize;

use super::local_corpus_fixture::vcf::{
    load_sample_metadata, load_vcf_corpus_fixture_manifest_path, SampleMetadataRow,
    DEFAULT_VCF_MINI_MANIFEST_PATH,
};
use super::local_stage_result_manifest::{
    path_relative_to_repo, validate_stage_result_manifest, BenchStageResultCommandV1,
    BenchStageResultManifestV1, BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_IBD_SMOKE_ROOT: &str = "target/local-smoke/vcf.ibd";
const LOCAL_VCF_IBD_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_ibd_smoke.v1";
const LOCAL_VCF_IBD_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-ibd-smoke";
const GOVERNED_VCF_IBD_STAGE_ID: &str = "vcf.ibd";
const GOVERNED_VCF_IBD_TOOL_ID: &str = "germline";
const GOVERNED_VCF_IBD_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_IBD_ASSET_PROFILE_ID: &str = "vcf_cohort";
const GOVERNED_VCF_IBD_INPUT_FIXTURE_ID: &str = "vcf_mini_multisample_cohort";
const DEFAULT_INPUT_VCF_NAME: &str = "ibd_input.vcf";
const DEFAULT_INPUT_SAMPLE_METADATA_NAME: &str = "sample_metadata.tsv";
const DEFAULT_OUTPUT_TSV_NAME: &str = "ibd.tsv";
const DEFAULT_OUTPUT_JSON_NAME: &str = "ibd.json";
const DEFAULT_OUTPUT_SOURCE_INPUT_NAME: &str = "source_ibd_input.tsv";
const DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME: &str = "source_ibd_segments.tsv";
const DEFAULT_OUTPUT_SOURCE_MERGED_SEGMENTS_NAME: &str = "source_ibd_merged_segments.tsv";
const DEFAULT_OUTPUT_SOURCE_FILTERED_SEGMENTS_NAME: &str = "source_ibd_filtered_segments.tsv";
const DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME: &str = "source_ibd_summary.json";
const DEFAULT_OUTPUT_SOURCE_METRICS_NAME: &str = "source_ibd_metrics.json";
const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
const DEFAULT_PROBE_INPUT_VCF_NAME: &str = "probe_sparse_overlap.vcf";
const DEFAULT_PROBE_SOURCE_SUMMARY_NAME: &str = "probe_source_ibd_summary.json";
const DEFAULT_PROBE_SOURCE_FILTERED_SEGMENTS_NAME: &str = "probe_source_ibd_filtered_segments.tsv";
const DEFAULT_PROBE_SOURCE_ROH_SUMMARY_NAME: &str = "probe_source_roh_summary.json";
const DEFAULT_PROBE_SOURCE_ROH_SEGMENTS_NAME: &str = "probe_source_roh_segments.tsv";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";
const GOVERNED_IBD_SEGMENT_MARKER_MINIMUM: usize = 1;
const PROBE_IBD_SEGMENT_MARKER_MINIMUM: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfIbdSmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ParsedIbdSegment {
    sample_a: String,
    sample_b: String,
    contig: String,
    start: u64,
    end: u64,
    length_cm: f64,
    marker_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfIbdSmokeRow {
    pub(crate) sample_a: String,
    pub(crate) sample_b: String,
    pub(crate) segment_count: u64,
    pub(crate) total_length: f64,
    pub(crate) overlap_marker_count: u64,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfIbdInsufficientOverlapProbe {
    pub(crate) input_vcf_path: String,
    pub(crate) ibd_status: String,
    pub(crate) insufficient_data_reason: String,
    pub(crate) overlap_marker_count: u64,
    pub(crate) filtered_segment_count: u64,
    pub(crate) source_ibd_summary_path: String,
    pub(crate) source_ibd_filtered_segments_path: String,
    pub(crate) unrelated_stage_id: String,
    pub(crate) unrelated_stage_status: String,
    pub(crate) unrelated_stage_segment_count: u64,
    pub(crate) source_roh_summary_path: String,
    pub(crate) source_roh_segments_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfIbdSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) fixture_manifest_path: String,
    pub(crate) input_vcf_path: String,
    pub(crate) sample_metadata_path: String,
    pub(crate) output_root: String,
    pub(crate) ibd_tsv_path: String,
    pub(crate) ibd_json_path: String,
    pub(crate) source_ibd_input_path: String,
    pub(crate) source_ibd_segments_path: String,
    pub(crate) source_ibd_merged_segments_path: String,
    pub(crate) source_ibd_filtered_segments_path: String,
    pub(crate) source_ibd_summary_path: String,
    pub(crate) source_ibd_metrics_path: String,
    pub(crate) source_logs_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) execution_mode: String,
    pub(crate) germline_tool_ok: bool,
    pub(crate) ibdhap_tool_ok: bool,
    pub(crate) pair_count: u64,
    pub(crate) rows: Vec<LocalVcfIbdSmokeRow>,
    pub(crate) insufficient_overlap_probe: LocalVcfIbdInsufficientOverlapProbe,
    pub(crate) status: String,
}

pub(crate) fn run_vcf_ibd_smoke(args: &parse::BenchLocalRunVcfIbdSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_ibd_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.ibd_json_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_ibd_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfIbdSmokeReport> {
    let contract = resolve_governed_vcf_ibd_smoke_contract(tool_id)?;
    let fixture_manifest_path = repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
    let fixture_manifest = load_vcf_corpus_fixture_manifest_path(&fixture_manifest_path)?;
    let fixture_dir = fixture_manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", fixture_manifest_path.display())
    })?;
    let input_vcf_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.multisample_vcf_path);
    let sample_metadata_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.sample_metadata_path);
    let expected_samples = fixture_manifest.expected_multisample_sample_ids.clone();
    if expected_samples.len() < 2 {
        bail!("governed VCF IBD smoke fixture must declare at least two cohort sample ids");
    }
    let sample_metadata = load_sample_metadata(&sample_metadata_source)?;
    validate_expected_cohort_samples(&sample_metadata, &expected_samples)?;

    let output_root = repo_root.join(DEFAULT_VCF_IBD_SMOKE_ROOT).join(&contract.tool_id);
    if output_root.exists() {
        fs::remove_dir_all(&output_root)
            .with_context(|| format!("remove {}", output_root.display()))?;
    }
    let artifacts_root = output_root.join("artifacts");
    let input_root = artifacts_root.join("input");
    let stage_root = artifacts_root.join("stage");
    let probe_root = artifacts_root.join("probe");
    fs::create_dir_all(&input_root).with_context(|| format!("create {}", input_root.display()))?;
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    fs::create_dir_all(&probe_root).with_context(|| format!("create {}", probe_root.display()))?;

    let input_vcf_path = input_root.join(DEFAULT_INPUT_VCF_NAME);
    fs::copy(&input_vcf_source, &input_vcf_path).with_context(|| {
        format!("copy {} to {}", input_vcf_source.display(), input_vcf_path.display())
    })?;
    let sample_metadata_path = input_root.join(DEFAULT_INPUT_SAMPLE_METADATA_NAME);
    fs::copy(&sample_metadata_source, &sample_metadata_path).with_context(|| {
        format!("copy {} to {}", sample_metadata_source.display(), sample_metadata_path.display())
    })?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_ibd_stage(
        &input_vcf_path,
        &stage_root,
        &IbdStageParams {
            toolchain: contract.tool_id.clone(),
            min_variant_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_samples: 2,
            min_segment_cm: 1.0,
            min_markers_per_segment: GOVERNED_IBD_SEGMENT_MARKER_MINIMUM,
            ..IbdStageParams::default()
        },
    )
    .with_context(|| format!("run governed VCF IBD smoke from {}", input_vcf_path.display()))?;

    let source_ibd_input_path = output_root.join(DEFAULT_OUTPUT_SOURCE_INPUT_NAME);
    fs::copy(&stage_outputs.ibd_input_tsv, &source_ibd_input_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.ibd_input_tsv.display(),
            source_ibd_input_path.display()
        )
    })?;
    let source_ibd_segments_path = output_root.join(DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME);
    fs::copy(&stage_outputs.ibd_segments_tsv, &source_ibd_segments_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.ibd_segments_tsv.display(),
            source_ibd_segments_path.display()
        )
    })?;
    let source_ibd_merged_segments_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_MERGED_SEGMENTS_NAME);
    fs::copy(&stage_outputs.ibd_merged_segments_tsv, &source_ibd_merged_segments_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.ibd_merged_segments_tsv.display(),
                source_ibd_merged_segments_path.display()
            )
        })?;
    let source_ibd_filtered_segments_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_FILTERED_SEGMENTS_NAME);
    fs::copy(&stage_outputs.ibd_filtered_segments_tsv, &source_ibd_filtered_segments_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.ibd_filtered_segments_tsv.display(),
                source_ibd_filtered_segments_path.display()
            )
        })?;
    let source_ibd_summary_path = output_root.join(DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME);
    fs::copy(&stage_outputs.ibd_summary_json, &source_ibd_summary_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.ibd_summary_json.display(),
            source_ibd_summary_path.display()
        )
    })?;
    let source_ibd_metrics_path = output_root.join(DEFAULT_OUTPUT_SOURCE_METRICS_NAME);
    fs::copy(&stage_outputs.ibd_metrics_json, &source_ibd_metrics_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.ibd_metrics_json.display(),
            source_ibd_metrics_path.display()
        )
    })?;
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    fs::copy(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let expected_sample_set = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    let filtered_segments =
        parse_ibd_filtered_segments(&source_ibd_filtered_segments_path, &expected_sample_set)?;
    let overlap_counts = compute_pair_overlap_marker_counts(&input_vcf_path, &expected_samples)?;
    let rows = summarize_ibd_rows(&filtered_segments, &overlap_counts)?;
    if rows.is_empty() {
        bail!("governed VCF IBD smoke expected at least one pair row");
    }

    let source_ibd_summary = read_json(&source_ibd_summary_path)?;
    let status = source_ibd_summary
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("IBD source summary missing `status`"))?
        .to_string();
    if status != "complete" {
        bail!("governed VCF IBD smoke expected `complete` status, found `{status}`");
    }
    let execution_mode = source_ibd_summary
        .get("execution_mode")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("IBD source summary missing `execution_mode`"))?
        .to_string();
    let germline_tool_ok = source_ibd_summary
        .pointer("/tool_attempts/germline")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("IBD source summary missing `tool_attempts.germline`"))?;
    let ibdhap_tool_ok = source_ibd_summary
        .pointer("/tool_attempts/ibdhap")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("IBD source summary missing `tool_attempts.ibdhap`"))?;

    let insufficient_overlap_probe = run_insufficient_overlap_probe(
        repo_root,
        &probe_root,
        &expected_samples[0],
        &expected_samples[1],
        &contract.tool_id,
    )?;

    let ibd_tsv_path = output_root.join(DEFAULT_OUTPUT_TSV_NAME);
    bijux_dna_infra::atomic_write_bytes(&ibd_tsv_path, build_ibd_tsv(&rows).as_bytes())?;
    let ibd_json_path = output_root.join(DEFAULT_OUTPUT_JSON_NAME);
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let pair_count = u64::try_from(rows.len()).map_err(|_| anyhow!("pair count overflow"))?;
    let report = LocalVcfIbdSmokeReport {
        schema_version: LOCAL_VCF_IBD_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_IBD_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        fixture_manifest_path: path_relative_to_repo(repo_root, &fixture_manifest_path),
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf_path),
        sample_metadata_path: path_relative_to_repo(repo_root, &sample_metadata_path),
        output_root: path_relative_to_repo(repo_root, &output_root),
        ibd_tsv_path: path_relative_to_repo(repo_root, &ibd_tsv_path),
        ibd_json_path: path_relative_to_repo(repo_root, &ibd_json_path),
        source_ibd_input_path: path_relative_to_repo(repo_root, &source_ibd_input_path),
        source_ibd_segments_path: path_relative_to_repo(repo_root, &source_ibd_segments_path),
        source_ibd_merged_segments_path: path_relative_to_repo(
            repo_root,
            &source_ibd_merged_segments_path,
        ),
        source_ibd_filtered_segments_path: path_relative_to_repo(
            repo_root,
            &source_ibd_filtered_segments_path,
        ),
        source_ibd_summary_path: path_relative_to_repo(repo_root, &source_ibd_summary_path),
        source_ibd_metrics_path: path_relative_to_repo(repo_root, &source_ibd_metrics_path),
        source_logs_path: path_relative_to_repo(repo_root, &source_logs_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at: started_at.clone(),
        finished_at: finished_at.clone(),
        elapsed_seconds,
        exit_code: 0,
        execution_mode,
        germline_tool_ok,
        ibdhap_tool_ok,
        pair_count,
        rows,
        insufficient_overlap_probe,
        status,
    };
    bijux_dna_infra::atomic_write_json(&ibd_json_path, &report)?;

    let stage_result_manifest = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: contract.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: contract.tool_id.clone() },
        command: BenchStageResultCommandV1 { rendered: report.command.clone() },
        runtime: BenchStageResultRuntimeV1 {
            mode: "local_smoke".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at,
            finished_at,
            elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs: vec![
            stage_result_output(
                repo_root,
                "ibd_tsv",
                DEFAULT_OUTPUT_TSV_NAME,
                &ibd_tsv_path,
                "table_output",
            ),
            stage_result_output(
                repo_root,
                "ibd_json",
                DEFAULT_OUTPUT_JSON_NAME,
                &ibd_json_path,
                "report_output",
            ),
            stage_result_output(
                repo_root,
                "source_ibd_input_tsv",
                DEFAULT_OUTPUT_SOURCE_INPUT_NAME,
                &source_ibd_input_path,
                "table_output",
            ),
            stage_result_output(
                repo_root,
                "source_ibd_segments_tsv",
                DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME,
                &source_ibd_segments_path,
                "table_output",
            ),
            stage_result_output(
                repo_root,
                "source_ibd_merged_segments_tsv",
                DEFAULT_OUTPUT_SOURCE_MERGED_SEGMENTS_NAME,
                &source_ibd_merged_segments_path,
                "table_output",
            ),
            stage_result_output(
                repo_root,
                "source_ibd_filtered_segments_tsv",
                DEFAULT_OUTPUT_SOURCE_FILTERED_SEGMENTS_NAME,
                &source_ibd_filtered_segments_path,
                "table_output",
            ),
            stage_result_output(
                repo_root,
                "source_ibd_summary_json",
                DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME,
                &source_ibd_summary_path,
                "report_output",
            ),
            stage_result_output(
                repo_root,
                "source_ibd_metrics_json",
                DEFAULT_OUTPUT_SOURCE_METRICS_NAME,
                &source_ibd_metrics_path,
                "report_output",
            ),
            stage_result_output(
                repo_root,
                "source_logs_txt",
                DEFAULT_OUTPUT_SOURCE_LOGS_NAME,
                &source_logs_path,
                "log_output",
            ),
            stage_result_output(
                repo_root,
                "probe_source_ibd_summary_json",
                DEFAULT_PROBE_SOURCE_SUMMARY_NAME,
                &repo_root.join(&report.insufficient_overlap_probe.source_ibd_summary_path),
                "report_output",
            ),
            stage_result_output(
                repo_root,
                "probe_source_ibd_filtered_segments_tsv",
                DEFAULT_PROBE_SOURCE_FILTERED_SEGMENTS_NAME,
                &repo_root
                    .join(&report.insufficient_overlap_probe.source_ibd_filtered_segments_path),
                "table_output",
            ),
            stage_result_output(
                repo_root,
                "probe_source_roh_summary_json",
                DEFAULT_PROBE_SOURCE_ROH_SUMMARY_NAME,
                &repo_root.join(&report.insufficient_overlap_probe.source_roh_summary_path),
                "report_output",
            ),
            stage_result_output(
                repo_root,
                "probe_source_roh_segments_tsv",
                DEFAULT_PROBE_SOURCE_ROH_SEGMENTS_NAME,
                &repo_root.join(&report.insufficient_overlap_probe.source_roh_segments_path),
                "table_output",
            ),
        ],
    };
    validate_stage_result_manifest(&stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    Ok(report)
}

fn resolve_governed_vcf_ibd_smoke_contract(tool_id: &str) -> Result<GovernedVcfIbdSmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_IBD_STAGE_ID)
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_IBD_STAGE_ID}`"))?;
    if tool_id != matrix_row.tool_id {
        bail!(
            "VCF IBD smoke only retains tool `{}` for `{}`; requested `{tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_IBD_CORPUS_ID {
        bail!(
            "VCF IBD smoke requires corpus `{GOVERNED_VCF_IBD_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_IBD_ASSET_PROFILE_ID {
        bail!(
            "VCF IBD smoke requires asset profile `{GOVERNED_VCF_IBD_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["ibd_segments".to_string()] {
        bail!(
            "VCF IBD smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfIbdSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_IBD_INPUT_FIXTURE_ID.to_string(),
    })
}

fn validate_expected_cohort_samples(
    sample_metadata: &[SampleMetadataRow],
    expected_samples: &[String],
) -> Result<()> {
    let expected_sample_set = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::<String>::new();
    for row in sample_metadata {
        if !expected_sample_set.contains(&row.sample_id) {
            continue;
        }
        if row.role != "cohort" {
            bail!(
                "governed VCF IBD smoke expected cohort role for `{}`, found `{}`",
                row.sample_id,
                row.role
            );
        }
        if !seen.insert(row.sample_id.clone()) {
            bail!("governed VCF IBD smoke metadata duplicated sample `{}`", row.sample_id);
        }
    }
    for sample_id in expected_samples {
        if !seen.contains(sample_id) {
            bail!("governed VCF IBD smoke metadata is missing expected sample `{sample_id}`");
        }
    }
    Ok(())
}

fn parse_ibd_filtered_segments(
    path: &Path,
    expected_samples: &BTreeSet<String>,
) -> Result<Vec<ParsedIbdSegment>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut seen_header = false;
    let mut rows = Vec::<ParsedIbdSegment>::new();
    for (index, line) in raw.lines().enumerate() {
        if line.starts_with('#') {
            continue;
        }
        if !seen_header {
            if line.trim() != "sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\tmarker_count" {
                bail!("IBD filtered-segments header drifted in {}: `{line}`", path.display());
            }
            seen_header = true;
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 7 {
            bail!(
                "IBD filtered-segments row {} in {} must have 7 columns",
                index + 1,
                path.display()
            );
        }
        let sample_a = columns[0].trim().to_string();
        let sample_b = columns[1].trim().to_string();
        if !expected_samples.contains(&sample_a) || !expected_samples.contains(&sample_b) {
            bail!("IBD filtered-segments table reported unexpected sample pair `{sample_a}` / `{sample_b}`");
        }
        let contig = columns[2].trim().to_string();
        let start = columns[3]
            .trim()
            .parse::<u64>()
            .with_context(|| format!("parse start from row {} in {}", index + 1, path.display()))?;
        let end = columns[4]
            .trim()
            .parse::<u64>()
            .with_context(|| format!("parse end from row {} in {}", index + 1, path.display()))?;
        let length_cm = columns[5].trim().parse::<f64>().with_context(|| {
            format!("parse length_cm from row {} in {}", index + 1, path.display())
        })?;
        let marker_count = columns[6].trim().parse::<u64>().with_context(|| {
            format!("parse marker_count from row {} in {}", index + 1, path.display())
        })?;
        rows.push(ParsedIbdSegment {
            sample_a,
            sample_b,
            contig,
            start,
            end,
            length_cm,
            marker_count,
        });
    }
    Ok(rows)
}

fn summarize_ibd_rows(
    segments: &[ParsedIbdSegment],
    overlap_counts: &BTreeMap<(String, String), u64>,
) -> Result<Vec<LocalVcfIbdSmokeRow>> {
    let mut totals = BTreeMap::<(String, String), (u64, f64)>::new();
    for segment in segments {
        let key = ordered_pair(&segment.sample_a, &segment.sample_b);
        let entry = totals.entry(key).or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += segment.length_cm;
    }
    let mut rows = Vec::<LocalVcfIbdSmokeRow>::new();
    for ((sample_a, sample_b), (segment_count, total_length)) in totals {
        let overlap_marker_count =
            overlap_counts.get(&(sample_a.clone(), sample_b.clone())).copied().ok_or_else(
                || anyhow!("IBD overlap-count derivation missed pair `{sample_a}` / `{sample_b}`"),
            )?;
        rows.push(LocalVcfIbdSmokeRow {
            sample_a,
            sample_b,
            segment_count,
            total_length,
            overlap_marker_count,
            status: "complete".to_string(),
        });
    }
    Ok(rows)
}

fn compute_pair_overlap_marker_counts(
    input_vcf: &Path,
    expected_samples: &[String],
) -> Result<BTreeMap<(String, String), u64>> {
    let raw =
        fs::read_to_string(input_vcf).with_context(|| format!("read {}", input_vcf.display()))?;
    let sample_ids = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .map(|line| line.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>())
        .ok_or_else(|| {
            anyhow!("IBD input VCF is missing sample header: {}", input_vcf.display())
        })?;
    if sample_ids != expected_samples {
        bail!(
            "IBD overlap-count derivation expected samples {:?}, found {:?}",
            expected_samples,
            sample_ids
        );
    }
    let mut counts = BTreeMap::<(String, String), u64>::new();
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let Some(gt_index) = parse_format_index(&fields, "GT") else {
            continue;
        };
        for sample_a_index in 0..sample_ids.len() {
            for sample_b_index in (sample_a_index + 1)..sample_ids.len() {
                let sample_a = fields.get(9 + sample_a_index).copied().unwrap_or_default();
                let sample_b = fields.get(9 + sample_b_index).copied().unwrap_or_default();
                if !genotype_is_called(sample_a, gt_index)
                    || !genotype_is_called(sample_b, gt_index)
                {
                    continue;
                }
                let key = ordered_pair(&sample_ids[sample_a_index], &sample_ids[sample_b_index]);
                *counts.entry(key).or_insert(0) += 1;
            }
        }
    }
    Ok(counts)
}

fn genotype_is_called(sample_field: &str, gt_index: usize) -> bool {
    sample_field
        .split(':')
        .nth(gt_index)
        .map(|gt| !gt.trim().is_empty() && gt != "." && gt != "./." && gt != ".|.")
        .unwrap_or(false)
}

fn ordered_pair(sample_a: &str, sample_b: &str) -> (String, String) {
    if sample_a <= sample_b {
        (sample_a.to_string(), sample_b.to_string())
    } else {
        (sample_b.to_string(), sample_a.to_string())
    }
}

fn run_insufficient_overlap_probe(
    repo_root: &Path,
    probe_root: &Path,
    sample_a: &str,
    sample_b: &str,
    tool_id: &str,
) -> Result<LocalVcfIbdInsufficientOverlapProbe> {
    let input_vcf_path = probe_root.join(DEFAULT_PROBE_INPUT_VCF_NAME);
    write_probe_sparse_overlap_vcf(&input_vcf_path, sample_a, sample_b)?;
    let ibd_stage_root = probe_root.join("ibd_stage");
    let roh_stage_root = probe_root.join("roh_stage");
    fs::create_dir_all(&ibd_stage_root)
        .with_context(|| format!("create {}", ibd_stage_root.display()))?;
    fs::create_dir_all(&roh_stage_root)
        .with_context(|| format!("create {}", roh_stage_root.display()))?;

    let ibd_outputs = run_ibd_stage(
        &input_vcf_path,
        &ibd_stage_root,
        &IbdStageParams {
            toolchain: tool_id.to_string(),
            min_variant_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_samples: 2,
            min_segment_cm: 1.0,
            min_markers_per_segment: PROBE_IBD_SEGMENT_MARKER_MINIMUM,
            ..IbdStageParams::default()
        },
    )
    .with_context(|| format!("run sparse-overlap IBD probe from {}", input_vcf_path.display()))?;
    let roh_outputs = run_roh_stage(
        &input_vcf_path,
        &roh_stage_root,
        &RohStageParams {
            toolchain: "plink2".to_string(),
            min_snp_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_segment_kb: 0,
            max_gap_bp: 10_000_000,
            ..RohStageParams::default()
        },
    )
    .with_context(|| format!("run sparse-overlap ROH probe from {}", input_vcf_path.display()))?;

    let source_ibd_summary_path = probe_root.join(DEFAULT_PROBE_SOURCE_SUMMARY_NAME);
    fs::copy(&ibd_outputs.ibd_summary_json, &source_ibd_summary_path).with_context(|| {
        format!(
            "copy {} to {}",
            ibd_outputs.ibd_summary_json.display(),
            source_ibd_summary_path.display()
        )
    })?;
    let source_ibd_filtered_segments_path =
        probe_root.join(DEFAULT_PROBE_SOURCE_FILTERED_SEGMENTS_NAME);
    fs::copy(&ibd_outputs.ibd_filtered_segments_tsv, &source_ibd_filtered_segments_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                ibd_outputs.ibd_filtered_segments_tsv.display(),
                source_ibd_filtered_segments_path.display()
            )
        })?;
    let source_roh_summary_path = probe_root.join(DEFAULT_PROBE_SOURCE_ROH_SUMMARY_NAME);
    fs::copy(&roh_outputs.roh_summary_json, &source_roh_summary_path).with_context(|| {
        format!(
            "copy {} to {}",
            roh_outputs.roh_summary_json.display(),
            source_roh_summary_path.display()
        )
    })?;
    let source_roh_segments_path = probe_root.join(DEFAULT_PROBE_SOURCE_ROH_SEGMENTS_NAME);
    fs::copy(&roh_outputs.roh_segments_tsv, &source_roh_segments_path).with_context(|| {
        format!(
            "copy {} to {}",
            roh_outputs.roh_segments_tsv.display(),
            source_roh_segments_path.display()
        )
    })?;

    let ibd_summary = read_json(&source_ibd_summary_path)?;
    let ibd_status = ibd_summary
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("sparse-overlap IBD summary missing `status`"))?
        .to_string();
    if ibd_status != "insufficient_marker_overlap" {
        bail!(
            "sparse-overlap IBD probe expected `insufficient_marker_overlap`, found `{ibd_status}`"
        );
    }
    let insufficient_data_reason = ibd_summary
        .get("insufficient_data_reason")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("sparse-overlap IBD summary missing `insufficient_data_reason`"))?
        .to_string();
    let filtered_segment_count = ibd_summary
        .get("segments_filtered")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("sparse-overlap IBD summary missing `segments_filtered`"))?;
    if filtered_segment_count != 0 {
        bail!("sparse-overlap IBD probe expected zero filtered segments, found {filtered_segment_count}");
    }

    let overlap_counts = compute_pair_overlap_marker_counts(
        &input_vcf_path,
        &[sample_a.to_string(), sample_b.to_string()],
    )?;
    let overlap_marker_count =
        overlap_counts.get(&ordered_pair(sample_a, sample_b)).copied().unwrap_or(0);

    let roh_summary = read_json(&source_roh_summary_path)?;
    let unrelated_stage_segment_count = roh_summary
        .get("segment_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("sparse-overlap ROH summary missing `segment_count`"))?;
    if unrelated_stage_segment_count == 0 {
        bail!("sparse-overlap probe expected ROH to remain productive");
    }

    Ok(LocalVcfIbdInsufficientOverlapProbe {
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf_path),
        ibd_status,
        insufficient_data_reason,
        overlap_marker_count,
        filtered_segment_count,
        source_ibd_summary_path: path_relative_to_repo(repo_root, &source_ibd_summary_path),
        source_ibd_filtered_segments_path: path_relative_to_repo(
            repo_root,
            &source_ibd_filtered_segments_path,
        ),
        unrelated_stage_id: "vcf.roh".to_string(),
        unrelated_stage_status: "complete".to_string(),
        unrelated_stage_segment_count,
        source_roh_summary_path: path_relative_to_repo(repo_root, &source_roh_summary_path),
        source_roh_segments_path: path_relative_to_repo(repo_root, &source_roh_segments_path),
    })
}

fn build_ibd_tsv(rows: &[LocalVcfIbdSmokeRow]) -> String {
    let mut rendered = String::from(
        "sample_a\tsample_b\tsegment_count\ttotal_length\toverlap_marker_count\tstatus\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{:.3}\t{}\t{}\n",
            row.sample_a,
            row.sample_b,
            row.segment_count,
            row.total_length,
            row.overlap_marker_count,
            row.status
        ));
    }
    rendered
}

fn stage_result_output(
    repo_root: &Path,
    artifact_id: &str,
    declared_path: &str,
    realized_path: &Path,
    role: &str,
) -> BenchStageResultOutputV1 {
    BenchStageResultOutputV1 {
        artifact_id: artifact_id.to_string(),
        declared_path: declared_path.to_string(),
        realized_path: path_relative_to_repo(repo_root, realized_path),
        role: role.to_string(),
        optional: false,
        exists: true,
    }
}

fn write_probe_sparse_overlap_vcf(path: &Path, sample_a: &str, sample_b: &str) -> Result<()> {
    let raw = format!(
        "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{sample_a}\t{sample_b}\nchr1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\t0/0\nchr1\t200\t.\tC\tT\t60\tPASS\t.\tGT\t0/1\t./.\nchr1\t300\t.\tG\tA\t60\tPASS\t.\tGT\t./.\t0/1\n"
    );
    bijux_dna_infra::atomic_write_bytes(path, raw.as_bytes())?;
    Ok(())
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn parse_record_fields(line: &str) -> Option<Vec<&str>> {
    if line.trim().is_empty() || line.starts_with('#') {
        None
    } else {
        Some(line.split('\t').collect())
    }
}

fn parse_format_index(fields: &[&str], name: &str) -> Option<usize> {
    fields.get(8)?.split(':').position(|token| token == name)
}

fn resolve_manifest_relative_path(manifest_dir: &Path, path: &Path) -> std::path::PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

fn timestamp_marker() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0));
    format!("{}", now.as_secs())
}

#[cfg(test)]
mod tests {
    use super::{
        compute_pair_overlap_marker_counts, resolve_governed_vcf_ibd_smoke_contract,
        run_local_vcf_ibd_smoke, write_probe_sparse_overlap_vcf,
    };

    #[test]
    fn governed_vcf_ibd_smoke_contract_matches_stage_matrix() {
        let contract =
            resolve_governed_vcf_ibd_smoke_contract("germline").expect("resolve contract");
        assert_eq!(contract.stage_id, "vcf.ibd");
        assert_eq!(contract.tool_id, "germline");
        assert_eq!(contract.corpus_id, "vcf_production_regression");
        assert_eq!(contract.input_fixture_id, "vcf_mini_multisample_cohort");
    }

    #[test]
    fn pair_overlap_counts_track_called_markers_only() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("probe.vcf");
        write_probe_sparse_overlap_vcf(&path, "sample_a", "sample_b").expect("write probe");
        let counts = compute_pair_overlap_marker_counts(
            &path,
            &["sample_a".to_string(), "sample_b".to_string()],
        )
        .expect("pair overlap counts");
        assert_eq!(counts.get(&("sample_a".to_string(), "sample_b".to_string())).copied(), Some(1));
    }

    #[test]
    fn governed_vcf_ibd_smoke_reports_pair_rows_and_localized_probe() {
        let repo_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).expect("repo root");
        let report = run_local_vcf_ibd_smoke(repo_root, "germline").expect("run local ibd smoke");
        assert_eq!(report.stage_id, "vcf.ibd");
        assert_eq!(report.tool_id, "germline");
        assert_eq!(report.corpus_id, "vcf_production_regression");
        assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
        assert_eq!(report.status, "complete");
        assert!(!report.rows.is_empty(), "expected at least one IBD pair row");
        assert!(report.rows.iter().all(|row| row.segment_count > 0
            && row.overlap_marker_count > 0
            && row.status == "complete"));
        assert_eq!(report.insufficient_overlap_probe.ibd_status, "insufficient_marker_overlap");
        assert_eq!(report.insufficient_overlap_probe.unrelated_stage_id, "vcf.roh");
        assert_eq!(report.insufficient_overlap_probe.unrelated_stage_status, "complete");
        assert!(
            report.insufficient_overlap_probe.unrelated_stage_segment_count > 0,
            "expected sparse-overlap probe to keep ROH productive"
        );
    }
}
