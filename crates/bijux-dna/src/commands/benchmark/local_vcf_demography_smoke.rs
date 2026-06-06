use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_stages_vcf::pipeline::{run_demography_stage, DemographyStageParams};
use serde::Serialize;

use super::local_stage_result_manifest::{
    path_relative_to_repo, validate_stage_result_manifest, BenchStageResultCommandV1,
    BenchStageResultManifestV1, BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::local_vcf_ibd_smoke::run_local_vcf_ibd_smoke;
use super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_DEMOGRAPHY_SMOKE_ROOT: &str = "target/local-smoke/vcf.demography";
const LOCAL_VCF_DEMOGRAPHY_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_demography_smoke.v1";
const LOCAL_VCF_DEMOGRAPHY_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-demography-smoke";
const GOVERNED_VCF_DEMOGRAPHY_STAGE_ID: &str = "vcf.demography";
const GOVERNED_VCF_DEMOGRAPHY_TOOL_ID: &str = "ibdne";
const GOVERNED_VCF_DEMOGRAPHY_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_DEMOGRAPHY_ASSET_PROFILE_ID: &str = "json_ibd_segments";
const GOVERNED_VCF_DEMOGRAPHY_INPUT_FIXTURE_ID: &str = "vcf_mini_multisample_cohort";
const GOVERNED_VCF_DEMOGRAPHY_EXPECTED_OUTPUT_ID: &str = "demography_report";
const GOVERNED_VCF_DEMOGRAPHY_UPSTREAM_TOOL_ID: &str = "germline";
const GOVERNED_VCF_DEMOGRAPHY_MIN_SEGMENTS: usize = 1;
const DEFAULT_INPUT_IBD_NAME: &str = "input_ibd_segments.tsv";
const DEFAULT_OUTPUT_JSON_NAME: &str = "demography.json";
const DEFAULT_OUTPUT_SOURCE_UPSTREAM_IBD_REPORT_NAME: &str = "source_ibd_smoke.json";
const DEFAULT_OUTPUT_SOURCE_UPSTREAM_FILTERED_SEGMENTS_NAME: &str =
    "source_ibd_filtered_segments.tsv";
const DEFAULT_OUTPUT_SOURCE_NE_TRAJECTORY_NAME: &str = "source_ne_trajectory.tsv";
const DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_CONTRACT_NAME: &str = "source_demography_contract.json";
const DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_METRICS_NAME: &str = "source_demography_metrics.json";
const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
const DEFAULT_PROBE_INPUT_IBD_NAME: &str = "probe_input_ibd_segments.tsv";
const DEFAULT_PROBE_SOURCE_NE_TRAJECTORY_NAME: &str = "probe_source_ne_trajectory.tsv";
const DEFAULT_PROBE_SOURCE_DEMOGRAPHY_CONTRACT_NAME: &str = "probe_source_demography_contract.json";
const DEFAULT_PROBE_SOURCE_DEMOGRAPHY_METRICS_NAME: &str = "probe_source_demography_metrics.json";
const DEFAULT_PROBE_SOURCE_LOGS_NAME: &str = "probe_source_logs.txt";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfDemographySmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfDemographyEstimate {
    pub(crate) time_bin: u64,
    pub(crate) ne: f64,
    pub(crate) ci_low: f64,
    pub(crate) ci_high: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfDemographyInsufficientProbe {
    pub(crate) input_ibd: String,
    pub(crate) method: String,
    pub(crate) status: String,
    pub(crate) insufficient_reason: String,
    pub(crate) time_bins: Vec<u64>,
    pub(crate) ne_estimates: Vec<LocalVcfDemographyEstimate>,
    pub(crate) source_ne_trajectory_path: String,
    pub(crate) source_demography_contract_path: String,
    pub(crate) source_demography_metrics_path: String,
    pub(crate) source_logs_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfDemographySmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) output_root: String,
    pub(crate) demography_json_path: String,
    pub(crate) input_ibd: String,
    pub(crate) source_upstream_ibd_report_path: String,
    pub(crate) source_upstream_filtered_segments_path: String,
    pub(crate) source_ne_trajectory_path: String,
    pub(crate) source_demography_contract_path: String,
    pub(crate) source_demography_metrics_path: String,
    pub(crate) source_logs_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) method: String,
    pub(crate) inference_status: String,
    pub(crate) tool_ok: bool,
    pub(crate) time_bins: Vec<u64>,
    pub(crate) ne_estimates: Vec<LocalVcfDemographyEstimate>,
    pub(crate) status: String,
    pub(crate) insufficient_reason: Option<String>,
    pub(crate) insufficient_data_probe: LocalVcfDemographyInsufficientProbe,
}

#[derive(Debug, Clone, PartialEq)]
struct NormalizedDemographyEvidence {
    method: String,
    inference_status: String,
    status: String,
    insufficient_reason: Option<String>,
    tool_ok: bool,
    time_bins: Vec<u64>,
    ne_estimates: Vec<LocalVcfDemographyEstimate>,
}

pub(crate) fn run_vcf_demography_smoke(
    args: &parse::BenchLocalRunVcfDemographySmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_demography_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.demography_json_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_demography_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfDemographySmokeReport> {
    let contract = resolve_governed_vcf_demography_smoke_contract(tool_id)?;
    let upstream_ibd_report =
        run_local_vcf_ibd_smoke(repo_root, GOVERNED_VCF_DEMOGRAPHY_UPSTREAM_TOOL_ID)?;
    let upstream_ibd_report_source = repo_root.join(&upstream_ibd_report.ibd_json_path);
    let upstream_filtered_segments_source =
        repo_root.join(&upstream_ibd_report.source_ibd_filtered_segments_path);
    ensure_file_exists(&upstream_ibd_report_source, "upstream VCF IBD smoke report")?;
    ensure_file_exists(&upstream_filtered_segments_source, "upstream VCF IBD filtered segments")?;

    let output_root = repo_root.join(DEFAULT_VCF_DEMOGRAPHY_SMOKE_ROOT).join(&contract.tool_id);
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

    let input_ibd_path = input_root.join(DEFAULT_INPUT_IBD_NAME);
    fs::copy(&upstream_filtered_segments_source, &input_ibd_path).with_context(|| {
        format!(
            "copy {} to {}",
            upstream_filtered_segments_source.display(),
            input_ibd_path.display()
        )
    })?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_demography_stage(
        &input_ibd_path,
        &stage_root,
        &DemographyStageParams {
            min_segments: GOVERNED_VCF_DEMOGRAPHY_MIN_SEGMENTS,
            ..DemographyStageParams::default()
        },
    )
    .with_context(|| {
        format!("run governed VCF demography smoke from {}", input_ibd_path.display())
    })?;

    let source_upstream_ibd_report_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_UPSTREAM_IBD_REPORT_NAME);
    fs::copy(&upstream_ibd_report_source, &source_upstream_ibd_report_path).with_context(|| {
        format!(
            "copy {} to {}",
            upstream_ibd_report_source.display(),
            source_upstream_ibd_report_path.display()
        )
    })?;
    let source_upstream_filtered_segments_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_UPSTREAM_FILTERED_SEGMENTS_NAME);
    fs::copy(&input_ibd_path, &source_upstream_filtered_segments_path).with_context(|| {
        format!(
            "copy {} to {}",
            input_ibd_path.display(),
            source_upstream_filtered_segments_path.display()
        )
    })?;
    let source_ne_trajectory_path = output_root.join(DEFAULT_OUTPUT_SOURCE_NE_TRAJECTORY_NAME);
    fs::copy(&stage_outputs.ne_trajectory_tsv, &source_ne_trajectory_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.ne_trajectory_tsv.display(),
            source_ne_trajectory_path.display()
        )
    })?;
    let source_demography_contract_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_CONTRACT_NAME);
    fs::copy(&stage_outputs.demography_json, &source_demography_contract_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.demography_json.display(),
                source_demography_contract_path.display()
            )
        },
    )?;
    let source_demography_metrics_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_METRICS_NAME);
    fs::copy(&stage_outputs.demography_metrics_json, &source_demography_metrics_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.demography_metrics_json.display(),
                source_demography_metrics_path.display()
            )
        })?;
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    fs::copy(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let evidence = load_demography_evidence(
        &source_demography_contract_path,
        &source_demography_metrics_path,
    )?;
    validate_primary_demography_evidence(&evidence)?;
    let insufficient_data_probe = run_insufficient_data_probe(
        repo_root,
        &probe_root,
        &upstream_ibd_report.insufficient_overlap_probe.source_ibd_filtered_segments_path,
    )?;

    let demography_json_path = output_root.join(DEFAULT_OUTPUT_JSON_NAME);
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let report = LocalVcfDemographySmokeReport {
        schema_version: LOCAL_VCF_DEMOGRAPHY_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_DEMOGRAPHY_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        output_root: path_relative_to_repo(repo_root, &output_root),
        demography_json_path: path_relative_to_repo(repo_root, &demography_json_path),
        input_ibd: path_relative_to_repo(repo_root, &input_ibd_path),
        source_upstream_ibd_report_path: path_relative_to_repo(
            repo_root,
            &source_upstream_ibd_report_path,
        ),
        source_upstream_filtered_segments_path: path_relative_to_repo(
            repo_root,
            &source_upstream_filtered_segments_path,
        ),
        source_ne_trajectory_path: path_relative_to_repo(repo_root, &source_ne_trajectory_path),
        source_demography_contract_path: path_relative_to_repo(
            repo_root,
            &source_demography_contract_path,
        ),
        source_demography_metrics_path: path_relative_to_repo(
            repo_root,
            &source_demography_metrics_path,
        ),
        source_logs_path: path_relative_to_repo(repo_root, &source_logs_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at,
        finished_at,
        elapsed_seconds,
        exit_code: 0,
        method: evidence.method,
        inference_status: evidence.inference_status,
        tool_ok: evidence.tool_ok,
        time_bins: evidence.time_bins,
        ne_estimates: evidence.ne_estimates,
        status: evidence.status,
        insufficient_reason: evidence.insufficient_reason,
        insufficient_data_probe,
    };
    bijux_dna_infra::atomic_write_json(&demography_json_path, &report)?;

    let stage_result_manifest = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: contract.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: contract.tool_id.clone() },
        command: BenchStageResultCommandV1 { rendered: report.command.clone() },
        runtime: BenchStageResultRuntimeV1 {
            mode: "local_smoke".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: report.started_at.clone(),
            finished_at: report.finished_at.clone(),
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
                "demography_json",
                DEFAULT_OUTPUT_JSON_NAME,
                &demography_json_path,
                "report_output",
            ),
            stage_result_output(
                repo_root,
                "source_upstream_ibd_report_json",
                DEFAULT_OUTPUT_SOURCE_UPSTREAM_IBD_REPORT_NAME,
                &source_upstream_ibd_report_path,
                "report_output",
            ),
            stage_result_output(
                repo_root,
                "source_upstream_ibd_filtered_segments_tsv",
                DEFAULT_OUTPUT_SOURCE_UPSTREAM_FILTERED_SEGMENTS_NAME,
                &source_upstream_filtered_segments_path,
                "table_output",
            ),
            stage_result_output(
                repo_root,
                "source_ne_trajectory_tsv",
                DEFAULT_OUTPUT_SOURCE_NE_TRAJECTORY_NAME,
                &source_ne_trajectory_path,
                "table_output",
            ),
            stage_result_output(
                repo_root,
                "source_demography_contract_json",
                DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_CONTRACT_NAME,
                &source_demography_contract_path,
                "report_output",
            ),
            stage_result_output(
                repo_root,
                "source_demography_metrics_json",
                DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_METRICS_NAME,
                &source_demography_metrics_path,
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
                "probe_input_ibd_segments_tsv",
                DEFAULT_PROBE_INPUT_IBD_NAME,
                &repo_root.join(&report.insufficient_data_probe.input_ibd),
                "table_output",
            ),
            stage_result_output(
                repo_root,
                "probe_source_ne_trajectory_tsv",
                DEFAULT_PROBE_SOURCE_NE_TRAJECTORY_NAME,
                &repo_root.join(&report.insufficient_data_probe.source_ne_trajectory_path),
                "table_output",
            ),
            stage_result_output(
                repo_root,
                "probe_source_demography_contract_json",
                DEFAULT_PROBE_SOURCE_DEMOGRAPHY_CONTRACT_NAME,
                &repo_root.join(&report.insufficient_data_probe.source_demography_contract_path),
                "report_output",
            ),
            stage_result_output(
                repo_root,
                "probe_source_demography_metrics_json",
                DEFAULT_PROBE_SOURCE_DEMOGRAPHY_METRICS_NAME,
                &repo_root.join(&report.insufficient_data_probe.source_demography_metrics_path),
                "report_output",
            ),
            stage_result_output(
                repo_root,
                "probe_source_logs_txt",
                DEFAULT_PROBE_SOURCE_LOGS_NAME,
                &repo_root.join(&report.insufficient_data_probe.source_logs_path),
                "log_output",
            ),
        ],
    };
    validate_stage_result_manifest(&stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    Ok(report)
}

fn run_insufficient_data_probe(
    repo_root: &Path,
    probe_root: &Path,
    upstream_probe_filtered_segments_path: &str,
) -> Result<LocalVcfDemographyInsufficientProbe> {
    let upstream_probe_filtered_segments_source =
        repo_root.join(upstream_probe_filtered_segments_path);
    ensure_file_exists(
        &upstream_probe_filtered_segments_source,
        "upstream sparse-overlap IBD filtered segments",
    )?;
    let input_ibd_path = probe_root.join(DEFAULT_PROBE_INPUT_IBD_NAME);
    fs::copy(&upstream_probe_filtered_segments_source, &input_ibd_path).with_context(|| {
        format!(
            "copy {} to {}",
            upstream_probe_filtered_segments_source.display(),
            input_ibd_path.display()
        )
    })?;
    let stage_root = probe_root.join("stage");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let stage_outputs = run_demography_stage(
        &input_ibd_path,
        &stage_root,
        &DemographyStageParams {
            min_segments: GOVERNED_VCF_DEMOGRAPHY_MIN_SEGMENTS,
            ..DemographyStageParams::default()
        },
    )
    .with_context(|| {
        format!("run sparse-overlap demography probe from {}", input_ibd_path.display())
    })?;

    let source_ne_trajectory_path = probe_root.join(DEFAULT_PROBE_SOURCE_NE_TRAJECTORY_NAME);
    fs::copy(&stage_outputs.ne_trajectory_tsv, &source_ne_trajectory_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.ne_trajectory_tsv.display(),
            source_ne_trajectory_path.display()
        )
    })?;
    let source_demography_contract_path =
        probe_root.join(DEFAULT_PROBE_SOURCE_DEMOGRAPHY_CONTRACT_NAME);
    fs::copy(&stage_outputs.demography_json, &source_demography_contract_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.demography_json.display(),
                source_demography_contract_path.display()
            )
        },
    )?;
    let source_demography_metrics_path =
        probe_root.join(DEFAULT_PROBE_SOURCE_DEMOGRAPHY_METRICS_NAME);
    fs::copy(&stage_outputs.demography_metrics_json, &source_demography_metrics_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.demography_metrics_json.display(),
                source_demography_metrics_path.display()
            )
        })?;
    let source_logs_path = probe_root.join(DEFAULT_PROBE_SOURCE_LOGS_NAME);
    fs::copy(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let evidence = load_demography_evidence(
        &source_demography_contract_path,
        &source_demography_metrics_path,
    )?;
    if evidence.status != "insufficient_data" {
        bail!(
            "sparse-overlap demography probe expected `insufficient_data`, found `{}`",
            evidence.status
        );
    }
    let insufficient_reason = evidence.insufficient_reason.clone().ok_or_else(|| {
        anyhow!("sparse-overlap demography probe must report `insufficient_data_reason`")
    })?;

    Ok(LocalVcfDemographyInsufficientProbe {
        input_ibd: path_relative_to_repo(repo_root, &input_ibd_path),
        method: evidence.method,
        status: evidence.status,
        insufficient_reason,
        time_bins: evidence.time_bins,
        ne_estimates: evidence.ne_estimates,
        source_ne_trajectory_path: path_relative_to_repo(repo_root, &source_ne_trajectory_path),
        source_demography_contract_path: path_relative_to_repo(
            repo_root,
            &source_demography_contract_path,
        ),
        source_demography_metrics_path: path_relative_to_repo(
            repo_root,
            &source_demography_metrics_path,
        ),
        source_logs_path: path_relative_to_repo(repo_root, &source_logs_path),
    })
}

fn resolve_governed_vcf_demography_smoke_contract(
    tool_id: &str,
) -> Result<GovernedVcfDemographySmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_DEMOGRAPHY_STAGE_ID)
        .ok_or_else(|| {
            anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_DEMOGRAPHY_STAGE_ID}`")
        })?;
    if matrix_row.tool_id != tool_id {
        bail!(
            "VCF demography smoke only retains tool `{}`; requested `{tool_id}`",
            matrix_row.tool_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_DEMOGRAPHY_CORPUS_ID {
        bail!(
            "VCF demography smoke requires corpus `{GOVERNED_VCF_DEMOGRAPHY_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_DEMOGRAPHY_ASSET_PROFILE_ID {
        bail!(
            "VCF demography smoke requires asset profile `{GOVERNED_VCF_DEMOGRAPHY_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec![GOVERNED_VCF_DEMOGRAPHY_EXPECTED_OUTPUT_ID.to_string()] {
        bail!(
            "VCF demography smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }

    Ok(GovernedVcfDemographySmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_DEMOGRAPHY_INPUT_FIXTURE_ID.to_string(),
    })
}

fn validate_primary_demography_evidence(evidence: &NormalizedDemographyEvidence) -> Result<()> {
    match evidence.status.as_str() {
        "complete" => {
            if evidence.ne_estimates.is_empty() {
                bail!("governed VCF demography smoke expected non-empty Ne estimates");
            }
            if evidence.time_bins.len() != evidence.ne_estimates.len() {
                bail!(
                    "governed VCF demography smoke time-bin count drifted from Ne estimate count"
                );
            }
            if evidence.insufficient_reason.is_some() {
                bail!("governed VCF demography smoke must not report `insufficient_reason` when status is complete");
            }
        }
        "insufficient_data" => {
            if evidence.insufficient_reason.is_none() {
                bail!(
                    "governed VCF demography smoke insufficient-data result must report a reason"
                );
            }
        }
        other => bail!("governed VCF demography smoke reported unsupported status `{other}`"),
    }
    Ok(())
}

fn load_demography_evidence(
    contract_path: &Path,
    metrics_path: &Path,
) -> Result<NormalizedDemographyEvidence> {
    let contract = read_json(contract_path)?;
    let metrics = read_json(metrics_path)?;
    let method = contract
        .get("method")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("demography contract is missing `method`"))?
        .to_string();
    let metrics_method = metrics
        .get("method")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("demography metrics are missing `method`"))?;
    if method != metrics_method {
        bail!("demography method drifted between contract and metrics");
    }
    let inference_status = contract
        .get("inference_status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("demography contract is missing `inference_status`"))?
        .to_string();
    let metrics_inference_status = metrics
        .get("inference_status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("demography metrics are missing `inference_status`"))?;
    if inference_status != metrics_inference_status {
        bail!("demography inference status drifted between contract and metrics");
    }
    let status = contract
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("demography contract is missing `status`"))?
        .to_string();
    let metrics_status = metrics
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("demography metrics are missing `status`"))?;
    if status != metrics_status {
        bail!("demography status drifted between contract and metrics");
    }
    let insufficient_reason = contract
        .get("insufficient_data_reason")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let metrics_insufficient_reason = metrics
        .get("insufficient_data_reason")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    if insufficient_reason != metrics_insufficient_reason {
        bail!("demography insufficient-data reason drifted between contract and metrics");
    }
    let time_bins = parse_time_bins(&contract)?;
    let metrics_time_bins = parse_time_bins(&metrics)?;
    if time_bins != metrics_time_bins {
        bail!("demography time bins drifted between contract and metrics");
    }
    let ne_estimates = parse_ne_estimates(&contract)?;
    let metrics_ne_estimates = parse_ne_estimates(&metrics)?;
    if ne_estimates != metrics_ne_estimates {
        bail!("demography Ne estimates drifted between contract and metrics");
    }
    let tool_ok = metrics
        .pointer("/tool_attempts/ibdne")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("demography metrics are missing `tool_attempts.ibdne`"))?;

    Ok(NormalizedDemographyEvidence {
        method,
        inference_status,
        status,
        insufficient_reason,
        tool_ok,
        time_bins,
        ne_estimates,
    })
}

fn parse_time_bins(payload: &serde_json::Value) -> Result<Vec<u64>> {
    payload
        .get("time_bins")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("demography payload is missing `time_bins`"))?
        .iter()
        .map(|value| {
            value
                .as_u64()
                .ok_or_else(|| anyhow!("demography `time_bins` entries must be unsigned integers"))
        })
        .collect()
}

fn parse_ne_estimates(payload: &serde_json::Value) -> Result<Vec<LocalVcfDemographyEstimate>> {
    payload
        .get("ne_estimates")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("demography payload is missing `ne_estimates`"))?
        .iter()
        .map(|row| {
            Ok(LocalVcfDemographyEstimate {
                time_bin: row
                    .get("generation")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| anyhow!("demography Ne estimate is missing `generation`"))?,
                ne: row
                    .get("ne")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or_else(|| anyhow!("demography Ne estimate is missing `ne`"))?,
                ci_low: row
                    .get("ci_low")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or_else(|| anyhow!("demography Ne estimate is missing `ci_low`"))?,
                ci_high: row
                    .get("ci_high")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or_else(|| anyhow!("demography Ne estimate is missing `ci_high`"))?,
            })
        })
        .collect()
}

fn ensure_file_exists(path: &Path, label: &str) -> Result<()> {
    if path.is_file() {
        Ok(())
    } else {
        bail!("{label} is missing at {}", path.display())
    }
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

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
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
        load_demography_evidence, resolve_governed_vcf_demography_smoke_contract,
        run_local_vcf_demography_smoke, LocalVcfDemographyEstimate,
    };

    #[test]
    fn governed_vcf_demography_smoke_contract_matches_stage_matrix() {
        let contract =
            resolve_governed_vcf_demography_smoke_contract("ibdne").expect("resolve contract");
        assert_eq!(contract.stage_id, "vcf.demography");
        assert_eq!(contract.tool_id, "ibdne");
        assert_eq!(contract.corpus_id, "vcf_production_regression");
        assert_eq!(contract.input_fixture_id, "vcf_mini_multisample_cohort");
    }

    #[test]
    fn demography_evidence_requires_aligned_contract_and_metrics() {
        let dir = tempfile::tempdir().expect("tempdir");
        let contract_path = dir.path().join("contract.json");
        let metrics_path = dir.path().join("metrics.json");
        let rows = vec![serde_json::json!({
            "generation": 5,
            "ne": 1000.0,
            "ci_low": 850.0,
            "ci_high": 1150.0
        })];
        let payload = serde_json::json!({
            "method": "ibdne",
            "inference_status": "fallback_estimate",
            "status": "complete",
            "insufficient_data_reason": serde_json::Value::Null,
            "time_bins": [5],
            "ne_estimates": rows,
            "tool_attempts": { "ibdne": false }
        });
        bijux_dna_infra::atomic_write_json(&contract_path, &payload).expect("write contract");
        bijux_dna_infra::atomic_write_json(&metrics_path, &payload).expect("write metrics");
        let evidence =
            load_demography_evidence(&contract_path, &metrics_path).expect("load evidence");
        assert_eq!(evidence.method, "ibdne");
        assert_eq!(evidence.status, "complete");
        assert_eq!(evidence.time_bins, vec![5]);
        assert_eq!(
            evidence.ne_estimates,
            vec![LocalVcfDemographyEstimate {
                time_bin: 5,
                ne: 1000.0,
                ci_low: 850.0,
                ci_high: 1150.0
            }]
        );
    }

    #[test]
    fn governed_vcf_demography_smoke_reports_main_run_and_probe() {
        let repo_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).expect("repo root");
        let report =
            run_local_vcf_demography_smoke(repo_root, "ibdne").expect("run local demography smoke");
        assert_eq!(report.stage_id, "vcf.demography");
        assert_eq!(report.tool_id, "ibdne");
        assert_eq!(report.corpus_id, "vcf_production_regression");
        assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
        assert_eq!(report.method, "ibdne");
        assert!(
            report.status == "complete" || report.status == "insufficient_data",
            "unexpected demography status: {}",
            report.status
        );
        if report.status == "complete" {
            assert!(!report.ne_estimates.is_empty(), "expected Ne estimates for complete result");
        }
        assert_eq!(report.insufficient_data_probe.status, "insufficient_data");
        assert_eq!(report.insufficient_data_probe.insufficient_reason, "not_enough_ibd_segments");
    }
}
