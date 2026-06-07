use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_stages_vcf::pipeline::{run_admixture_stage, AdmixtureStageParams};
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

const DEFAULT_VCF_ADMIXTURE_SMOKE_ROOT: &str = "runs/bench/local-smoke/vcf.admixture";
const LOCAL_VCF_ADMIXTURE_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_admixture_smoke.v1";
const LOCAL_VCF_ADMIXTURE_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-admixture-smoke";
const GOVERNED_VCF_ADMIXTURE_STAGE_ID: &str = "vcf.admixture";
const GOVERNED_VCF_ADMIXTURE_TOOL_ID: &str = "plink2";
const GOVERNED_VCF_ADMIXTURE_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_ADMIXTURE_ASSET_PROFILE_ID: &str = "vcf_cohort";
const GOVERNED_VCF_ADMIXTURE_INPUT_FIXTURE_ID: &str = "vcf_mini_multisample_cohort";
const DEFAULT_INPUT_VCF_NAME: &str = "admixture_input.vcf";
const DEFAULT_INPUT_SAMPLE_METADATA_NAME: &str = "sample_metadata.tsv";
const DEFAULT_INPUT_POPULATION_METADATA_NAME: &str = "population_metadata.tsv";
const DEFAULT_INPUT_POPULATION_LABELS_NAME: &str = "population_labels.json";
const DEFAULT_OUTPUT_TSV_NAME: &str = "admixture.tsv";
const DEFAULT_OUTPUT_JSON_NAME: &str = "admixture.json";
const DEFAULT_OUTPUT_SOURCE_Q_MATRIX_NAME: &str = "source_admixture_q_matrix.tsv";
const DEFAULT_OUTPUT_SOURCE_K_SELECTION_NAME: &str = "source_admixture_k_selection.json";
const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";
const GOVERNED_VCF_ADMIXTURE_K_VALUES: &[usize] = &[2, 3, 4];
const ANCESTRY_SUM_TOLERANCE: f64 = 1e-6;

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfAdmixtureSmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfAdmixtureSmokeRow {
    pub(crate) sample_id: String,
    pub(crate) population_id: String,
    pub(crate) population_label: String,
    pub(crate) sex: String,
    #[serde(rename = "K")]
    pub(crate) k: usize,
    pub(crate) status: String,
    #[serde(flatten)]
    pub(crate) clusters: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfAdmixtureSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) fixture_manifest_path: String,
    pub(crate) input_vcf_path: String,
    pub(crate) sample_metadata_path: String,
    pub(crate) population_metadata_path: String,
    pub(crate) population_labels_manifest_path: String,
    pub(crate) output_root: String,
    pub(crate) admixture_tsv_path: String,
    pub(crate) admixture_json_path: String,
    pub(crate) source_q_matrix_path: String,
    pub(crate) source_k_selection_path: String,
    pub(crate) source_logs_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) execution_mode: String,
    pub(crate) tool_ok: bool,
    pub(crate) selected_k: usize,
    pub(crate) status: String,
    pub(crate) insufficient_data_reason: Option<String>,
    pub(crate) sample_count: u64,
    pub(crate) population_count: u64,
    pub(crate) cluster_headers: Vec<String>,
    pub(crate) rows: Vec<LocalVcfAdmixtureSmokeRow>,
}

pub(crate) fn run_vcf_admixture_smoke(
    args: &parse::BenchLocalRunVcfAdmixtureSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_admixture_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.admixture_json_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_admixture_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfAdmixtureSmokeReport> {
    let contract = resolve_governed_vcf_admixture_smoke_contract(tool_id)?;
    let fixture_manifest_path = repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
    let fixture_manifest = load_vcf_corpus_fixture_manifest_path(&fixture_manifest_path)?;
    let fixture_dir = fixture_manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", fixture_manifest_path.display())
    })?;
    let input_vcf_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.multisample_vcf_path);
    let sample_metadata_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.sample_metadata_path);
    let population_metadata_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.population_metadata_path);
    let expected_samples = fixture_manifest.expected_multisample_sample_ids.clone();
    if expected_samples.is_empty() {
        bail!("governed VCF admixture smoke fixture must declare multisample cohort sample ids");
    }

    let sample_metadata = load_sample_metadata(&sample_metadata_source)?;
    let metadata_by_sample = build_metadata_by_sample(&sample_metadata, &expected_samples)?;
    let population_labels = load_population_labels(&population_metadata_source)?;

    let output_root = repo_root.join(DEFAULT_VCF_ADMIXTURE_SMOKE_ROOT).join(&contract.tool_id);
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
    let population_metadata_path = input_root.join(DEFAULT_INPUT_POPULATION_METADATA_NAME);
    fs::copy(&population_metadata_source, &population_metadata_path).with_context(|| {
        format!(
            "copy {} to {}",
            population_metadata_source.display(),
            population_metadata_path.display()
        )
    })?;
    let population_labels_manifest_path = input_root.join(DEFAULT_INPUT_POPULATION_LABELS_NAME);
    write_population_labels_manifest(
        &population_labels_manifest_path,
        &metadata_by_sample,
        &expected_samples,
    )?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_admixture_stage(
        &input_vcf_path,
        &stage_root,
        &AdmixtureStageParams {
            toolchain: contract.tool_id.clone(),
            k_values: GOVERNED_VCF_ADMIXTURE_K_VALUES.to_vec(),
            sample_metadata_manifest: Some(population_labels_manifest_path.clone()),
        },
    )
    .with_context(|| {
        format!("run governed VCF admixture smoke from {}", input_vcf_path.display())
    })?;

    let source_q_matrix_path = output_root.join(DEFAULT_OUTPUT_SOURCE_Q_MATRIX_NAME);
    fs::copy(&stage_outputs.q_matrix_tsv, &source_q_matrix_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.q_matrix_tsv.display(),
            source_q_matrix_path.display()
        )
    })?;
    let source_k_selection_path = output_root.join(DEFAULT_OUTPUT_SOURCE_K_SELECTION_NAME);
    fs::copy(&stage_outputs.k_selection_json, &source_k_selection_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.k_selection_json.display(),
            source_k_selection_path.display()
        )
    })?;
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    fs::copy(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let source_manifest = read_json(&source_k_selection_path)?;
    let execution_mode = source_manifest
        .get("execution_mode")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("admixture manifest missing `execution_mode`"))?
        .to_string();
    let tool_ok = source_manifest
        .get("tool_ok")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("admixture manifest missing `tool_ok`"))?;
    let status = source_manifest
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("admixture manifest missing `status`"))?
        .to_string();
    let insufficient_data_reason = source_manifest
        .get("insufficient_data_reason")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let selected_k = usize::try_from(
        source_manifest
            .get("selected_k")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| anyhow!("admixture manifest missing `selected_k`"))?,
    )
    .map_err(|_| anyhow!("selected_k overflow"))?;
    let sample_count = source_manifest
        .get("sample_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("admixture manifest missing `sample_count`"))?;
    let population_count = source_manifest
        .get("population_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("admixture manifest missing `population_count`"))?;
    let cluster_headers = source_manifest
        .get("cluster_headers")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("admixture manifest missing `cluster_headers`"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("admixture manifest cluster_headers must be strings"))
        })
        .collect::<Result<Vec<_>>>()?;
    validate_admixture_manifest(&source_manifest, &expected_samples, selected_k, &cluster_headers)?;

    let rows = parse_admixture_rows(
        &source_q_matrix_path,
        &metadata_by_sample,
        &population_labels,
        &cluster_headers,
        selected_k,
        &status,
    )?;
    validate_sample_coverage(&rows, &expected_samples)?;

    let admixture_tsv_path = output_root.join(DEFAULT_OUTPUT_TSV_NAME);
    bijux_dna_infra::atomic_write_bytes(
        &admixture_tsv_path,
        build_admixture_tsv(&rows, &cluster_headers).as_bytes(),
    )?;
    let admixture_json_path = output_root.join(DEFAULT_OUTPUT_JSON_NAME);
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let report = LocalVcfAdmixtureSmokeReport {
        schema_version: LOCAL_VCF_ADMIXTURE_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_ADMIXTURE_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        fixture_manifest_path: path_relative_to_repo(repo_root, &fixture_manifest_path),
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf_path),
        sample_metadata_path: path_relative_to_repo(repo_root, &sample_metadata_path),
        population_metadata_path: path_relative_to_repo(repo_root, &population_metadata_path),
        population_labels_manifest_path: path_relative_to_repo(
            repo_root,
            &population_labels_manifest_path,
        ),
        output_root: path_relative_to_repo(repo_root, &output_root),
        admixture_tsv_path: path_relative_to_repo(repo_root, &admixture_tsv_path),
        admixture_json_path: path_relative_to_repo(repo_root, &admixture_json_path),
        source_q_matrix_path: path_relative_to_repo(repo_root, &source_q_matrix_path),
        source_k_selection_path: path_relative_to_repo(repo_root, &source_k_selection_path),
        source_logs_path: path_relative_to_repo(repo_root, &source_logs_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at: started_at.clone(),
        finished_at: finished_at.clone(),
        elapsed_seconds,
        exit_code: 0,
        execution_mode,
        tool_ok,
        selected_k,
        status,
        insufficient_data_reason,
        sample_count,
        population_count,
        cluster_headers: cluster_headers.clone(),
        rows,
    };
    bijux_dna_infra::atomic_write_json(&admixture_json_path, &report)?;

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
                artifact_id: "admixture_tsv".to_string(),
                declared_path: DEFAULT_OUTPUT_TSV_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &admixture_tsv_path),
                role: "table_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "admixture_json".to_string(),
                declared_path: DEFAULT_OUTPUT_JSON_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &admixture_json_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_q_matrix_tsv".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_Q_MATRIX_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_q_matrix_path),
                role: "table_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_k_selection_json".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_K_SELECTION_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_k_selection_path),
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

fn resolve_governed_vcf_admixture_smoke_contract(
    tool_id: &str,
) -> Result<GovernedVcfAdmixtureSmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_ADMIXTURE_STAGE_ID)
        .ok_or_else(|| {
            anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_ADMIXTURE_STAGE_ID}`")
        })?;
    if tool_id != matrix_row.tool_id {
        bail!(
            "VCF admixture smoke only retains tool `{}` for `{}`; requested `{tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.tool_id != GOVERNED_VCF_ADMIXTURE_TOOL_ID {
        bail!(
            "VCF admixture smoke requires tool `{GOVERNED_VCF_ADMIXTURE_TOOL_ID}`, found `{}`",
            matrix_row.tool_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_ADMIXTURE_CORPUS_ID {
        bail!(
            "VCF admixture smoke requires corpus `{GOVERNED_VCF_ADMIXTURE_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_ADMIXTURE_ASSET_PROFILE_ID {
        bail!(
            "VCF admixture smoke requires asset profile `{GOVERNED_VCF_ADMIXTURE_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["admixture_report".to_string()] {
        bail!(
            "VCF admixture smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfAdmixtureSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_ADMIXTURE_INPUT_FIXTURE_ID.to_string(),
    })
}

fn build_metadata_by_sample(
    sample_metadata: &[SampleMetadataRow],
    expected_samples: &[String],
) -> Result<BTreeMap<String, SampleMetadataRow>> {
    let expected_sample_set = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    let mut metadata_by_sample = BTreeMap::<String, SampleMetadataRow>::new();
    for row in sample_metadata {
        if !expected_sample_set.contains(&row.sample_id) {
            continue;
        }
        if row.role != "cohort" {
            bail!(
                "governed VCF admixture smoke expected cohort role for `{}`, found `{}`",
                row.sample_id,
                row.role
            );
        }
        metadata_by_sample.insert(row.sample_id.clone(), row.clone());
    }
    for sample_id in expected_samples {
        if !metadata_by_sample.contains_key(sample_id) {
            bail!("governed VCF admixture smoke metadata is missing expected sample `{sample_id}`");
        }
    }
    Ok(metadata_by_sample)
}

fn load_population_labels(path: &Path) -> Result<BTreeMap<String, String>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut rows = raw.lines();
    let header = rows.next().ok_or_else(|| anyhow!("population metadata is empty"))?;
    if header.trim() != "population_id\tpopulation_label\tsuper_population\trole" {
        bail!("population metadata header drifted in {}", path.display());
    }
    let mut labels = BTreeMap::<String, String>::new();
    for (index, line) in rows.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 4 {
            bail!(
                "population metadata row {} in {} must have 4 columns",
                index + 2,
                path.display()
            );
        }
        let population_id = columns[0].trim();
        let population_label = columns[1].trim();
        if population_id.is_empty() || population_label.is_empty() {
            bail!(
                "population metadata row {} in {} must keep non-empty ids and labels",
                index + 2,
                path.display()
            );
        }
        labels.insert(population_id.to_string(), population_label.to_string());
    }
    Ok(labels)
}

fn write_population_labels_manifest(
    path: &Path,
    metadata_by_sample: &BTreeMap<String, SampleMetadataRow>,
    expected_samples: &[String],
) -> Result<()> {
    let payload = serde_json::json!({
        "samples": expected_samples.iter().map(|sample_id| {
            let row = metadata_by_sample
                .get(sample_id)
                .unwrap_or_else(|| panic!("missing metadata for sample `{sample_id}`"));
            serde_json::json!({
                "sample": row.sample_id,
                "population": row.population_id,
            })
        }).collect::<Vec<_>>()
    });
    bijux_dna_infra::atomic_write_json(path, &payload)?;
    Ok(())
}

fn validate_admixture_manifest(
    manifest: &serde_json::Value,
    expected_samples: &[String],
    selected_k: usize,
    cluster_headers: &[String],
) -> Result<()> {
    let manifest_sample_ids = manifest
        .get("sample_ids")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("admixture manifest missing `sample_ids`"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("admixture manifest sample_ids must be strings"))
        })
        .collect::<Result<Vec<_>>>()?;
    if manifest_sample_ids != expected_samples {
        bail!(
            "admixture manifest sample_ids drifted: expected {:?}, found {:?}",
            expected_samples,
            manifest_sample_ids
        );
    }
    if cluster_headers.len() != selected_k {
        bail!(
            "admixture manifest cluster_headers length drifted: expected {}, found {}",
            selected_k,
            cluster_headers.len()
        );
    }
    let manifest_sample_labels = manifest
        .get("sample_population_labels")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("admixture manifest missing `sample_population_labels`"))?;
    if manifest_sample_labels.len() != expected_samples.len() {
        bail!(
            "admixture manifest sample_population_labels length drifted: expected {}, found {}",
            expected_samples.len(),
            manifest_sample_labels.len()
        );
    }
    Ok(())
}

fn parse_admixture_rows(
    path: &Path,
    metadata_by_sample: &BTreeMap<String, SampleMetadataRow>,
    population_labels: &BTreeMap<String, String>,
    cluster_headers: &[String],
    selected_k: usize,
    status: &str,
) -> Result<Vec<LocalVcfAdmixtureSmokeRow>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header =
        lines.next().ok_or_else(|| anyhow!("admixture table is empty: {}", path.display()))?;
    let expected_header = format!("sample\t{}", cluster_headers.join("\t"));
    if header.trim() != expected_header {
        bail!("admixture table header drifted in {}: `{header}`", path.display());
    }
    let mut rows = Vec::<LocalVcfAdmixtureSmokeRow>::new();
    let mut seen = BTreeSet::<String>::new();
    for (index, line) in lines.enumerate() {
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != selected_k + 1 {
            bail!(
                "admixture row {} in {} must provide sample plus {} cluster columns",
                index + 2,
                path.display(),
                selected_k
            );
        }
        let sample_id = columns[0].trim().to_string();
        if sample_id.is_empty() {
            bail!("admixture row {} in {} is missing sample id", index + 2, path.display());
        }
        if !seen.insert(sample_id.clone()) {
            bail!("admixture table duplicated sample `{sample_id}` in {}", path.display());
        }
        let metadata = metadata_by_sample.get(&sample_id).ok_or_else(|| {
            anyhow!("admixture table sample `{sample_id}` is missing governed metadata")
        })?;
        let population_label = population_labels.get(&metadata.population_id).ok_or_else(|| {
            anyhow!("population metadata is missing label for `{}`", metadata.population_id)
        })?;
        let mut clusters = BTreeMap::<String, f64>::new();
        let mut ancestry_sum = 0.0_f64;
        for (column_index, cluster_header) in cluster_headers.iter().enumerate() {
            let value = columns[column_index + 1].parse::<f64>().with_context(|| {
                format!("parse {} for `{sample_id}` from {}", cluster_header, path.display())
            })?;
            clusters.insert(cluster_header.clone(), value);
            ancestry_sum += value;
        }
        if (ancestry_sum - 1.0).abs() > ANCESTRY_SUM_TOLERANCE {
            bail!(
                "admixture row `{sample_id}` in {} sums to {:.6}, expected 1.0 ± {}",
                path.display(),
                ancestry_sum,
                ANCESTRY_SUM_TOLERANCE
            );
        }
        rows.push(LocalVcfAdmixtureSmokeRow {
            sample_id,
            population_id: metadata.population_id.clone(),
            population_label: population_label.clone(),
            sex: metadata.sex.clone(),
            k: selected_k,
            status: status.to_string(),
            clusters,
        });
    }
    if rows.is_empty() {
        bail!("admixture table contained no sample rows in {}", path.display());
    }
    Ok(rows)
}

fn validate_sample_coverage(
    rows: &[LocalVcfAdmixtureSmokeRow],
    expected_samples: &[String],
) -> Result<()> {
    let observed_samples = rows.iter().map(|row| row.sample_id.clone()).collect::<Vec<_>>();
    let observed_sample_set = observed_samples.iter().cloned().collect::<BTreeSet<_>>();
    let expected_sample_set = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    let excluded_samples = expected_samples
        .iter()
        .filter(|sample_id| !observed_sample_set.contains(*sample_id))
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_samples = observed_samples
        .iter()
        .filter(|sample_id| !expected_sample_set.contains(*sample_id))
        .cloned()
        .collect::<Vec<_>>();
    if !excluded_samples.is_empty()
        || !unexpected_samples.is_empty()
        || rows.len() != expected_samples.len()
    {
        bail!(
            "governed VCF admixture smoke expected samples {:?}, observed {:?}",
            expected_samples,
            observed_samples
        );
    }
    Ok(())
}

fn build_admixture_tsv(rows: &[LocalVcfAdmixtureSmokeRow], cluster_headers: &[String]) -> String {
    let mut rendered = String::from("sample_id\tpopulation_id\tpopulation_label\tsex\tK\tstatus");
    for cluster_header in cluster_headers {
        rendered.push('\t');
        rendered.push_str(cluster_header);
    }
    rendered.push('\n');
    for row in rows {
        rendered.push_str(&row.sample_id);
        rendered.push('\t');
        rendered.push_str(&row.population_id);
        rendered.push('\t');
        rendered.push_str(&row.population_label);
        rendered.push('\t');
        rendered.push_str(&row.sex);
        rendered.push('\t');
        rendered.push_str(&row.k.to_string());
        rendered.push('\t');
        rendered.push_str(&row.status);
        for cluster_header in cluster_headers {
            rendered.push('\t');
            rendered.push_str(
                &row.clusters
                    .get(cluster_header)
                    .map(|value| format!("{value:.6}"))
                    .unwrap_or_else(|| "0.000000".to_string()),
            );
        }
        rendered.push('\n');
    }
    rendered
}

fn resolve_manifest_relative_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn timestamp_marker() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::{parse_admixture_rows, run_local_vcf_admixture_smoke};
    use crate::commands::benchmark::local_corpus_fixture::vcf::SampleMetadataRow;
    use std::collections::BTreeMap;

    #[test]
    fn admixture_rows_reject_duplicate_samples() {
        let temp = tempfile::tempdir().expect("tempdir");
        let table = temp.path().join("admixture.tsv");
        std::fs::write(
            &table,
            "sample\tcluster_1\tcluster_2\nsample_a\t0.4\t0.6\nsample_a\t0.3\t0.7\n",
        )
        .expect("write admixture table");
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "sample_a".to_string(),
            SampleMetadataRow {
                sample_id: "sample_a".to_string(),
                population_id: "cohort_alpha".to_string(),
                sex: "female".to_string(),
                role: "cohort".to_string(),
                description: "sample a".to_string(),
            },
        );
        let mut populations = BTreeMap::new();
        populations.insert("cohort_alpha".to_string(), "Cohort Alpha".to_string());
        let err = parse_admixture_rows(
            &table,
            &metadata,
            &populations,
            &["cluster_1".to_string(), "cluster_2".to_string()],
            2,
            "complete",
        )
        .expect_err("duplicate samples");
        assert!(err.to_string().contains("duplicated sample"));
    }

    #[test]
    fn admixture_rows_require_ancestry_sum_to_one() {
        let temp = tempfile::tempdir().expect("tempdir");
        let table = temp.path().join("admixture.tsv");
        std::fs::write(&table, "sample\tcluster_1\tcluster_2\nsample_a\t0.4\t0.5\n")
            .expect("write admixture table");
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "sample_a".to_string(),
            SampleMetadataRow {
                sample_id: "sample_a".to_string(),
                population_id: "cohort_alpha".to_string(),
                sex: "female".to_string(),
                role: "cohort".to_string(),
                description: "sample a".to_string(),
            },
        );
        let mut populations = BTreeMap::new();
        populations.insert("cohort_alpha".to_string(), "Cohort Alpha".to_string());
        let err = parse_admixture_rows(
            &table,
            &metadata,
            &populations,
            &["cluster_1".to_string(), "cluster_2".to_string()],
            2,
            "complete",
        )
        .expect_err("ancestry sum mismatch");
        assert!(err.to_string().contains("expected 1.0"));
    }

    #[test]
    fn governed_vcf_admixture_smoke_reports_complete_cohort_rows() {
        let repo_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).expect("repo root");
        let report =
            run_local_vcf_admixture_smoke(repo_root, "plink2").expect("run local admixture smoke");
        assert_eq!(report.stage_id, "vcf.admixture");
        assert_eq!(report.tool_id, "plink2");
        assert_eq!(report.corpus_id, "vcf_production_regression");
        assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
        assert_eq!(report.selected_k, 2);
        assert_eq!(report.status, "complete");
        assert_eq!(report.sample_count, 4);
        assert_eq!(report.population_count, 2);
        assert_eq!(report.cluster_headers, vec!["cluster_1", "cluster_2"]);
        assert_eq!(
            report.rows.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(),
            vec!["sample_a", "sample_b", "sample_c", "sample_d"]
        );
        assert!(
            report.rows.iter().all(|row| {
                row.clusters.values().copied().sum::<f64>() >= 1.0 - 1e-6
                    && row.clusters.values().copied().sum::<f64>() <= 1.0 + 1e-6
            }),
            "expected ancestry rows to stay normalized"
        );
    }
}
