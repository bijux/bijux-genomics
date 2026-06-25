use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob;
use super::local_hpc_execution_resolver::collect_hpc_execution_resolver_report;
use super::local_hpc_job_completion::{
    classify_local_hpc_job_completion, resolve_local_hpc_job_result_paths,
    LocalHpcJobCompletionState,
};
use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use super::local_hpc_simulation_tree::{
    remap_job_to_simulation_root, remove_stage_result_manifest, write_stage_result_manifest,
};
use super::local_stage_result_manifest::{path_relative_to_repo, BenchStageResultStatus};
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_HPC_RESULT_COLLECTION_SIMULATION_SCHEMA_VERSION: &str =
    "bijux.bench.local_hpc_result_collection_simulation.v1";
pub(crate) const DEFAULT_HPC_RESULT_COLLECTION_SIMULATION_PATH: &str =
    "runs/bench/hpc-dry-run/result-collection-simulation.json";
const DEFAULT_HPC_RESULT_COLLECTION_SIMULATION_TREE_NAME: &str =
    "result-collection-simulation-tree";

const COMPLETE_RESULT_JOB_ID: &str =
    "benchmark:vcf:vcf_production_regression:vcf.qc:vcf_cohort:bcftools";
const FAILED_RESULT_JOB_ID: &str =
    "benchmark:vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools";
const MISSING_RESULT_JOB_ID: &str =
    "benchmark:bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:contammix";
const INSUFFICIENT_RESULT_JOB_ID: &str = "pipeline:relatedness-segments-vcf:vcf.demography";
const UNAVAILABLE_RESULT_JOB_ID: &str = "pipeline:relatedness-segments-vcf:vcf.ibd";
const INSUFFICIENT_FIXTURE_PATH: &str =
    "benchmarks/tests/fixtures/bench/parsers/vcf/segments/ibdne/vcf.demography/insufficient_data/expected.normalized.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalHpcResultCollectionStatus {
    Complete,
    Failed,
    Missing,
    Insufficient,
    Unavailable,
}

impl LocalHpcResultCollectionStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Failed => "failed",
            Self::Missing => "missing",
            Self::Insufficient => "insufficient",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcResultCollectionSimulationRow {
    pub(crate) record_id: String,
    pub(crate) source_surface: String,
    pub(crate) collection_status: String,
    pub(crate) job_id_local: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) result_id: Option<String>,
    pub(crate) pipeline_id: Option<String>,
    pub(crate) node_id: Option<String>,
    pub(crate) evidence_path: String,
    pub(crate) manifest_path: Option<String>,
    pub(crate) declared_output_count: usize,
    pub(crate) present_output_count: usize,
    pub(crate) detail: String,
    pub(crate) insufficient_data_reason: Option<String>,
    pub(crate) unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcResultCollectionSimulationBehavior {
    pub(crate) complete_result_job_id: String,
    pub(crate) failed_result_job_id: String,
    pub(crate) missing_result_job_id: String,
    pub(crate) insufficient_result_job_id: String,
    pub(crate) unavailable_result_job_id: String,
    pub(crate) complete_row_present: bool,
    pub(crate) failed_row_present: bool,
    pub(crate) missing_row_present: bool,
    pub(crate) insufficient_row_present: bool,
    pub(crate) unavailable_row_present: bool,
    pub(crate) failed_distinct_from_missing: bool,
    pub(crate) insufficient_distinct_from_unavailable: bool,
    pub(crate) proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcResultCollectionSimulationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) simulation_root: String,
    pub(crate) execution_resolver_path: String,
    pub(crate) row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) failed_row_count: usize,
    pub(crate) missing_row_count: usize,
    pub(crate) insufficient_row_count: usize,
    pub(crate) unavailable_row_count: usize,
    pub(crate) behavior: LocalHpcResultCollectionSimulationBehavior,
    pub(crate) rows: Vec<LocalHpcResultCollectionSimulationRow>,
}

pub(crate) fn run_render_hpc_result_collection_simulation(
    args: &parse::BenchLocalRenderHpcResultCollectionSimulationArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths
            .resolve_repo_relative(Path::new(DEFAULT_HPC_RESULT_COLLECTION_SIMULATION_PATH))
    });
    let report = render_hpc_result_collection_simulation(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_result_collection_simulation(
    args: &parse::BenchLocalValidateHpcResultCollectionSimulationArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths
            .resolve_repo_relative(Path::new(DEFAULT_HPC_RESULT_COLLECTION_SIMULATION_PATH))
    });
    let report = validate_hpc_result_collection_simulation_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_result_collection_simulation(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcResultCollectionSimulationReport> {
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let report = build_hpc_result_collection_simulation(repo_root, &absolute_output)?;
    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output, &report)?;
    Ok(report)
}

pub(crate) fn collect_hpc_result_collection_simulation(
    repo_root: &Path,
) -> Result<LocalHpcResultCollectionSimulationReport> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let output_path = benchmark_paths
        .resolve_repo_relative(Path::new(DEFAULT_HPC_RESULT_COLLECTION_SIMULATION_PATH));
    build_hpc_result_collection_simulation(repo_root, &output_path)
}

pub(crate) fn validate_hpc_result_collection_simulation_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcResultCollectionSimulationReport> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let report = build_hpc_result_collection_simulation(repo_root, &absolute_manifest_path)?;
    let observed = fs::read(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let expected = serde_json::to_vec_pretty(&report)
        .context("serialize governed HPC result-collection simulation")?;
    if observed != expected {
        return Err(anyhow!(
            "HPC result-collection simulation `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-result-collection-simulation --output {}`",
            absolute_manifest_path.display(),
            report.output_path
        ));
    }
    Ok(report)
}

fn build_hpc_result_collection_simulation(
    repo_root: &Path,
    absolute_output_path: &Path,
) -> Result<LocalHpcResultCollectionSimulationReport> {
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        absolute_output_path,
        "HPC result-collection simulation output",
    )?;
    let simulation_root = absolute_output_path
        .parent()
        .ok_or_else(|| {
            anyhow!(
                "HPC result-collection simulation output `{}` has no parent",
                absolute_output_path.display()
            )
        })?
        .join(DEFAULT_HPC_RESULT_COLLECTION_SIMULATION_TREE_NAME);
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &simulation_root,
        "HPC result-collection simulation tree",
    )?;

    if simulation_root.exists() {
        fs::remove_dir_all(&simulation_root)
            .with_context(|| format!("remove {}", simulation_root.display()))?;
    }
    fs::create_dir_all(&simulation_root)
        .with_context(|| format!("create {}", simulation_root.display()))?;

    let selected_jobs = load_local_hpc_selected_jobs(repo_root)?;
    let jobs_by_id = selected_jobs
        .into_iter()
        .map(|job| (job.job_id_local.clone(), job))
        .collect::<BTreeMap<_, _>>();
    validate_seeded_job_presence(&jobs_by_id)?;

    let seeded_jobs = [
        COMPLETE_RESULT_JOB_ID,
        FAILED_RESULT_JOB_ID,
        MISSING_RESULT_JOB_ID,
        INSUFFICIENT_RESULT_JOB_ID,
    ]
    .into_iter()
    .map(|job_id_local| {
        let job = jobs_by_id.get(job_id_local).ok_or_else(|| {
            anyhow!("HPC result-collection simulation is missing seeded job `{job_id_local}`")
        })?;
        remap_job_to_simulation_root(repo_root, &simulation_root, job)
            .map(|simulated_job| (job_id_local.to_string(), simulated_job))
    })
    .collect::<Result<BTreeMap<_, _>>>()?;

    write_stage_result_manifest(
        repo_root,
        seeded_jobs.get(COMPLETE_RESULT_JOB_ID).expect("validated complete job"),
        BenchStageResultStatus::Succeeded,
    )?;
    write_stage_result_manifest(
        repo_root,
        seeded_jobs.get(FAILED_RESULT_JOB_ID).expect("validated failed job"),
        BenchStageResultStatus::Failed,
    )?;
    write_stage_result_manifest(
        repo_root,
        seeded_jobs.get(MISSING_RESULT_JOB_ID).expect("validated missing job"),
        BenchStageResultStatus::Succeeded,
    )?;
    remove_stage_result_manifest(
        repo_root,
        seeded_jobs.get(MISSING_RESULT_JOB_ID).expect("validated missing job"),
    )?;
    write_stage_result_manifest(
        repo_root,
        seeded_jobs.get(INSUFFICIENT_RESULT_JOB_ID).expect("validated insufficient job"),
        BenchStageResultStatus::Succeeded,
    )?;
    seed_insufficient_output_payload(
        repo_root,
        seeded_jobs.get(INSUFFICIENT_RESULT_JOB_ID).expect("validated insufficient job"),
    )?;

    let mut rows = vec![
        collect_result_row(
            repo_root,
            seeded_jobs.get(COMPLETE_RESULT_JOB_ID).expect("validated complete job"),
        )?,
        collect_result_row(
            repo_root,
            seeded_jobs.get(FAILED_RESULT_JOB_ID).expect("validated failed job"),
        )?,
        collect_result_row(
            repo_root,
            seeded_jobs.get(MISSING_RESULT_JOB_ID).expect("validated missing job"),
        )?,
        collect_result_row(
            repo_root,
            seeded_jobs.get(INSUFFICIENT_RESULT_JOB_ID).expect("validated insufficient job"),
        )?,
        collect_unavailable_row(
            &collect_hpc_execution_resolver_report(repo_root)?,
            UNAVAILABLE_RESULT_JOB_ID,
        )?,
    ];
    rows.sort_by(|left, right| left.record_id.cmp(&right.record_id));

    let complete_row_count = count_status(&rows, LocalHpcResultCollectionStatus::Complete);
    let failed_row_count = count_status(&rows, LocalHpcResultCollectionStatus::Failed);
    let missing_row_count = count_status(&rows, LocalHpcResultCollectionStatus::Missing);
    let insufficient_row_count = count_status(&rows, LocalHpcResultCollectionStatus::Insufficient);
    let unavailable_row_count = count_status(&rows, LocalHpcResultCollectionStatus::Unavailable);
    let behavior = build_behavior(&rows)?;
    let report = LocalHpcResultCollectionSimulationReport {
        schema_version: LOCAL_HPC_RESULT_COLLECTION_SIMULATION_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, absolute_output_path),
        simulation_root: path_relative_to_repo(repo_root, &simulation_root),
        execution_resolver_path:
            super::local_hpc_execution_resolver::DEFAULT_HPC_EXECUTION_RESOLVER_PATH.to_string(),
        row_count: rows.len(),
        complete_row_count,
        failed_row_count,
        missing_row_count,
        insufficient_row_count,
        unavailable_row_count,
        behavior,
        rows,
    };
    ensure_result_collection_simulation_contract(&report)?;
    Ok(report)
}

fn validate_seeded_job_presence(
    jobs_by_id: &BTreeMap<String, BenchLocalAllDomainSlurmSubmitJob>,
) -> Result<()> {
    for job_id_local in [
        COMPLETE_RESULT_JOB_ID,
        FAILED_RESULT_JOB_ID,
        MISSING_RESULT_JOB_ID,
        INSUFFICIENT_RESULT_JOB_ID,
        UNAVAILABLE_RESULT_JOB_ID,
    ] {
        if !jobs_by_id.contains_key(job_id_local) {
            return Err(anyhow!(
                "HPC result-collection simulation requires governed seeded job `{job_id_local}`"
            ));
        }
    }
    Ok(())
}

fn seed_insufficient_output_payload(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
) -> Result<()> {
    let result_paths = resolve_local_hpc_job_result_paths(repo_root, job)?;
    let insufficient_output = result_paths
        .declared_output_paths
        .iter()
        .find(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|file_name| file_name.contains("insufficient_data"))
        })
        .ok_or_else(|| {
            anyhow!(
                "HPC result-collection simulation requires an insufficient-data output for `{}`",
                job.job_id_local
            )
        })?;
    let fixture = fs::read_to_string(repo_root.join(INSUFFICIENT_FIXTURE_PATH))
        .with_context(|| format!("read {}", repo_root.join(INSUFFICIENT_FIXTURE_PATH).display()))?;
    fs::write(insufficient_output, fixture)
        .with_context(|| format!("write {}", insufficient_output.display()))?;
    Ok(())
}

fn collect_result_row(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
) -> Result<LocalHpcResultCollectionSimulationRow> {
    let result_paths = resolve_local_hpc_job_result_paths(repo_root, job)?;
    let classification = classify_local_hpc_job_completion(repo_root, job, &result_paths);
    match classification.state {
        LocalHpcJobCompletionState::ValidCompleted => {
            if let Some((evidence_path, insufficient_reason)) =
                detect_insufficient_output(repo_root, &result_paths.declared_output_paths)?
            {
                Ok(LocalHpcResultCollectionSimulationRow {
                    record_id: format!("result:{}", job.job_id_local),
                    source_surface: "stage_result_manifest".to_string(),
                    collection_status: LocalHpcResultCollectionStatus::Insufficient
                        .as_str()
                        .to_string(),
                    job_id_local: job.job_id_local.clone(),
                    domain: job.domain.clone(),
                    stage_id: job.stage_id.clone(),
                    tool_id: job.tool_id.clone(),
                    result_id: job.result_id.clone(),
                    pipeline_id: job.pipeline_id.clone(),
                    node_id: job.node_id.clone(),
                    evidence_path,
                    manifest_path: Some(path_relative_to_repo(
                        repo_root,
                        &result_paths.stage_result_manifest_path,
                    )),
                    declared_output_count: classification.declared_output_count,
                    present_output_count: classification.present_output_count,
                    detail: "governed simulated result carries explicit insufficient-data evidence"
                        .to_string(),
                    insufficient_data_reason: insufficient_reason,
                    unavailable_reason: None,
                })
            } else {
                Ok(LocalHpcResultCollectionSimulationRow {
                    record_id: format!("result:{}", job.job_id_local),
                    source_surface: "stage_result_manifest".to_string(),
                    collection_status: LocalHpcResultCollectionStatus::Complete
                        .as_str()
                        .to_string(),
                    job_id_local: job.job_id_local.clone(),
                    domain: job.domain.clone(),
                    stage_id: job.stage_id.clone(),
                    tool_id: job.tool_id.clone(),
                    result_id: job.result_id.clone(),
                    pipeline_id: job.pipeline_id.clone(),
                    node_id: job.node_id.clone(),
                    evidence_path: path_relative_to_repo(repo_root, &result_paths.result_root),
                    manifest_path: Some(path_relative_to_repo(
                        repo_root,
                        &result_paths.stage_result_manifest_path,
                    )),
                    declared_output_count: classification.declared_output_count,
                    present_output_count: classification.present_output_count,
                    detail: "governed simulated result is complete and report-ready".to_string(),
                    insufficient_data_reason: None,
                    unavailable_reason: None,
                })
            }
        }
        LocalHpcJobCompletionState::FailedStageResultManifest => {
            Ok(LocalHpcResultCollectionSimulationRow {
                record_id: format!("result:{}", job.job_id_local),
                source_surface: "stage_result_manifest".to_string(),
                collection_status: LocalHpcResultCollectionStatus::Failed.as_str().to_string(),
                job_id_local: job.job_id_local.clone(),
                domain: job.domain.clone(),
                stage_id: job.stage_id.clone(),
                tool_id: job.tool_id.clone(),
                result_id: job.result_id.clone(),
                pipeline_id: job.pipeline_id.clone(),
                node_id: job.node_id.clone(),
                evidence_path: path_relative_to_repo(
                    repo_root,
                    &result_paths.stage_result_manifest_path,
                ),
                manifest_path: Some(path_relative_to_repo(
                    repo_root,
                    &result_paths.stage_result_manifest_path,
                )),
                declared_output_count: classification.declared_output_count,
                present_output_count: classification.present_output_count,
                detail: "governed simulated stage-result manifest reports runtime failure"
                    .to_string(),
                insufficient_data_reason: None,
                unavailable_reason: None,
            })
        }
        LocalHpcJobCompletionState::MissingStageResultManifest => {
            Ok(LocalHpcResultCollectionSimulationRow {
                record_id: format!("result:{}", job.job_id_local),
                source_surface: "stage_result_manifest".to_string(),
                collection_status: LocalHpcResultCollectionStatus::Missing.as_str().to_string(),
                job_id_local: job.job_id_local.clone(),
                domain: job.domain.clone(),
                stage_id: job.stage_id.clone(),
                tool_id: job.tool_id.clone(),
                result_id: job.result_id.clone(),
                pipeline_id: job.pipeline_id.clone(),
                node_id: job.node_id.clone(),
                evidence_path: path_relative_to_repo(
                    repo_root,
                    &result_paths.stage_result_manifest_path,
                ),
                manifest_path: None,
                declared_output_count: classification.declared_output_count,
                present_output_count: classification.present_output_count,
                detail: "governed simulated result is missing the stage-result manifest"
                    .to_string(),
                insufficient_data_reason: None,
                unavailable_reason: None,
            })
        }
        LocalHpcJobCompletionState::InvalidStageResultManifest
        | LocalHpcJobCompletionState::StalePartialOutputs => Err(anyhow!(
            "HPC result-collection simulation seeded unexpected completion state `{}` for `{}`",
            classification.state.as_str(),
            job.job_id_local
        )),
    }
}

fn detect_insufficient_output(
    repo_root: &Path,
    output_paths: &[PathBuf],
) -> Result<Option<(String, Option<String>)>> {
    for output_path in output_paths {
        let file_name = output_path.file_name().and_then(|value| value.to_str()).unwrap_or("");
        if !file_name.contains("insufficient_data") {
            continue;
        }
        let payload = read_json_document(output_path)?;
        let normalized = payload.get("normalized").unwrap_or(&payload);
        let status = normalized.get("status").and_then(Value::as_str);
        let inference_status = normalized.get("inference_status").and_then(Value::as_str);
        if status == Some("insufficient_data") || inference_status == Some("insufficient_data") {
            return Ok(Some((
                path_relative_to_repo(repo_root, output_path),
                normalized
                    .get("insufficient_data_reason")
                    .or_else(|| normalized.get("insufficient_reason"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
            )));
        }
    }
    Ok(None)
}

fn collect_unavailable_row(
    resolver_report: &super::local_hpc_execution_resolver::LocalHpcExecutionResolverReport,
    job_id_local: &str,
) -> Result<LocalHpcResultCollectionSimulationRow> {
    let row = resolver_report
        .rows
        .iter()
        .find(|row| {
            row.selected_job_ids.iter().any(|candidate| candidate == job_id_local)
                && row.resolution_kind == "unavailable_with_reason"
        })
        .ok_or_else(|| {
            anyhow!(
                "HPC result-collection simulation requires unavailable resolver evidence for `{job_id_local}`"
            )
        })?;
    Ok(LocalHpcResultCollectionSimulationRow {
        record_id: format!("unavailable:{}", job_id_local),
        source_surface: "execution_resolver".to_string(),
        collection_status: LocalHpcResultCollectionStatus::Unavailable.as_str().to_string(),
        job_id_local: job_id_local.to_string(),
        domain: row.domains.join(","),
        stage_id: row.selected_stage_ids.join(","),
        tool_id: row.tool_id.clone(),
        result_id: None,
        pipeline_id: None,
        node_id: None,
        evidence_path: resolver_report.output_path.clone(),
        manifest_path: None,
        declared_output_count: 0,
        present_output_count: 0,
        detail: "governed execution resolver marks the selected HPC tool unavailable".to_string(),
        insufficient_data_reason: None,
        unavailable_reason: row.unavailable_reason.clone(),
    })
}

fn read_json_document(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn count_status(
    rows: &[LocalHpcResultCollectionSimulationRow],
    status: LocalHpcResultCollectionStatus,
) -> usize {
    rows.iter().filter(|row| row.collection_status == status.as_str()).count()
}

fn build_behavior(
    rows: &[LocalHpcResultCollectionSimulationRow],
) -> Result<LocalHpcResultCollectionSimulationBehavior> {
    let rows_by_record_id =
        rows.iter().map(|row| (row.record_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    let complete = rows_by_record_id
        .get(&format!("result:{COMPLETE_RESULT_JOB_ID}") as &str)
        .ok_or_else(|| anyhow!("missing complete result row"))?;
    let failed = rows_by_record_id
        .get(&format!("result:{FAILED_RESULT_JOB_ID}") as &str)
        .ok_or_else(|| anyhow!("missing failed result row"))?;
    let missing = rows_by_record_id
        .get(&format!("result:{MISSING_RESULT_JOB_ID}") as &str)
        .ok_or_else(|| anyhow!("missing missing-result row"))?;
    let insufficient = rows_by_record_id
        .get(&format!("result:{INSUFFICIENT_RESULT_JOB_ID}") as &str)
        .ok_or_else(|| anyhow!("missing insufficient row"))?;
    let unavailable = rows_by_record_id
        .get(&format!("unavailable:{UNAVAILABLE_RESULT_JOB_ID}") as &str)
        .ok_or_else(|| anyhow!("missing unavailable row"))?;
    let behavior = LocalHpcResultCollectionSimulationBehavior {
        complete_result_job_id: COMPLETE_RESULT_JOB_ID.to_string(),
        failed_result_job_id: FAILED_RESULT_JOB_ID.to_string(),
        missing_result_job_id: MISSING_RESULT_JOB_ID.to_string(),
        insufficient_result_job_id: INSUFFICIENT_RESULT_JOB_ID.to_string(),
        unavailable_result_job_id: UNAVAILABLE_RESULT_JOB_ID.to_string(),
        complete_row_present: complete.collection_status
            == LocalHpcResultCollectionStatus::Complete.as_str(),
        failed_row_present: failed.collection_status
            == LocalHpcResultCollectionStatus::Failed.as_str(),
        missing_row_present: missing.collection_status
            == LocalHpcResultCollectionStatus::Missing.as_str(),
        insufficient_row_present: insufficient.collection_status
            == LocalHpcResultCollectionStatus::Insufficient.as_str()
            && insufficient.insufficient_data_reason.is_some(),
        unavailable_row_present: unavailable.collection_status
            == LocalHpcResultCollectionStatus::Unavailable.as_str()
            && unavailable.unavailable_reason.is_some(),
        failed_distinct_from_missing: failed.collection_status != missing.collection_status
            && failed.manifest_path.is_some()
            && missing.manifest_path.is_none(),
        insufficient_distinct_from_unavailable: insufficient.collection_status
            != unavailable.collection_status
            && insufficient.insufficient_data_reason.is_some()
            && unavailable.unavailable_reason.is_some(),
        proven: false,
    };
    Ok(LocalHpcResultCollectionSimulationBehavior {
        proven: behavior.complete_row_present
            && behavior.failed_row_present
            && behavior.missing_row_present
            && behavior.insufficient_row_present
            && behavior.unavailable_row_present
            && behavior.failed_distinct_from_missing
            && behavior.insufficient_distinct_from_unavailable,
        ..behavior
    })
}

fn ensure_result_collection_simulation_contract(
    report: &LocalHpcResultCollectionSimulationReport,
) -> Result<()> {
    if report.row_count != 5 || report.rows.len() != 5 {
        return Err(anyhow!(
            "HPC result-collection simulation must keep exactly five governed report-input rows"
        ));
    }
    if report.complete_row_count != 1
        || report.failed_row_count != 1
        || report.missing_row_count != 1
        || report.insufficient_row_count != 1
        || report.unavailable_row_count != 1
    {
        return Err(anyhow!(
            "HPC result-collection simulation must keep one row for each governed collection status"
        ));
    }
    if !report.behavior.proven {
        return Err(anyhow!(
            "HPC result-collection simulation must prove complete, failed, missing, insufficient, and unavailable rows stay distinct"
        ));
    }
    Ok(())
}
