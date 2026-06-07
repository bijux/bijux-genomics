use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_essential_pipeline_fake_runs::{
    fake_run_essential_pipelines, EssentialPipelineFakeRunNodeReport,
    EssentialPipelineFakeRunsReport,
};
use crate::commands::benchmark::local_pipeline_dag::{
    validate_pipeline_dag_path, LocalPipelineDagValidationNodeReport,
};
use crate::commands::benchmark::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, path_relative_to_repo,
};
use crate::commands::benchmark::readiness::essential_pipeline_corpus_assets::ESSENTIAL_PIPELINE_IDS;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ESSENTIAL_PIPELINE_PARTIAL_RESUME_REPORT_PATH: &str =
    "benchmarks/readiness/essential-pipeline-partial-resume.json";
const DEFAULT_ESSENTIAL_PIPELINE_PARTIAL_RESUME_SIMULATION_ROOT: &str =
    "benchmarks/readiness/essential-pipeline-partial-resume-tree";
const ESSENTIAL_PIPELINE_PARTIAL_RESUME_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.essential_pipeline_partial_resume.v1";
const SEEDED_INVALID_PIPELINE_ID: &str = "relatedness-segments-vcf";
const SEEDED_INVALID_NODE_ID: &str = "vcf.ibd";
const SEEDED_UNRELATED_CONTINUED_NODE_ID: &str = "vcf.roh";
const SEEDED_DOWNSTREAM_RERUN_NODE_ID: &str = "vcf.demography";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompletionState {
    ValidCompleted,
    InvalidStageResultManifest,
    MissingStageResultManifest,
}

impl CompletionState {
    fn as_str(self) -> &'static str {
        match self {
            Self::ValidCompleted => "valid_completed",
            Self::InvalidStageResultManifest => "invalid_stage_result_manifest",
            Self::MissingStageResultManifest => "missing_stage_result_manifest",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResumeAction {
    Skip,
    Rerun,
}

impl ResumeAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Skip => "skip",
            Self::Rerun => "rerun",
        }
    }
}

#[derive(Debug, Clone)]
struct CompletionClassification {
    state: CompletionState,
    outputs_present: bool,
    manifest_output_count: usize,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EssentialPipelinePartialResumeRow {
    pub(crate) pipeline_id: String,
    pub(crate) node_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) completion_state: String,
    pub(crate) resume_action: String,
    pub(crate) reason: String,
    pub(crate) dependency_count: usize,
    pub(crate) depends_on: Vec<String>,
    pub(crate) stage_result_path: String,
    pub(crate) outputs_present: bool,
    pub(crate) manifest_output_count: usize,
    pub(crate) unrelated_branch_continues: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EssentialPipelinePartialResumeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) simulation_root: String,
    pub(crate) pipeline_count: usize,
    pub(crate) node_count: usize,
    pub(crate) valid_completed_node_count: usize,
    pub(crate) invalid_manifest_node_count: usize,
    pub(crate) missing_manifest_node_count: usize,
    pub(crate) skip_node_count: usize,
    pub(crate) rerun_node_count: usize,
    pub(crate) seeded_invalid_node_id: String,
    pub(crate) downstream_rerun_node_ids: Vec<String>,
    pub(crate) continued_unrelated_node_ids: Vec<String>,
    pub(crate) passes_behavior_test: bool,
    pub(crate) rows: Vec<EssentialPipelinePartialResumeRow>,
}

pub(crate) fn run_render_essential_pipeline_partial_resume(
    args: &parse::BenchReadinessRenderEssentialPipelinePartialResumeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_essential_pipeline_partial_resume(
        &repo_root,
        args.output.clone().unwrap_or_else(|| {
            PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_PARTIAL_RESUME_REPORT_PATH)
        }),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_essential_pipeline_partial_resume(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<EssentialPipelinePartialResumeReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let simulation_root = repo_relative_path(
        repo_root,
        &PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_PARTIAL_RESUME_SIMULATION_ROOT),
    );
    let fake_run_report = fake_run_essential_pipelines(repo_root, simulation_root.clone())?;
    inject_invalid_stage_result_manifest(repo_root, &fake_run_report)?;
    let rows = collect_partial_resume_rows(repo_root, &fake_run_report)?;
    let report = build_report(repo_root, &output_path, &simulation_root, rows)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

fn inject_invalid_stage_result_manifest(
    repo_root: &Path,
    fake_run_report: &EssentialPipelineFakeRunsReport,
) -> Result<()> {
    let seeded_node = fake_run_report
        .pipelines
        .iter()
        .find(|pipeline| pipeline.pipeline_id == SEEDED_INVALID_PIPELINE_ID)
        .and_then(|pipeline| pipeline.nodes.iter().find(|node| node.node_id == SEEDED_INVALID_NODE_ID))
        .ok_or_else(|| {
            anyhow!(
                "essential pipeline partial-resume simulation cannot find seeded invalid node `{SEEDED_INVALID_PIPELINE_ID}` / `{SEEDED_INVALID_NODE_ID}`"
            )
        })?;
    let stage_result_path = repo_root.join(&seeded_node.stage_result_path);
    let mut payload: serde_json::Value = serde_json::from_slice(
        &fs::read(&stage_result_path)
            .with_context(|| format!("read {}", stage_result_path.display()))?,
    )
    .with_context(|| format!("parse {}", stage_result_path.display()))?;
    payload["command"]["rendered"] = serde_json::Value::String(String::new());
    bijux_dna_infra::atomic_write_json(&stage_result_path, &payload)
        .with_context(|| format!("write {}", stage_result_path.display()))?;
    Ok(())
}

fn collect_partial_resume_rows(
    repo_root: &Path,
    fake_run_report: &EssentialPipelineFakeRunsReport,
) -> Result<Vec<EssentialPipelinePartialResumeRow>> {
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

        let mut classifications = BTreeMap::<String, CompletionClassification>::new();
        for node_id in &dag_report.topological_order {
            let fake_node = fake_nodes
                .get(&(pipeline_id.to_string(), node_id.clone()))
                .ok_or_else(|| {
                    anyhow!(
                        "essential pipeline partial-resume simulation is missing fake-run node `{pipeline_id}` / `{node_id}`"
                    )
                })?;
            let dag_node = nodes_by_id.get(node_id).ok_or_else(|| {
                anyhow!(
                    "essential pipeline partial-resume simulation is missing DAG node `{pipeline_id}` / `{node_id}`"
                )
            })?;
            classifications
                .insert(node_id.clone(), classify_completion_state(repo_root, dag_node, fake_node));
        }

        let mut rerun_nodes = BTreeSet::<String>::new();
        for node_id in &dag_report.topological_order {
            let dag_node = nodes_by_id.get(node_id).expect("validated node");
            let classification = classifications.get(node_id).expect("classified node");
            let rerun_reason = if classification.state != CompletionState::ValidCompleted {
                Some(classification.detail.clone())
            } else if dag_node.depends_on.iter().any(|dependency| rerun_nodes.contains(dependency))
            {
                Some("upstream_dependency_rerun".to_string())
            } else {
                None
            };
            let action = if rerun_reason.is_some() {
                rerun_nodes.insert(node_id.clone());
                ResumeAction::Rerun
            } else {
                ResumeAction::Skip
            };
            let fake_node =
                fake_nodes.get(&(pipeline_id.to_string(), node_id.clone())).expect("fake node");
            rows.push(EssentialPipelinePartialResumeRow {
                pipeline_id: (*pipeline_id).to_string(),
                node_id: node_id.clone(),
                stage_id: dag_node.stage_id.clone(),
                tool_id: fake_node.tool_id.clone(),
                completion_state: classification.state.as_str().to_string(),
                resume_action: action.as_str().to_string(),
                reason: rerun_reason.unwrap_or_else(|| "valid_completed".to_string()),
                dependency_count: dag_node.depends_on.len(),
                depends_on: dag_node.depends_on.clone(),
                stage_result_path: fake_node.stage_result_path.clone(),
                outputs_present: classification.outputs_present,
                manifest_output_count: classification.manifest_output_count,
                unrelated_branch_continues: *pipeline_id == SEEDED_INVALID_PIPELINE_ID
                    && node_id == SEEDED_UNRELATED_CONTINUED_NODE_ID
                    && action == ResumeAction::Skip,
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

fn classify_completion_state(
    repo_root: &Path,
    dag_node: &LocalPipelineDagValidationNodeReport,
    fake_node: &EssentialPipelineFakeRunNodeReport,
) -> CompletionClassification {
    let stage_result_path = repo_root.join(&fake_node.stage_result_path);
    if !stage_result_path.is_file() {
        return CompletionClassification {
            state: CompletionState::MissingStageResultManifest,
            outputs_present: false,
            manifest_output_count: 0,
            detail: "missing_stage_result_manifest".to_string(),
        };
    }

    match load_validated_stage_result_manifest_path(&stage_result_path) {
        Ok(manifest) => {
            if manifest.stage_id != dag_node.stage_id {
                return CompletionClassification {
                    state: CompletionState::InvalidStageResultManifest,
                    outputs_present: false,
                    manifest_output_count: manifest.outputs.len(),
                    detail: "invalid_stage_result_manifest".to_string(),
                };
            }
            if manifest.tool.id != fake_node.tool_id {
                return CompletionClassification {
                    state: CompletionState::InvalidStageResultManifest,
                    outputs_present: false,
                    manifest_output_count: manifest.outputs.len(),
                    detail: "invalid_stage_result_manifest".to_string(),
                };
            }
            let all_outputs_present = manifest
                .outputs
                .iter()
                .all(|output| output.exists && repo_root.join(&output.realized_path).exists());
            if !all_outputs_present {
                return CompletionClassification {
                    state: CompletionState::InvalidStageResultManifest,
                    outputs_present: false,
                    manifest_output_count: manifest.outputs.len(),
                    detail: "invalid_stage_result_manifest".to_string(),
                };
            }
            CompletionClassification {
                state: CompletionState::ValidCompleted,
                outputs_present: true,
                manifest_output_count: manifest.outputs.len(),
                detail: "valid_completed".to_string(),
            }
        }
        Err(_) => CompletionClassification {
            state: CompletionState::InvalidStageResultManifest,
            outputs_present: false,
            manifest_output_count: 0,
            detail: "invalid_stage_result_manifest".to_string(),
        },
    }
}

fn build_report(
    repo_root: &Path,
    output_path: &Path,
    simulation_root: &Path,
    rows: Vec<EssentialPipelinePartialResumeRow>,
) -> Result<EssentialPipelinePartialResumeReport> {
    let valid_completed_node_count = rows
        .iter()
        .filter(|row| row.completion_state == CompletionState::ValidCompleted.as_str())
        .count();
    let invalid_manifest_node_count = rows
        .iter()
        .filter(|row| row.completion_state == CompletionState::InvalidStageResultManifest.as_str())
        .count();
    let missing_manifest_node_count = rows
        .iter()
        .filter(|row| row.completion_state == CompletionState::MissingStageResultManifest.as_str())
        .count();
    let skip_node_count =
        rows.iter().filter(|row| row.resume_action == ResumeAction::Skip.as_str()).count();
    let rerun_node_count =
        rows.iter().filter(|row| row.resume_action == ResumeAction::Rerun.as_str()).count();
    let downstream_rerun_node_ids = rows
        .iter()
        .filter(|row| row.reason == "upstream_dependency_rerun")
        .map(|row| format!("{}::{}", row.pipeline_id, row.node_id))
        .collect::<Vec<_>>();
    let continued_unrelated_node_ids = rows
        .iter()
        .filter(|row| row.unrelated_branch_continues)
        .map(|row| format!("{}::{}", row.pipeline_id, row.node_id))
        .collect::<Vec<_>>();

    let seeded_invalid_row = rows
        .iter()
        .find(|row| {
            row.pipeline_id == SEEDED_INVALID_PIPELINE_ID && row.node_id == SEEDED_INVALID_NODE_ID
        })
        .ok_or_else(|| {
            anyhow!(
                "essential pipeline partial-resume simulation is missing the seeded invalid row"
            )
        })?;
    let downstream_row = rows
        .iter()
        .find(|row| {
            row.pipeline_id == SEEDED_INVALID_PIPELINE_ID
                && row.node_id == SEEDED_DOWNSTREAM_RERUN_NODE_ID
        })
        .ok_or_else(|| {
            anyhow!(
                "essential pipeline partial-resume simulation is missing the downstream rerun row"
            )
        })?;
    let unrelated_row = rows
        .iter()
        .find(|row| {
            row.pipeline_id == SEEDED_INVALID_PIPELINE_ID
                && row.node_id == SEEDED_UNRELATED_CONTINUED_NODE_ID
        })
        .ok_or_else(|| {
            anyhow!(
                "essential pipeline partial-resume simulation is missing the unrelated branch row"
            )
        })?;

    let passes_behavior_test = seeded_invalid_row.completion_state
        == CompletionState::InvalidStageResultManifest.as_str()
        && seeded_invalid_row.resume_action == ResumeAction::Rerun.as_str()
        && downstream_row.resume_action == ResumeAction::Rerun.as_str()
        && downstream_row.reason == "upstream_dependency_rerun"
        && unrelated_row.resume_action == ResumeAction::Skip.as_str()
        && unrelated_row.unrelated_branch_continues
        && skip_node_count > rerun_node_count;
    if !passes_behavior_test {
        return Err(anyhow!(
            "essential pipeline partial-resume simulation did not preserve valid nodes, rerun the invalid branch, and continue an unrelated branch"
        ));
    }

    Ok(EssentialPipelinePartialResumeReport {
        schema_version: ESSENTIAL_PIPELINE_PARTIAL_RESUME_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        simulation_root: path_relative_to_repo(repo_root, simulation_root),
        pipeline_count: ESSENTIAL_PIPELINE_IDS.len(),
        node_count: rows.len(),
        valid_completed_node_count,
        invalid_manifest_node_count,
        missing_manifest_node_count,
        skip_node_count,
        rerun_node_count,
        seeded_invalid_node_id: format!("{SEEDED_INVALID_PIPELINE_ID}::{SEEDED_INVALID_NODE_ID}"),
        downstream_rerun_node_ids,
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
