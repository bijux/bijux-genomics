use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_stages_vcf::pipeline::{run_roh_stage, RohStageParams};
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

const DEFAULT_VCF_ROH_SMOKE_ROOT: &str = "runs/bench/local-smoke/vcf.roh";
const LOCAL_VCF_ROH_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_roh_smoke.v1";
const LOCAL_VCF_ROH_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-roh-smoke";
const GOVERNED_VCF_ROH_STAGE_ID: &str = "vcf.roh";
const GOVERNED_VCF_ROH_TOOL_ID: &str = "plink2";
const GOVERNED_VCF_ROH_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_ROH_ASSET_PROFILE_ID: &str = "vcf_cohort";
const GOVERNED_VCF_ROH_INPUT_FIXTURE_ID: &str = "vcf_mini_multisample_cohort";
const DEFAULT_INPUT_VCF_NAME: &str = "roh_input.vcf";
const DEFAULT_INPUT_SAMPLE_METADATA_NAME: &str = "sample_metadata.tsv";
const DEFAULT_OUTPUT_TSV_NAME: &str = "roh.tsv";
const DEFAULT_OUTPUT_JSON_NAME: &str = "roh.json";
const DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME: &str = "source_roh_segments.tsv";
const DEFAULT_OUTPUT_SOURCE_PER_SAMPLE_NAME: &str = "source_roh_per_sample.tsv";
const DEFAULT_OUTPUT_SOURCE_ROH_REPORT_NAME: &str = "source_roh.json";
const DEFAULT_OUTPUT_SOURCE_METRICS_NAME: &str = "source_metrics.json";
const DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME: &str = "source_roh_summary.json";
const DEFAULT_OUTPUT_SOURCE_ROH_METRICS_NAME: &str = "source_roh_metrics.json";
const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfRohSmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfRohSmokeSegment {
    pub(crate) sample_id: String,
    pub(crate) contig: String,
    pub(crate) start: u64,
    pub(crate) end: u64,
    pub(crate) length: u64,
    pub(crate) variant_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfRohSmokePerSampleSummary {
    pub(crate) sample_id: String,
    pub(crate) segment_count: u64,
    pub(crate) total_length: u64,
    pub(crate) mean_length: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfRohSmokeReport {
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
    pub(crate) roh_tsv_path: String,
    pub(crate) roh_json_path: String,
    pub(crate) source_roh_segments_path: String,
    pub(crate) source_roh_per_sample_path: String,
    pub(crate) source_roh_report_path: String,
    pub(crate) source_metrics_path: String,
    pub(crate) source_roh_summary_path: String,
    pub(crate) source_roh_metrics_path: String,
    pub(crate) source_logs_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) execution_mode: String,
    pub(crate) tool_ok: bool,
    pub(crate) sample_count: u64,
    pub(crate) segment_count: u64,
    pub(crate) total_length: u64,
    pub(crate) segments: Vec<LocalVcfRohSmokeSegment>,
    pub(crate) per_sample_summary: Vec<LocalVcfRohSmokePerSampleSummary>,
    pub(crate) status: String,
}

pub(crate) fn run_vcf_roh_smoke(args: &parse::BenchLocalRunVcfRohSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_roh_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.roh_json_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_roh_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfRohSmokeReport> {
    let contract = resolve_governed_vcf_roh_smoke_contract(tool_id)?;
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
    if expected_samples.is_empty() {
        bail!("governed VCF ROH smoke fixture must declare multisample cohort sample ids");
    }
    let sample_metadata = load_sample_metadata(&sample_metadata_source)?;
    validate_expected_cohort_samples(&sample_metadata, &expected_samples)?;

    let output_root = repo_root.join(DEFAULT_VCF_ROH_SMOKE_ROOT).join(&contract.tool_id);
    if output_root.exists() {
        fs::remove_dir_all(&output_root)
            .with_context(|| format!("remove {}", output_root.display()))?;
    }
    let artifacts_root = output_root.join("artifacts");
    let input_root = artifacts_root.join("input");
    let stage_root = artifacts_root.join("stage");
    fs::create_dir_all(&input_root).with_context(|| format!("create {}", input_root.display()))?;
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;

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
    let stage_outputs = run_roh_stage(
        &input_vcf_path,
        &stage_root,
        &RohStageParams {
            toolchain: contract.tool_id.clone(),
            min_snp_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_segment_kb: 0,
            max_gap_bp: 10_000_000,
            ..RohStageParams::default()
        },
    )
    .with_context(|| format!("run governed VCF ROH smoke from {}", input_vcf_path.display()))?;

    let source_roh_segments_path = output_root.join(DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME);
    fs::copy(&stage_outputs.roh_segments_tsv, &source_roh_segments_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.roh_segments_tsv.display(),
            source_roh_segments_path.display()
        )
    })?;
    let source_roh_per_sample_path = output_root.join(DEFAULT_OUTPUT_SOURCE_PER_SAMPLE_NAME);
    fs::copy(&stage_outputs.roh_per_sample_tsv, &source_roh_per_sample_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.roh_per_sample_tsv.display(),
                source_roh_per_sample_path.display()
            )
        },
    )?;
    let source_roh_report_path = output_root.join(DEFAULT_OUTPUT_SOURCE_ROH_REPORT_NAME);
    fs::copy(&stage_outputs.roh_json, &source_roh_report_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.roh_json.display(), source_roh_report_path.display())
    })?;
    let source_metrics_path = output_root.join(DEFAULT_OUTPUT_SOURCE_METRICS_NAME);
    fs::copy(&stage_outputs.metrics_json, &source_metrics_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.metrics_json.display(),
            source_metrics_path.display()
        )
    })?;
    let source_roh_summary_path = output_root.join(DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME);
    fs::copy(&stage_outputs.roh_summary_json, &source_roh_summary_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.roh_summary_json.display(),
            source_roh_summary_path.display()
        )
    })?;
    let source_roh_metrics_path = output_root.join(DEFAULT_OUTPUT_SOURCE_ROH_METRICS_NAME);
    fs::copy(&stage_outputs.roh_metrics_json, &source_roh_metrics_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.roh_metrics_json.display(),
            source_roh_metrics_path.display()
        )
    })?;
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    fs::copy(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let expected_sample_set = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    let segments = parse_roh_segments(&source_roh_segments_path, &expected_sample_set)?;
    let per_sample_summary =
        parse_roh_per_sample_summary(&source_roh_per_sample_path, &expected_samples)?;
    let source_roh_report = read_json(&source_roh_report_path)?;
    validate_roh_summary_contract(&source_roh_report, &segments, &per_sample_summary)?;

    let execution_mode = source_roh_report
        .get("execution_mode")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("ROH source report missing `execution_mode`"))?
        .to_string();
    let tool_ok = source_roh_report
        .pointer("/tool_attempts/plink2_homozyg")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("ROH source report missing `tool_attempts.plink2_homozyg`"))?;
    let segment_count =
        u64::try_from(segments.len()).map_err(|_| anyhow!("segment count overflow"))?;
    let sample_count =
        u64::try_from(per_sample_summary.len()).map_err(|_| anyhow!("sample count overflow"))?;
    let total_length = segments.iter().map(|segment| segment.length).sum::<u64>();

    let roh_tsv_path = output_root.join(DEFAULT_OUTPUT_TSV_NAME);
    bijux_dna_infra::atomic_write_bytes(&roh_tsv_path, build_roh_tsv(&segments).as_bytes())?;
    let roh_json_path = output_root.join(DEFAULT_OUTPUT_JSON_NAME);
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let report = LocalVcfRohSmokeReport {
        schema_version: LOCAL_VCF_ROH_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_ROH_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        fixture_manifest_path: path_relative_to_repo(repo_root, &fixture_manifest_path),
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf_path),
        sample_metadata_path: path_relative_to_repo(repo_root, &sample_metadata_path),
        output_root: path_relative_to_repo(repo_root, &output_root),
        roh_tsv_path: path_relative_to_repo(repo_root, &roh_tsv_path),
        roh_json_path: path_relative_to_repo(repo_root, &roh_json_path),
        source_roh_segments_path: path_relative_to_repo(repo_root, &source_roh_segments_path),
        source_roh_per_sample_path: path_relative_to_repo(repo_root, &source_roh_per_sample_path),
        source_roh_report_path: path_relative_to_repo(repo_root, &source_roh_report_path),
        source_metrics_path: path_relative_to_repo(repo_root, &source_metrics_path),
        source_roh_summary_path: path_relative_to_repo(repo_root, &source_roh_summary_path),
        source_roh_metrics_path: path_relative_to_repo(repo_root, &source_roh_metrics_path),
        source_logs_path: path_relative_to_repo(repo_root, &source_logs_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at: started_at.clone(),
        finished_at: finished_at.clone(),
        elapsed_seconds,
        exit_code: 0,
        execution_mode,
        tool_ok,
        sample_count,
        segment_count,
        total_length,
        segments,
        per_sample_summary,
        status: "complete".to_string(),
    };
    bijux_dna_infra::atomic_write_json(&roh_json_path, &report)?;

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
            BenchStageResultOutputV1 {
                artifact_id: "roh_tsv".to_string(),
                declared_path: DEFAULT_OUTPUT_TSV_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &roh_tsv_path),
                role: "table_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "roh_json".to_string(),
                declared_path: DEFAULT_OUTPUT_JSON_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &roh_json_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_roh_segments_tsv".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_roh_segments_path),
                role: "table_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_roh_per_sample_tsv".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_PER_SAMPLE_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_roh_per_sample_path),
                role: "table_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_roh_report_json".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_ROH_REPORT_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_roh_report_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_metrics_json".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_METRICS_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_metrics_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_roh_summary_json".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_roh_summary_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_roh_metrics_json".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_ROH_METRICS_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_roh_metrics_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_logs_txt".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_LOGS_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_logs_path),
                role: "log_output".to_string(),
                optional: false,
                exists: true,
            },
        ],
    };
    validate_stage_result_manifest(&stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    Ok(report)
}

fn resolve_governed_vcf_roh_smoke_contract(tool_id: &str) -> Result<GovernedVcfRohSmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_ROH_STAGE_ID)
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_ROH_STAGE_ID}`"))?;
    if tool_id != matrix_row.tool_id {
        bail!(
            "VCF ROH smoke only retains tool `{}` for `{}`; requested `{tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_ROH_CORPUS_ID {
        bail!(
            "VCF ROH smoke requires corpus `{GOVERNED_VCF_ROH_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_ROH_ASSET_PROFILE_ID {
        bail!(
            "VCF ROH smoke requires asset profile `{GOVERNED_VCF_ROH_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["roh_report".to_string()] {
        bail!(
            "VCF ROH smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfRohSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_ROH_INPUT_FIXTURE_ID.to_string(),
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
                "governed VCF ROH smoke expected cohort role for `{}`, found `{}`",
                row.sample_id,
                row.role
            );
        }
        if !seen.insert(row.sample_id.clone()) {
            bail!("governed VCF ROH smoke metadata duplicated sample `{}`", row.sample_id);
        }
    }
    for sample_id in expected_samples {
        if !seen.contains(sample_id) {
            bail!("governed VCF ROH smoke metadata is missing expected sample `{sample_id}`");
        }
    }
    Ok(())
}

fn parse_roh_segments(
    path: &Path,
    expected_samples: &BTreeSet<String>,
) -> Result<Vec<LocalVcfRohSmokeSegment>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header =
        lines.next().ok_or_else(|| anyhow!("ROH segments table is empty: {}", path.display()))?;
    if header.trim() != "sample\tcontig\tstart\tend\tlength_bp\tn_sites" {
        bail!("ROH segments header drifted in {}: `{header}`", path.display());
    }
    let mut segments = Vec::<LocalVcfRohSmokeSegment>::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 6 {
            bail!("ROH segments row {} in {} must have 6 columns", index + 2, path.display());
        }
        let sample_id = columns[0].trim().to_string();
        if !expected_samples.contains(&sample_id) {
            bail!("ROH segments table reported unexpected sample `{sample_id}`");
        }
        let contig = columns[1].trim().to_string();
        if contig.is_empty() {
            bail!(
                "ROH segments row {} in {} must keep a non-empty contig",
                index + 2,
                path.display()
            );
        }
        let start = columns[2].trim().parse::<u64>().with_context(|| {
            format!("parse ROH segment start from row {} in {}", index + 2, path.display())
        })?;
        let end = columns[3].trim().parse::<u64>().with_context(|| {
            format!("parse ROH segment end from row {} in {}", index + 2, path.display())
        })?;
        let length = columns[4].trim().parse::<u64>().with_context(|| {
            format!("parse ROH segment length from row {} in {}", index + 2, path.display())
        })?;
        let variant_count = columns[5].trim().parse::<u64>().with_context(|| {
            format!("parse ROH segment variant count from row {} in {}", index + 2, path.display())
        })?;
        if end < start {
            bail!("ROH segments row {} in {} has end before start", index + 2, path.display());
        }
        if length == 0 {
            bail!("ROH segments row {} in {} must keep positive length", index + 2, path.display());
        }
        if variant_count == 0 {
            bail!(
                "ROH segments row {} in {} must keep positive variant count",
                index + 2,
                path.display()
            );
        }
        segments.push(LocalVcfRohSmokeSegment {
            sample_id,
            contig,
            start,
            end,
            length,
            variant_count,
        });
    }
    Ok(segments)
}

fn parse_roh_per_sample_summary(
    path: &Path,
    expected_samples: &[String],
) -> Result<Vec<LocalVcfRohSmokePerSampleSummary>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow!("ROH per-sample summary is empty: {}", path.display()))?;
    if header.trim() != "sample\tsegment_count\ttotal_length_bp\tmean_length_bp" {
        bail!("ROH per-sample header drifted in {}: `{header}`", path.display());
    }
    let mut rows = Vec::<LocalVcfRohSmokePerSampleSummary>::new();
    let mut seen = BTreeSet::<String>::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 4 {
            bail!("ROH per-sample row {} in {} must have 4 columns", index + 2, path.display());
        }
        let sample_id = columns[0].trim().to_string();
        if !expected_samples.iter().any(|expected| expected == &sample_id) {
            bail!("ROH per-sample summary reported unexpected sample `{sample_id}`");
        }
        if !seen.insert(sample_id.clone()) {
            bail!("ROH per-sample summary duplicated sample `{sample_id}`");
        }
        let segment_count = columns[1].trim().parse::<u64>().with_context(|| {
            format!("parse ROH segment count from row {} in {}", index + 2, path.display())
        })?;
        let total_length = columns[2].trim().parse::<u64>().with_context(|| {
            format!("parse ROH total length from row {} in {}", index + 2, path.display())
        })?;
        let mean_length = columns[3].trim().parse::<f64>().with_context(|| {
            format!("parse ROH mean length from row {} in {}", index + 2, path.display())
        })?;
        rows.push(LocalVcfRohSmokePerSampleSummary {
            sample_id,
            segment_count,
            total_length,
            mean_length,
        });
    }
    if rows.len() != expected_samples.len() {
        bail!(
            "ROH per-sample summary expected {} rows, found {}",
            expected_samples.len(),
            rows.len()
        );
    }
    let observed = rows.iter().map(|row| row.sample_id.clone()).collect::<BTreeSet<_>>();
    let expected = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    if observed != expected {
        bail!("ROH per-sample summary drifted from governed sample contract");
    }
    Ok(rows)
}

fn validate_roh_summary_contract(
    source_roh_report: &serde_json::Value,
    segments: &[LocalVcfRohSmokeSegment],
    per_sample_summary: &[LocalVcfRohSmokePerSampleSummary],
) -> Result<()> {
    let segment_count =
        u64::try_from(segments.len()).map_err(|_| anyhow!("segment count overflow"))?;
    let total_length = segments.iter().map(|segment| segment.length).sum::<u64>();
    let reported_segment_count = source_roh_report
        .get("segment_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("ROH source report missing `segment_count`"))?;
    let reported_total_length = source_roh_report
        .get("total_length_bp")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("ROH source report missing `total_length_bp`"))?;
    if reported_segment_count != segment_count {
        bail!(
            "ROH source report segment count drifted: expected {segment_count}, found {reported_segment_count}"
        );
    }
    if reported_total_length != total_length {
        bail!(
            "ROH source report total length drifted: expected {total_length}, found {reported_total_length}"
        );
    }

    let mut derived = BTreeMap::<String, (u64, u64)>::new();
    for segment in segments {
        let entry = derived.entry(segment.sample_id.clone()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += segment.length;
    }
    for summary in per_sample_summary {
        let (segment_count, total_length) =
            derived.get(&summary.sample_id).copied().unwrap_or((0, 0));
        if summary.segment_count != segment_count {
            bail!(
                "ROH per-sample summary segment count drifted for `{}`: expected {}, found {}",
                summary.sample_id,
                segment_count,
                summary.segment_count
            );
        }
        if summary.total_length != total_length {
            bail!(
                "ROH per-sample summary total length drifted for `{}`: expected {}, found {}",
                summary.sample_id,
                total_length,
                summary.total_length
            );
        }
    }
    Ok(())
}

fn build_roh_tsv(segments: &[LocalVcfRohSmokeSegment]) -> String {
    let mut rendered = String::from("sample_id\tcontig\tstart\tend\tlength\tvariant_count\n");
    for segment in segments {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            segment.sample_id,
            segment.contig,
            segment.start,
            segment.end,
            segment.length,
            segment.variant_count
        ));
    }
    rendered
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
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
        parse_roh_per_sample_summary, parse_roh_segments, resolve_governed_vcf_roh_smoke_contract,
        run_local_vcf_roh_smoke,
    };

    #[test]
    fn governed_vcf_roh_smoke_contract_matches_stage_matrix() {
        let contract = resolve_governed_vcf_roh_smoke_contract("plink2").expect("resolve contract");
        assert_eq!(contract.stage_id, "vcf.roh");
        assert_eq!(contract.tool_id, "plink2");
        assert_eq!(contract.corpus_id, "vcf_production_regression");
        assert_eq!(contract.input_fixture_id, "vcf_mini_multisample_cohort");
    }

    #[test]
    fn roh_segments_parser_rejects_unexpected_samples() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("segments.tsv");
        std::fs::write(
            &path,
            "sample\tcontig\tstart\tend\tlength_bp\tn_sites\npanel_ref_1\tchr1\t10\t20\t10\t2\n",
        )
        .expect("write segments");
        let err = parse_roh_segments(&path, &["sample_a".to_string()].into_iter().collect())
            .expect_err("unexpected sample");
        assert!(err.to_string().contains("unexpected sample"));
    }

    #[test]
    fn roh_per_sample_parser_rejects_duplicate_samples() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("per-sample.tsv");
        std::fs::write(
            &path,
            "sample\tsegment_count\ttotal_length_bp\tmean_length_bp\nsample_a\t1\t20\t20.0\nsample_a\t0\t0\t0.0\n",
        )
        .expect("write per-sample");
        let err = parse_roh_per_sample_summary(&path, &["sample_a".to_string()])
            .expect_err("duplicate sample");
        assert!(err.to_string().contains("duplicated sample"));
    }

    #[test]
    fn governed_vcf_roh_smoke_reports_segments_and_per_sample_summary() {
        let repo_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).expect("repo root");
        let report = run_local_vcf_roh_smoke(repo_root, "plink2").expect("run local roh smoke");
        assert_eq!(report.stage_id, "vcf.roh");
        assert_eq!(report.tool_id, "plink2");
        assert_eq!(report.corpus_id, "vcf_production_regression");
        assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
        assert_eq!(report.status, "complete");
        assert_eq!(report.sample_count, 4);
        assert_eq!(report.per_sample_summary.len(), 4);
        assert_eq!(
            report.per_sample_summary.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(),
            vec!["sample_a", "sample_b", "sample_c", "sample_d"]
        );
        assert_eq!(
            report.per_sample_summary.iter().map(|row| row.segment_count).sum::<u64>(),
            report.segment_count
        );
        assert_eq!(
            report.per_sample_summary.iter().map(|row| row.total_length).sum::<u64>(),
            report.total_length
        );
        assert!(report
            .segments
            .iter()
            .all(|segment| !segment.sample_id.is_empty() && segment.variant_count > 0));
    }
}
