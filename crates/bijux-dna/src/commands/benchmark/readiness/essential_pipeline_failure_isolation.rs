use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_essential_pipeline_fake_runs::{
    fake_run_essential_pipelines, EssentialPipelineFakeRunNodeReport,
    EssentialPipelineFakeRunOutputEntry, EssentialPipelineFakeRunsReport,
};
use crate::commands::benchmark::local_pipeline_dag::{
    validate_pipeline_dag_path, LocalPipelineDagValidationNodeReport,
};
use crate::commands::benchmark::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, path_relative_to_repo, BenchStageResultStatus,
};
use crate::commands::benchmark::readiness::essential_pipeline_corpus_assets::ESSENTIAL_PIPELINE_IDS;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ESSENTIAL_PIPELINE_FAILURE_ISOLATION_REPORT_PATH: &str =
    "target/bench-readiness/essential-pipeline-failure-isolation.json";
const DEFAULT_ESSENTIAL_PIPELINE_FAILURE_ISOLATION_SIMULATION_ROOT: &str =
    "target/bench-readiness/essential-pipeline-failure-isolation-tree";
const ESSENTIAL_PIPELINE_FAILURE_ISOLATION_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.essential_pipeline_failure_isolation.v1";
const SEEDED_FAILED_PIPELINE_ID: &str = "relatedness-segments-vcf";
const SEEDED_FAILED_NODE_ID: &str = "vcf.ibd";
const SEEDED_BLOCKED_NODE_ID: &str = "vcf.demography";
const SEEDED_CONTINUED_NODE_ID: &str = "vcf.roh";
const SEEDED_FAILURE_EXIT_CODE: i32 = 17;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionState {
    Completed,
    Failed,
    Blocked,
}

impl ExecutionState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone)]
struct FailureClassification {
    state: ExecutionState,
    detail: String,
    exit_code: i32,
    outputs_present: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EssentialPipelineFailureIsolationRow {
    pub(crate) pipeline_id: String,
    pub(crate) node_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) execution_state: String,
    pub(crate) reason: String,
    pub(crate) dependency_count: usize,
    pub(crate) depends_on: Vec<String>,
    pub(crate) stage_result_path: String,
    pub(crate) exit_code: i32,
    pub(crate) outputs_present: bool,
    pub(crate) unrelated_branch_continues: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EssentialPipelineFailureIsolationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) simulation_root: String,
    pub(crate) pipeline_count: usize,
    pub(crate) node_count: usize,
    pub(crate) completed_node_count: usize,
    pub(crate) failed_node_count: usize,
    pub(crate) blocked_node_count: usize,
    pub(crate) seeded_failed_node_id: String,
    pub(crate) blocked_descendant_node_ids: Vec<String>,
    pub(crate) continued_unrelated_node_ids: Vec<String>,
    pub(crate) passes_behavior_test: bool,
    pub(crate) rows: Vec<EssentialPipelineFailureIsolationRow>,
}

pub(crate) fn run_render_essential_pipeline_failure_isolation(
    args: &parse::BenchReadinessRenderEssentialPipelineFailureIsolationArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_essential_pipeline_failure_isolation(
        &repo_root,
        args.output.clone().unwrap_or_else(|| {
            PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_FAILURE_ISOLATION_REPORT_PATH)
        }),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_essential_pipeline_failure_isolation(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<EssentialPipelineFailureIsolationReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let simulation_root = repo_relative_path(
        repo_root,
        &PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_FAILURE_ISOLATION_SIMULATION_ROOT),
    );
    let fake_run_report = fake_run_essential_pipelines(repo_root, simulation_root.clone())?;
    inject_failed_stage_result(repo_root, &fake_run_report)?;
    let rows = collect_failure_isolation_rows(repo_root, &fake_run_report)?;
    let report = build_report(repo_root, &output_path, &simulation_root, rows)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

fn inject_failed_stage_result(
    repo_root: &Path,
    fake_run_report: &EssentialPipelineFakeRunsReport,
) -> Result<()> {
    let seeded_node = fake_run_report
        .pipelines
        .iter()
        .find(|pipeline| pipeline.pipeline_id == SEEDED_FAILED_PIPELINE_ID)
        .and_then(|pipeline| pipeline.nodes.iter().find(|node| node.node_id == SEEDED_FAILED_NODE_ID))
        .ok_or_else(|| {
            anyhow!(
                "essential pipeline failure-isolation simulation cannot find seeded failed node `{SEEDED_FAILED_PIPELINE_ID}` / `{SEEDED_FAILED_NODE_ID}`"
            )
        })?;

    remove_realized_outputs(repo_root, &seeded_node.outputs)?;
    let stage_result_path = repo_root.join(&seeded_node.stage_result_path);
    let mut payload: serde_json::Value = serde_json::from_slice(
        &fs::read(&stage_result_path)
            .with_context(|| format!("read {}", stage_result_path.display()))?,
    )
    .with_context(|| format!("parse {}", stage_result_path.display()))?;
    payload["runtime"]["status"] = serde_json::Value::String("failed".to_string());
    payload["runtime"]["exit_code"] = serde_json::Value::from(SEEDED_FAILURE_EXIT_CODE);
    if let Some(outputs) = payload.get_mut("outputs").and_then(serde_json::Value::as_array_mut) {
        for output in outputs {
            output["exists"] = serde_json::Value::Bool(false);
        }
    }
    bijux_dna_infra::atomic_write_json(&stage_result_path, &payload)
        .with_context(|| format!("write {}", stage_result_path.display()))?;
    Ok(())
}

fn remove_realized_outputs(
    repo_root: &Path,
    outputs: &[EssentialPipelineFakeRunOutputEntry],
) -> Result<()> {
    for output in outputs {
        let path = repo_root.join(&output.fake_run_path);
        if path.is_dir() {
            fs::remove_dir_all(&path).with_context(|| format!("remove {}", path.display()))?;
        } else if path.exists() {
            fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
        }
    }
    Ok(())
}

fn collect_failure_isolation_rows(
    repo_root: &Path,
    fake_run_report: &EssentialPipelineFakeRunsReport,
) -> Result<Vec<EssentialPipelineFailureIsolationRow>> {
    let fake_nodes = fake_run_report
        .pipelines
        .iter()
        .flat_map(|pipeline| {
            pipeline
                .nodes
                .iter()
                .map(|node| ((pipeline.pipeline_id.clone(), node.node_id.clone()), node.clone()))
        })
        .collect::<BTreeMap<(String, String), EssentialPipelineFakeRunNodeReport>>();

    let mut rows = Vec::new();
    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        let dag_report = validate_pipeline_dag_path(
            repo_root,
            &crate::commands::benchmark::local_pipeline_dag::benchmark_local_pipeline_config_path(
                repo_root,
                pipeline_id,
            ),
            &repo_root.join("target/local-ready/pipeline-dag").join(format!("{pipeline_id}.json")),
        )?;
        let nodes_by_id = dag_report
            .nodes
            .iter()
            .map(|node| (node.node_id.clone(), node.clone()))
            .collect::<BTreeMap<String, LocalPipelineDagValidationNodeReport>>();

        let mut classifications = BTreeMap::<String, FailureClassification>::new();
        for node_id in &dag_report.topological_order {
            let fake_node = fake_nodes
                .get(&(pipeline_id.to_string(), node_id.clone()))
                .ok_or_else(|| {
                    anyhow!(
                        "essential pipeline failure-isolation simulation is missing fake-run node `{pipeline_id}` / `{node_id}`"
                    )
                })?;
            let dag_node = nodes_by_id.get(node_id).ok_or_else(|| {
                anyhow!(
                    "essential pipeline failure-isolation simulation is missing DAG node `{pipeline_id}` / `{node_id}`"
                )
            })?;
            classifications
                .insert(node_id.clone(), classify_failure_state(repo_root, dag_node, fake_node));
        }

        let mut failed_or_blocked_nodes = BTreeSet::<String>::new();
        for node_id in &dag_report.topological_order {
            let dag_node = nodes_by_id.get(node_id).expect("validated node");
            let classification = classifications.get(node_id).expect("classified node");
            let (state, reason) = if classification.state == ExecutionState::Failed {
                failed_or_blocked_nodes.insert(node_id.clone());
                (ExecutionState::Failed, classification.detail.clone())
            } else if dag_node
                .depends_on
                .iter()
                .any(|dependency| failed_or_blocked_nodes.contains(dependency))
            {
                failed_or_blocked_nodes.insert(node_id.clone());
                (ExecutionState::Blocked, "failed_dependency_blocked".to_string())
            } else {
                (ExecutionState::Completed, classification.detail.clone())
            };
            let fake_node =
                fake_nodes.get(&(pipeline_id.to_string(), node_id.clone())).expect("fake node");
            rows.push(EssentialPipelineFailureIsolationRow {
                pipeline_id: (*pipeline_id).to_string(),
                node_id: node_id.clone(),
                stage_id: dag_node.stage_id.clone(),
                tool_id: fake_node.tool_id.clone(),
                execution_state: state.as_str().to_string(),
                reason,
                dependency_count: dag_node.depends_on.len(),
                depends_on: dag_node.depends_on.clone(),
                stage_result_path: fake_node.stage_result_path.clone(),
                exit_code: classification.exit_code,
                outputs_present: classification.outputs_present,
                unrelated_branch_continues: *pipeline_id == SEEDED_FAILED_PIPELINE_ID
                    && node_id == SEEDED_CONTINUED_NODE_ID
                    && state == ExecutionState::Completed,
            });
        }
    }

    rows.sort_by(|left, right| {
        left.pipeline_id
            .cmp(&right.pipeline_id)
            .then_with(|| left.node_id.cmp(&right.node_id))
            .then_with(|| left.stage_id.cmp(&right.stage_id))
    });
    Ok(rows)
}

fn classify_failure_state(
    repo_root: &Path,
    dag_node: &LocalPipelineDagValidationNodeReport,
    fake_node: &EssentialPipelineFakeRunNodeReport,
) -> FailureClassification {
    let stage_result_path = repo_root.join(&fake_node.stage_result_path);
    match load_validated_stage_result_manifest_path(&stage_result_path) {
        Ok(manifest) => {
            let outputs_present = manifest
                .outputs
                .iter()
                .all(|output| output.exists && repo_root.join(&output.realized_path).exists());
            let is_failed = manifest.runtime.status == BenchStageResultStatus::Failed
                || manifest.runtime.exit_code != 0
                || !outputs_present
                || manifest.stage_id != dag_node.stage_id
                || manifest.tool.id != fake_node.tool_id;
            if is_failed {
                FailureClassification {
                    state: ExecutionState::Failed,
                    detail: "injected_stage_failure".to_string(),
                    exit_code: manifest.runtime.exit_code,
                    outputs_present,
                }
            } else {
                FailureClassification {
                    state: ExecutionState::Completed,
                    detail: "completed".to_string(),
                    exit_code: manifest.runtime.exit_code,
                    outputs_present,
                }
            }
        }
        Err(_) => FailureClassification {
            state: ExecutionState::Failed,
            detail: "invalid_failed_stage_result".to_string(),
            exit_code: SEEDED_FAILURE_EXIT_CODE,
            outputs_present: false,
        },
    }
}

fn build_report(
    repo_root: &Path,
    output_path: &Path,
    simulation_root: &Path,
    rows: Vec<EssentialPipelineFailureIsolationRow>,
) -> Result<EssentialPipelineFailureIsolationReport> {
    let completed_node_count =
        rows.iter().filter(|row| row.execution_state == ExecutionState::Completed.as_str()).count();
    let failed_node_count =
        rows.iter().filter(|row| row.execution_state == ExecutionState::Failed.as_str()).count();
    let blocked_node_count =
        rows.iter().filter(|row| row.execution_state == ExecutionState::Blocked.as_str()).count();
    let blocked_descendant_node_ids = rows
        .iter()
        .filter(|row| row.execution_state == ExecutionState::Blocked.as_str())
        .map(|row| format!("{}::{}", row.pipeline_id, row.node_id))
        .collect::<Vec<_>>();
    let continued_unrelated_node_ids = rows
        .iter()
        .filter(|row| row.unrelated_branch_continues)
        .map(|row| format!("{}::{}", row.pipeline_id, row.node_id))
        .collect::<Vec<_>>();

    let failed_row = rows
        .iter()
        .find(|row| {
            row.pipeline_id == SEEDED_FAILED_PIPELINE_ID && row.node_id == SEEDED_FAILED_NODE_ID
        })
        .ok_or_else(|| {
            anyhow!(
                "essential pipeline failure-isolation simulation is missing the seeded failed row"
            )
        })?;
    let blocked_row = rows
        .iter()
        .find(|row| row.pipeline_id == SEEDED_FAILED_PIPELINE_ID && row.node_id == SEEDED_BLOCKED_NODE_ID)
        .ok_or_else(|| anyhow!("essential pipeline failure-isolation simulation is missing the blocked descendant row"))?;
    let continued_row = rows
        .iter()
        .find(|row| row.pipeline_id == SEEDED_FAILED_PIPELINE_ID && row.node_id == SEEDED_CONTINUED_NODE_ID)
        .ok_or_else(|| anyhow!("essential pipeline failure-isolation simulation is missing the continued branch row"))?;

    let passes_behavior_test = failed_row.execution_state == ExecutionState::Failed.as_str()
        && failed_row.exit_code == SEEDED_FAILURE_EXIT_CODE
        && blocked_row.execution_state == ExecutionState::Blocked.as_str()
        && blocked_row.reason == "failed_dependency_blocked"
        && continued_row.execution_state == ExecutionState::Completed.as_str()
        && continued_row.unrelated_branch_continues
        && failed_node_count == 1
        && blocked_node_count == 1;
    if !passes_behavior_test {
        return Err(anyhow!(
            "essential pipeline failure-isolation simulation did not block only dependent descendants while preserving unrelated work"
        ));
    }

    Ok(EssentialPipelineFailureIsolationReport {
        schema_version: ESSENTIAL_PIPELINE_FAILURE_ISOLATION_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        simulation_root: path_relative_to_repo(repo_root, simulation_root),
        pipeline_count: ESSENTIAL_PIPELINE_IDS.len(),
        node_count: rows.len(),
        completed_node_count,
        failed_node_count,
        blocked_node_count,
        seeded_failed_node_id: format!("{SEEDED_FAILED_PIPELINE_ID}::{SEEDED_FAILED_NODE_ID}"),
        blocked_descendant_node_ids,
        continued_unrelated_node_ids,
        passes_behavior_test,
        rows,
    })
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}
