use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_stages_vcf::pipeline::{
    run_population_structure_stage, AdmixtureStageParams, PopulationStructureStageParams,
};
use serde::Serialize;

use super::local_stage_result_manifest::{
    path_relative_to_repo, validate_stage_result_manifest, BenchStageResultCommandV1,
    BenchStageResultManifestV1, BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::local_vcf_admixture_smoke::run_local_vcf_admixture_smoke;
use super::local_vcf_pca_smoke::run_local_vcf_pca_smoke;
use super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_POPULATION_STRUCTURE_SMOKE_ROOT: &str =
    "runs/bench/local-smoke/vcf.population_structure";
const LOCAL_VCF_POPULATION_STRUCTURE_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_population_structure_smoke.v1";
const LOCAL_VCF_POPULATION_STRUCTURE_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-population-structure-smoke";
const GOVERNED_VCF_POPULATION_STRUCTURE_STAGE_ID: &str = "vcf.population_structure";
const GOVERNED_VCF_POPULATION_STRUCTURE_TOOL_ID: &str = "plink2";
const GOVERNED_VCF_POPULATION_STRUCTURE_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_POPULATION_STRUCTURE_ASSET_PROFILE_ID: &str = "vcf_cohort";
const GOVERNED_VCF_POPULATION_STRUCTURE_INPUT_FIXTURE_ID: &str = "vcf_mini_multisample_cohort";
const DEFAULT_OUTPUT_REPORT_NAME: &str = "population_structure.json";
const DEFAULT_OUTPUT_SOURCE_STAGE_REPORT_NAME: &str = "source_population_structure.json";
const DEFAULT_OUTPUT_SOURCE_PRUNED_VARIANTS_NAME: &str = "source_pruned_variants.tsv";
const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
const DEFAULT_OUTPUT_SOURCE_PCA_REPORT_NAME: &str = "source_pca.json";
const DEFAULT_OUTPUT_SOURCE_ADMIXTURE_REPORT_NAME: &str = "source_admixture.json";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfPopulationStructureSmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct ConsumedPcaReport {
    pub(crate) report_path: String,
    pub(crate) sample_count: u64,
    pub(crate) execution_mode: String,
    pub(crate) tool_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct ConsumedAdmixtureReport {
    pub(crate) report_path: String,
    pub(crate) sample_count: u64,
    pub(crate) selected_k: u64,
    pub(crate) execution_mode: String,
    pub(crate) tool_ok: bool,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfPopulationStructureSampleGroup {
    pub(crate) sample_id: String,
    pub(crate) population_id: String,
    pub(crate) population_label: String,
    pub(crate) sex: String,
    pub(crate) dominant_cluster: String,
    pub(crate) dominant_fraction: f64,
    pub(crate) pc1: f64,
    pub(crate) pc2: f64,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfPopulationStructureDistanceSummary {
    pub(crate) sample_count: u64,
    pub(crate) pair_count: u64,
    pub(crate) within_population_pair_count: u64,
    pub(crate) cross_population_pair_count: u64,
    pub(crate) min_pc_distance: f64,
    pub(crate) max_pc_distance: f64,
    pub(crate) mean_pc_distance: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfPopulationStructureSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) output_root: String,
    pub(crate) population_structure_json_path: String,
    pub(crate) source_population_structure_path: String,
    pub(crate) source_pruned_variants_path: String,
    pub(crate) source_logs_path: String,
    pub(crate) source_pca_report_path: String,
    pub(crate) source_admixture_report_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) consumed_pca: ConsumedPcaReport,
    pub(crate) consumed_admixture: ConsumedAdmixtureReport,
    pub(crate) sample_groups: Vec<LocalVcfPopulationStructureSampleGroup>,
    pub(crate) distance_summary: LocalVcfPopulationStructureDistanceSummary,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq)]
struct PcaSampleRow {
    sample_id: String,
    population_id: String,
    population_label: String,
    sex: String,
    pc1: f64,
    pc2: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct AdmixtureSampleRow {
    sample_id: String,
    population_id: String,
    population_label: String,
    sex: String,
    status: String,
    clusters: Vec<(String, f64)>,
}

pub(crate) fn run_vcf_population_structure_smoke(
    args: &parse::BenchLocalRunVcfPopulationStructureSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_population_structure_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.population_structure_json_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_population_structure_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfPopulationStructureSmokeReport> {
    let contract = resolve_governed_vcf_population_structure_smoke_contract(tool_id)?;
    let pca_report = run_local_vcf_pca_smoke(repo_root, GOVERNED_VCF_POPULATION_STRUCTURE_TOOL_ID)?;
    let admixture_report =
        run_local_vcf_admixture_smoke(repo_root, GOVERNED_VCF_POPULATION_STRUCTURE_TOOL_ID)?;

    let upstream_input_vcf = repo_root.join(&pca_report.input_vcf_path);
    let upstream_population_labels_manifest =
        repo_root.join(&pca_report.population_labels_manifest_path);
    ensure_file_exists(&upstream_input_vcf, "population-structure upstream input VCF")?;
    ensure_file_exists(
        &upstream_population_labels_manifest,
        "population-structure upstream metadata manifest",
    )?;

    let output_root =
        repo_root.join(DEFAULT_VCF_POPULATION_STRUCTURE_SMOKE_ROOT).join(&contract.tool_id);
    if output_root.exists() {
        fs::remove_dir_all(&output_root)
            .with_context(|| format!("remove {}", output_root.display()))?;
    }
    let artifacts_root = output_root.join("artifacts");
    let stage_root = artifacts_root.join("stage");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_population_structure_stage(
        &upstream_input_vcf,
        &stage_root,
        &PopulationStructureStageParams {
            toolchain: contract.tool_id.clone(),
            smartpca: true,
            run_admixture: true,
            sample_metadata_manifest: Some(upstream_population_labels_manifest.clone()),
            admixture_params: Some(AdmixtureStageParams {
                toolchain: contract.tool_id.clone(),
                k_values: vec![2, 3, 4],
                sample_metadata_manifest: Some(upstream_population_labels_manifest.clone()),
            }),
            preprocessing: Default::default(),
        },
    )
    .with_context(|| {
        format!("run governed VCF population-structure smoke from {}", upstream_input_vcf.display())
    })?;

    let source_population_structure_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_STAGE_REPORT_NAME);
    fs::copy(&stage_outputs.population_structure_json, &source_population_structure_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.population_structure_json.display(),
                source_population_structure_path.display()
            )
        })?;
    let source_pruned_variants_path = output_root.join(DEFAULT_OUTPUT_SOURCE_PRUNED_VARIANTS_NAME);
    fs::copy(&stage_outputs.pruned_variants_tsv, &source_pruned_variants_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.pruned_variants_tsv.display(),
                source_pruned_variants_path.display()
            )
        },
    )?;
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    fs::copy(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let source_pca_report_source = repo_root.join(&pca_report.pca_json_path);
    let source_admixture_report_source = repo_root.join(&admixture_report.admixture_json_path);
    ensure_file_exists(&source_pca_report_source, "consumed PCA report")?;
    ensure_file_exists(&source_admixture_report_source, "consumed admixture report")?;

    let source_pca_report_path = output_root.join(DEFAULT_OUTPUT_SOURCE_PCA_REPORT_NAME);
    fs::copy(&source_pca_report_source, &source_pca_report_path).with_context(|| {
        format!(
            "copy {} to {}",
            source_pca_report_source.display(),
            source_pca_report_path.display()
        )
    })?;
    let source_admixture_report_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_ADMIXTURE_REPORT_NAME);
    fs::copy(&source_admixture_report_source, &source_admixture_report_path).with_context(
        || {
            format!(
                "copy {} to {}",
                source_admixture_report_source.display(),
                source_admixture_report_path.display()
            )
        },
    )?;

    let consumed_pca_json = read_json(&source_pca_report_path)?;
    let consumed_admixture_json = read_json(&source_admixture_report_path)?;
    let source_population_structure_json = read_json(&source_population_structure_path)?;

    let consumed_pca =
        summarize_consumed_pca(repo_root, &source_pca_report_path, &consumed_pca_json)?;
    let consumed_admixture = summarize_consumed_admixture(
        repo_root,
        &source_admixture_report_path,
        &consumed_admixture_json,
    )?;
    let pca_rows = parse_pca_rows(&consumed_pca_json)?;
    let admixture_rows = parse_admixture_rows(&consumed_admixture_json)?;
    validate_consumed_stage_inputs(&source_population_structure_json, &pca_rows, &admixture_rows)?;
    let sample_groups = build_sample_groups(&pca_rows, &admixture_rows)?;
    let distance_summary = build_distance_summary(&sample_groups)?;
    let status = source_population_structure_json
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("complete")
        .to_string();

    let population_structure_json_path = output_root.join(DEFAULT_OUTPUT_REPORT_NAME);
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let report = LocalVcfPopulationStructureSmokeReport {
        schema_version: LOCAL_VCF_POPULATION_STRUCTURE_SMOKE_SCHEMA_VERSION,
        command: format!(
            "{LOCAL_VCF_POPULATION_STRUCTURE_SMOKE_COMMAND} --tool-id {}",
            contract.tool_id
        ),
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        output_root: path_relative_to_repo(repo_root, &output_root),
        population_structure_json_path: path_relative_to_repo(
            repo_root,
            &population_structure_json_path,
        ),
        source_population_structure_path: path_relative_to_repo(
            repo_root,
            &source_population_structure_path,
        ),
        source_pruned_variants_path: path_relative_to_repo(repo_root, &source_pruned_variants_path),
        source_logs_path: path_relative_to_repo(repo_root, &source_logs_path),
        source_pca_report_path: path_relative_to_repo(repo_root, &source_pca_report_path),
        source_admixture_report_path: path_relative_to_repo(
            repo_root,
            &source_admixture_report_path,
        ),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at: started_at.clone(),
        finished_at: finished_at.clone(),
        elapsed_seconds,
        exit_code: 0,
        consumed_pca,
        consumed_admixture,
        sample_groups,
        distance_summary,
        status,
    };
    bijux_dna_infra::atomic_write_json(&population_structure_json_path, &report)?;

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
                artifact_id: "population_structure_json".to_string(),
                declared_path: DEFAULT_OUTPUT_REPORT_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &population_structure_json_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_population_structure_json".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_STAGE_REPORT_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_population_structure_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_pruned_variants_tsv".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_PRUNED_VARIANTS_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_pruned_variants_path),
                role: "table_output".to_string(),
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
            BenchStageResultOutputV1 {
                artifact_id: "source_pca_report_json".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_PCA_REPORT_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_pca_report_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_admixture_report_json".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_ADMIXTURE_REPORT_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_admixture_report_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
        ],
    };
    validate_stage_result_manifest(&stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    Ok(report)
}

fn resolve_governed_vcf_population_structure_smoke_contract(
    tool_id: &str,
) -> Result<GovernedVcfPopulationStructureSmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_POPULATION_STRUCTURE_STAGE_ID)
        .ok_or_else(|| {
            anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_POPULATION_STRUCTURE_STAGE_ID}`")
        })?;
    if tool_id != matrix_row.tool_id {
        bail!(
            "VCF population-structure smoke only retains tool `{}` for `{}`; requested `{tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.tool_id != GOVERNED_VCF_POPULATION_STRUCTURE_TOOL_ID {
        bail!(
            "VCF population-structure smoke requires tool `{GOVERNED_VCF_POPULATION_STRUCTURE_TOOL_ID}`, found `{}`",
            matrix_row.tool_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_POPULATION_STRUCTURE_CORPUS_ID {
        bail!(
            "VCF population-structure smoke requires corpus `{GOVERNED_VCF_POPULATION_STRUCTURE_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_POPULATION_STRUCTURE_ASSET_PROFILE_ID {
        bail!(
            "VCF population-structure smoke requires asset profile `{GOVERNED_VCF_POPULATION_STRUCTURE_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["population_structure_report".to_string()] {
        bail!(
            "VCF population-structure smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfPopulationStructureSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_POPULATION_STRUCTURE_INPUT_FIXTURE_ID.to_string(),
    })
}

fn summarize_consumed_pca(
    repo_root: &Path,
    report_path: &Path,
    payload: &serde_json::Value,
) -> Result<ConsumedPcaReport> {
    Ok(ConsumedPcaReport {
        report_path: path_relative_to_repo(repo_root, report_path),
        sample_count: payload
            .get("sample_count")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| anyhow!("consumed PCA report missing `sample_count`"))?,
        execution_mode: payload
            .get("execution_mode")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("consumed PCA report missing `execution_mode`"))?
            .to_string(),
        tool_ok: payload
            .get("tool_ok")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("consumed PCA report missing `tool_ok`"))?,
    })
}

fn summarize_consumed_admixture(
    repo_root: &Path,
    report_path: &Path,
    payload: &serde_json::Value,
) -> Result<ConsumedAdmixtureReport> {
    Ok(ConsumedAdmixtureReport {
        report_path: path_relative_to_repo(repo_root, report_path),
        sample_count: payload
            .get("sample_count")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| anyhow!("consumed admixture report missing `sample_count`"))?,
        selected_k: payload
            .get("selected_k")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| anyhow!("consumed admixture report missing `selected_k`"))?,
        execution_mode: payload
            .get("execution_mode")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("consumed admixture report missing `execution_mode`"))?
            .to_string(),
        tool_ok: payload
            .get("tool_ok")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("consumed admixture report missing `tool_ok`"))?,
        status: payload
            .get("status")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("consumed admixture report missing `status`"))?
            .to_string(),
    })
}

fn parse_pca_rows(payload: &serde_json::Value) -> Result<Vec<PcaSampleRow>> {
    payload
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("consumed PCA report missing `rows`"))?
        .iter()
        .map(|row| {
            Ok(PcaSampleRow {
                sample_id: row
                    .get("sample_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed PCA row missing `sample_id`"))?
                    .to_string(),
                population_id: row
                    .get("population_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed PCA row missing `population_id`"))?
                    .to_string(),
                population_label: row
                    .get("population_label")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed PCA row missing `population_label`"))?
                    .to_string(),
                sex: row
                    .get("sex")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed PCA row missing `sex`"))?
                    .to_string(),
                pc1: row
                    .get("pc1")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or_else(|| anyhow!("consumed PCA row missing `pc1`"))?,
                pc2: row
                    .get("pc2")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or_else(|| anyhow!("consumed PCA row missing `pc2`"))?,
            })
        })
        .collect::<Result<Vec<_>>>()
}

fn parse_admixture_rows(payload: &serde_json::Value) -> Result<Vec<AdmixtureSampleRow>> {
    let cluster_headers = payload
        .get("cluster_headers")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("consumed admixture report missing `cluster_headers`"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("consumed admixture cluster headers must be strings"))
        })
        .collect::<Result<Vec<_>>>()?;
    payload
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("consumed admixture report missing `rows`"))?
        .iter()
        .map(|row| {
            let mut clusters = Vec::new();
            for header in &cluster_headers {
                clusters.push((
                    header.clone(),
                    row.get(header)
                        .and_then(serde_json::Value::as_f64)
                        .ok_or_else(|| anyhow!("consumed admixture row missing `{header}`"))?,
                ));
            }
            Ok(AdmixtureSampleRow {
                sample_id: row
                    .get("sample_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed admixture row missing `sample_id`"))?
                    .to_string(),
                population_id: row
                    .get("population_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed admixture row missing `population_id`"))?
                    .to_string(),
                population_label: row
                    .get("population_label")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed admixture row missing `population_label`"))?
                    .to_string(),
                sex: row
                    .get("sex")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed admixture row missing `sex`"))?
                    .to_string(),
                status: row
                    .get("status")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed admixture row missing `status`"))?
                    .to_string(),
                clusters,
            })
        })
        .collect::<Result<Vec<_>>>()
}

fn validate_consumed_stage_inputs(
    stage_payload: &serde_json::Value,
    pca_rows: &[PcaSampleRow],
    admixture_rows: &[AdmixtureSampleRow],
) -> Result<()> {
    let stage_sample_ids = stage_payload
        .get("sample_ids")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("population-structure stage report missing `sample_ids`"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("population-structure stage sample_ids must be strings"))
        })
        .collect::<Result<Vec<_>>>()?;
    let pca_sample_ids = pca_rows.iter().map(|row| row.sample_id.clone()).collect::<Vec<_>>();
    let admixture_sample_ids =
        admixture_rows.iter().map(|row| row.sample_id.clone()).collect::<Vec<_>>();
    if stage_sample_ids != pca_sample_ids {
        bail!(
            "population-structure stage sample_ids drifted from consumed PCA rows: {:?} vs {:?}",
            stage_sample_ids,
            pca_sample_ids
        );
    }
    if stage_sample_ids != admixture_sample_ids {
        bail!(
            "population-structure stage sample_ids drifted from consumed admixture rows: {:?} vs {:?}",
            stage_sample_ids,
            admixture_sample_ids
        );
    }
    Ok(())
}

fn build_sample_groups(
    pca_rows: &[PcaSampleRow],
    admixture_rows: &[AdmixtureSampleRow],
) -> Result<Vec<LocalVcfPopulationStructureSampleGroup>> {
    let mut groups = Vec::new();
    for (pca_row, admixture_row) in pca_rows.iter().zip(admixture_rows.iter()) {
        if pca_row.sample_id != admixture_row.sample_id {
            bail!(
                "consumed PCA/admixture sample order drifted: `{}` vs `{}`",
                pca_row.sample_id,
                admixture_row.sample_id
            );
        }
        let (dominant_cluster, dominant_fraction) = admixture_row
            .clusters
            .iter()
            .max_by(|left, right| left.1.partial_cmp(&right.1).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .ok_or_else(|| anyhow!("admixture row is missing cluster values"))?;
        groups.push(LocalVcfPopulationStructureSampleGroup {
            sample_id: pca_row.sample_id.clone(),
            population_id: pca_row.population_id.clone(),
            population_label: pca_row.population_label.clone(),
            sex: pca_row.sex.clone(),
            dominant_cluster,
            dominant_fraction,
            pc1: pca_row.pc1,
            pc2: pca_row.pc2,
            status: admixture_row.status.clone(),
        });
    }
    Ok(groups)
}

fn build_distance_summary(
    sample_groups: &[LocalVcfPopulationStructureSampleGroup],
) -> Result<LocalVcfPopulationStructureDistanceSummary> {
    if sample_groups.is_empty() {
        bail!("population-structure smoke requires at least one sample group");
    }
    let mut distances = Vec::new();
    let mut within_population_pair_count = 0_u64;
    let mut cross_population_pair_count = 0_u64;
    for left_index in 0..sample_groups.len() {
        for right_index in (left_index + 1)..sample_groups.len() {
            let left = &sample_groups[left_index];
            let right = &sample_groups[right_index];
            let delta_pc1 = left.pc1 - right.pc1;
            let delta_pc2 = left.pc2 - right.pc2;
            let distance = (delta_pc1.powi(2) + delta_pc2.powi(2)).sqrt();
            distances.push(distance);
            if left.population_id == right.population_id {
                within_population_pair_count += 1;
            } else {
                cross_population_pair_count += 1;
            }
        }
    }
    let pair_count = u64::try_from(distances.len()).map_err(|_| anyhow!("pair count overflow"))?;
    let sample_count =
        u64::try_from(sample_groups.len()).map_err(|_| anyhow!("sample count overflow"))?;
    let (min_pc_distance, max_pc_distance, mean_pc_distance) = if distances.is_empty() {
        (0.0, 0.0, 0.0)
    } else {
        let min =
            distances.iter().copied().fold(f64::INFINITY, |current, value| current.min(value));
        let max =
            distances.iter().copied().fold(f64::NEG_INFINITY, |current, value| current.max(value));
        let mean = distances.iter().copied().sum::<f64>() / distances.len() as f64;
        (min, max, mean)
    };
    Ok(LocalVcfPopulationStructureDistanceSummary {
        sample_count,
        pair_count,
        within_population_pair_count,
        cross_population_pair_count,
        min_pc_distance,
        max_pc_distance,
        mean_pc_distance,
    })
}

fn ensure_file_exists(path: &Path, label: &str) -> Result<()> {
    if !path.is_file() {
        bail!("{label} is missing: {}", path.display());
    }
    Ok(())
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
    use super::{
        build_distance_summary, ensure_file_exists, run_local_vcf_population_structure_smoke,
        LocalVcfPopulationStructureSampleGroup,
    };

    #[test]
    fn consumed_report_paths_must_exist() {
        let temp = tempfile::tempdir().expect("tempdir");
        let missing = temp.path().join("missing.json");
        let err = ensure_file_exists(&missing, "consumed PCA report").expect_err("missing path");
        assert!(err.to_string().contains("consumed PCA report"));
    }

    #[test]
    fn distance_summary_counts_within_and_cross_population_pairs() {
        let rows = vec![
            LocalVcfPopulationStructureSampleGroup {
                sample_id: "sample_a".to_string(),
                population_id: "cohort_alpha".to_string(),
                population_label: "Cohort Alpha".to_string(),
                sex: "female".to_string(),
                dominant_cluster: "cluster_1".to_string(),
                dominant_fraction: 1.0,
                pc1: 0.0,
                pc2: 0.0,
                status: "complete".to_string(),
            },
            LocalVcfPopulationStructureSampleGroup {
                sample_id: "sample_b".to_string(),
                population_id: "cohort_alpha".to_string(),
                population_label: "Cohort Alpha".to_string(),
                sex: "male".to_string(),
                dominant_cluster: "cluster_1".to_string(),
                dominant_fraction: 1.0,
                pc1: 0.1,
                pc2: 0.0,
                status: "complete".to_string(),
            },
            LocalVcfPopulationStructureSampleGroup {
                sample_id: "sample_c".to_string(),
                population_id: "cohort_beta".to_string(),
                population_label: "Cohort Beta".to_string(),
                sex: "female".to_string(),
                dominant_cluster: "cluster_2".to_string(),
                dominant_fraction: 1.0,
                pc1: 1.0,
                pc2: 0.0,
                status: "complete".to_string(),
            },
        ];
        let summary = build_distance_summary(&rows).expect("distance summary");
        assert_eq!(summary.sample_count, 3);
        assert_eq!(summary.pair_count, 3);
        assert_eq!(summary.within_population_pair_count, 1);
        assert_eq!(summary.cross_population_pair_count, 2);
        assert!(summary.max_pc_distance >= summary.min_pc_distance);
    }

    #[test]
    fn governed_vcf_population_structure_smoke_consumes_pca_and_admixture_reports() {
        let repo_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).expect("repo root");
        let report = run_local_vcf_population_structure_smoke(repo_root, "plink2")
            .expect("run local population structure smoke");
        assert_eq!(report.stage_id, "vcf.population_structure");
        assert_eq!(report.tool_id, "plink2");
        assert_eq!(report.corpus_id, "vcf_production_regression");
        assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
        assert_eq!(report.consumed_pca.sample_count, 4);
        assert_eq!(report.consumed_admixture.sample_count, 4);
        assert_eq!(report.consumed_admixture.selected_k, 2);
        assert_eq!(report.status, "complete");
        assert_eq!(report.sample_groups.len(), 4);
        assert_eq!(report.distance_summary.sample_count, 4);
        assert_eq!(report.distance_summary.pair_count, 6);
    }
}
