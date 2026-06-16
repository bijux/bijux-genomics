use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use super::essential_pipeline_corpus_assets::{
    render_essential_pipeline_corpus_assets, EssentialPipelineCorpusAssetsRow,
    DEFAULT_ESSENTIAL_PIPELINE_CORPUS_ASSETS_PATH, ESSENTIAL_PIPELINE_IDS,
};
use super::essential_pipeline_rendered_commands::{
    render_essential_pipeline_commands, EssentialPipelineRenderedCommandRow,
    DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH,
};
use super::essential_pipeline_report_map::{
    render_essential_pipeline_report_map, EssentialPipelineReportMapRow,
    DEFAULT_ESSENTIAL_PIPELINE_REPORT_MAP_PATH,
};
use crate::commands::benchmark::local_pipeline_dag::{
    benchmark_local_pipeline_config_path, validate_pipeline_dag_path,
    LocalPipelineDagValidationNodeReport,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ESSENTIAL_PIPELINES_LOCAL_COMPLETE_PATH: &str =
    "benchmarks/readiness/pipelines/ESSENTIAL_PIPELINES_LOCAL_COMPLETE.json";
const ESSENTIAL_PIPELINES_LOCAL_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.essential_pipelines_local_complete.v1";
const EXPECTED_PIPELINE_COUNT: usize = 10;
const EXPECTED_NODE_COUNT: usize = 93;
const EXPECTED_OUTPUT_COUNT: usize = 267;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelinesLocalCompleteNodeRow {
    pub(crate) pipeline_id: String,
    pub(crate) node_id: String,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) input_paths: String,
    pub(crate) output_paths: String,
    pub(crate) tool_id: String,
    pub(crate) render_status: String,
    pub(crate) command_source: String,
    pub(crate) declared_output_count: usize,
    pub(crate) report_map_output_count: usize,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelinesLocalCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) pipeline_count: usize,
    pub(crate) node_count: usize,
    pub(crate) completed_node_count: usize,
    pub(crate) failing_node_count: usize,
    pub(crate) rendered_node_count: usize,
    pub(crate) structured_skip_node_count: usize,
    pub(crate) corpus_asset_row_count: usize,
    pub(crate) rendered_command_row_count: usize,
    pub(crate) report_map_row_count: usize,
    pub(crate) declared_output_count: usize,
    pub(crate) reported_output_count: usize,
    pub(crate) failing_nodes: Vec<String>,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<EssentialPipelinesLocalCompleteNodeRow>,
}

pub(crate) fn run_render_essential_pipelines_local_complete(
    args: &parse::BenchReadinessRenderEssentialPipelinesLocalCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_essential_pipelines_local_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ESSENTIAL_PIPELINES_LOCAL_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_essential_pipelines_local_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<EssentialPipelinesLocalCompleteReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let corpus_assets = render_essential_pipeline_corpus_assets(
        repo_root,
        PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_CORPUS_ASSETS_PATH),
    )?;
    let rendered_commands = render_essential_pipeline_commands(
        repo_root,
        PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH),
    )?;
    let report_map = render_essential_pipeline_report_map(
        repo_root,
        PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_REPORT_MAP_PATH),
    )?;

    let corpus_asset_row_count = corpus_assets.row_count;
    let rendered_command_row_count = rendered_commands.row_count;
    let report_map_row_count = report_map.row_count;
    let rendered_node_count = rendered_commands.rendered_row_count;
    let structured_skip_node_count = rendered_commands.structured_skip_row_count;
    let reported_output_count = report_map.rows.len();

    let corpus_by_node = corpus_assets
        .rows
        .into_iter()
        .map(|row| ((row.pipeline_id.clone(), row.node_id.clone()), row))
        .collect::<BTreeMap<_, _>>();
    let commands_by_node = rendered_commands
        .rows
        .into_iter()
        .map(|row| ((row.pipeline_id.clone(), row.node_id.clone()), row))
        .collect::<BTreeMap<_, _>>();
    let report_map_by_binding = collect_report_map_outputs(report_map.rows);

    let mut rows = Vec::new();
    let mut node_count = 0usize;
    let mut declared_output_count = 0usize;

    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        let config_path = benchmark_local_pipeline_config_path(repo_root, pipeline_id);
        let report_path = repo_root
            .join("benchmarks/readiness/local-ready/pipeline-dag")
            .join(format!("{pipeline_id}.json"));
        let dag = validate_pipeline_dag_path(repo_root, &config_path, &report_path)?;
        let nodes_by_id =
            dag.nodes.iter().map(|node| (node.node_id.clone(), node)).collect::<BTreeMap<_, _>>();

        for node_id in &dag.topological_order {
            let node = nodes_by_id.get(node_id).ok_or_else(|| {
                anyhow!(
                    "essential pipeline local-complete gate is missing node `{node_id}` in pipeline `{pipeline_id}`"
                )
            })?;
            node_count += 1;
            declared_output_count += node.outputs.len();
            rows.push(build_node_row(
                pipeline_id,
                node,
                &corpus_by_node,
                &commands_by_node,
                &report_map_by_binding,
            )?);
        }
    }

    rows.sort_by(|left, right| {
        left.pipeline_id
            .cmp(&right.pipeline_id)
            .then_with(|| left.node_id.cmp(&right.node_id))
            .then_with(|| left.stage_id.cmp(&right.stage_id))
    });

    let completed_node_count = rows.iter().filter(|row| row.ok).count();
    let failing_nodes = rows
        .iter()
        .filter(|row| !row.ok)
        .map(|row| format!("{}/{}", row.pipeline_id, row.node_id))
        .collect::<Vec<_>>();
    let failing_node_count = failing_nodes.len();
    let report = EssentialPipelinesLocalCompleteReport {
        schema_version: ESSENTIAL_PIPELINES_LOCAL_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        pipeline_count: ESSENTIAL_PIPELINE_IDS.len(),
        node_count,
        completed_node_count,
        failing_node_count,
        rendered_node_count,
        structured_skip_node_count,
        corpus_asset_row_count,
        rendered_command_row_count,
        report_map_row_count,
        declared_output_count,
        reported_output_count,
        failing_nodes,
        ok: failing_node_count == 0
            && ESSENTIAL_PIPELINE_IDS.len() == EXPECTED_PIPELINE_COUNT
            && node_count == EXPECTED_NODE_COUNT
            && completed_node_count == EXPECTED_NODE_COUNT
            && corpus_asset_row_count == EXPECTED_NODE_COUNT
            && rendered_command_row_count == EXPECTED_NODE_COUNT
            && report_map_row_count == EXPECTED_OUTPUT_COUNT
            && declared_output_count == EXPECTED_OUTPUT_COUNT
            && reported_output_count == EXPECTED_OUTPUT_COUNT,
        rows,
    };

    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;

    if !report.ok {
        bail!("every essential pipeline node must keep exact local completion coverage");
    }

    Ok(report)
}

fn build_node_row(
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    corpus_by_node: &BTreeMap<(String, String), EssentialPipelineCorpusAssetsRow>,
    commands_by_node: &BTreeMap<(String, String), EssentialPipelineRenderedCommandRow>,
    report_map_by_binding: &BTreeMap<(String, String, String), BTreeSet<String>>,
) -> Result<EssentialPipelinesLocalCompleteNodeRow> {
    let key = (pipeline_id.to_string(), node.node_id.clone());
    let corpus = corpus_by_node.get(&key).ok_or_else(|| {
        anyhow!(
            "essential pipeline local-complete gate is missing corpus/assets coverage for `{pipeline_id}` / `{}`",
            node.node_id
        )
    })?;
    let command = commands_by_node.get(&key).ok_or_else(|| {
        anyhow!(
            "essential pipeline local-complete gate is missing rendered command coverage for `{pipeline_id}` / `{}`",
            node.node_id
        )
    })?;

    if corpus.stage_id != node.stage_id {
        bail!(
            "corpus/assets stage drifted for `{pipeline_id}` / `{}`: expected `{}`, found `{}`",
            node.node_id,
            node.stage_id,
            corpus.stage_id
        );
    }
    if command.stage_id != node.stage_id {
        bail!(
            "rendered-command stage drifted for `{pipeline_id}` / `{}`: expected `{}`, found `{}`",
            node.node_id,
            node.stage_id,
            command.stage_id
        );
    }

    let expected_input_paths = render_declared_input_paths(node);
    let expected_output_paths = render_declared_output_paths(node);
    if corpus.input_paths != expected_input_paths {
        bail!(
            "input-path contract drifted for `{pipeline_id}` / `{}`: expected `{expected_input_paths}`, found `{}`",
            node.node_id,
            corpus.input_paths
        );
    }
    if corpus.output_paths != expected_output_paths {
        bail!(
            "output-path contract drifted for `{pipeline_id}` / `{}`: expected `{expected_output_paths}`, found `{}`",
            node.node_id,
            corpus.output_paths
        );
    }
    if corpus.corpus_id.trim().is_empty() || corpus.asset_profile_id.trim().is_empty() {
        bail!(
            "corpus/assets coverage kept an empty governed binding for `{pipeline_id}` / `{}`",
            node.node_id
        );
    }

    match command.render_status.as_str() {
        "rendered" => {
            if command.command_steps.is_empty() || command.script_commands.is_empty() {
                bail!(
                    "rendered command coverage is incomplete for `{pipeline_id}` / `{}`",
                    node.node_id
                );
            }
        }
        "structured_skip" => {
            let Some(skip) = &command.skip else {
                bail!(
                    "structured skip coverage is missing skip metadata for `{pipeline_id}` / `{}`",
                    node.node_id
                );
            };
            if skip.condition_id.trim().is_empty() || skip.detail.trim().is_empty() {
                bail!(
                    "structured skip coverage is incomplete for `{pipeline_id}` / `{}`",
                    node.node_id
                );
            }
        }
        other => {
            bail!(
                "essential pipeline local-complete gate found unsupported render status `{other}` for `{pipeline_id}` / `{}`",
                node.node_id
            );
        }
    }

    let expected_outputs = node.outputs.iter().cloned().collect::<BTreeSet<_>>();
    let actual_outputs = report_map_by_binding
        .get(&(pipeline_id.to_string(), node.stage_id.clone(), command.tool_id.clone()))
        .cloned()
        .ok_or_else(|| {
            anyhow!(
                "essential pipeline local-complete gate is missing report-map rows for `{pipeline_id}` / `{}` / `{}` / `{}`",
                node.node_id,
                node.stage_id,
                command.tool_id
            )
        })?;
    if actual_outputs != expected_outputs {
        bail!(
            "report-map output drifted for `{pipeline_id}` / `{}`: missing={:?} extra={:?}",
            node.node_id,
            diff_outputs(&expected_outputs, &actual_outputs),
            diff_outputs(&actual_outputs, &expected_outputs)
        );
    }

    Ok(EssentialPipelinesLocalCompleteNodeRow {
        pipeline_id: pipeline_id.to_string(),
        node_id: node.node_id.clone(),
        stage_id: node.stage_id.clone(),
        corpus_id: corpus.corpus_id.clone(),
        asset_profile_id: corpus.asset_profile_id.clone(),
        input_paths: corpus.input_paths.clone(),
        output_paths: corpus.output_paths.clone(),
        tool_id: command.tool_id.clone(),
        render_status: command.render_status.clone(),
        command_source: command.command_source.clone(),
        declared_output_count: expected_outputs.len(),
        report_map_output_count: actual_outputs.len(),
        ok: true,
        detail: format!(
            "validated exact corpus/assets, declared input/output paths, `{}` command coverage, and {} report-map output(s)",
            command.command_source,
            actual_outputs.len()
        ),
    })
}

fn collect_report_map_outputs(
    rows: Vec<EssentialPipelineReportMapRow>,
) -> BTreeMap<(String, String, String), BTreeSet<String>> {
    let mut outputs = BTreeMap::<(String, String, String), BTreeSet<String>>::new();
    for row in rows {
        outputs
            .entry((row.pipeline_id, row.stage_id, row.tool_id))
            .or_default()
            .insert(row.output_metric);
    }
    outputs
}

fn render_declared_input_paths(node: &LocalPipelineDagValidationNodeReport) -> String {
    node.external_inputs
        .iter()
        .map(|value| format!("external:{value}"))
        .chain(node.upstream_inputs.iter().map(|value| format!("upstream:{value}")))
        .collect::<Vec<_>>()
        .join(",")
}

fn render_declared_output_paths(node: &LocalPipelineDagValidationNodeReport) -> String {
    node.outputs.iter().map(|value| format!("output:{value}")).collect::<Vec<_>>().join(",")
}

fn diff_outputs(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.difference(right).cloned().collect::<Vec<_>>()
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_essential_pipelines_local_complete, DEFAULT_ESSENTIAL_PIPELINES_LOCAL_COMPLETE_PATH,
        ESSENTIAL_PIPELINES_LOCAL_COMPLETE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_essential_pipelines_local_complete_reports_governed_pass_state() {
        let report = render_essential_pipelines_local_complete(
            &repo_root(),
            PathBuf::from(DEFAULT_ESSENTIAL_PIPELINES_LOCAL_COMPLETE_PATH),
        )
        .expect("render essential pipeline local-complete report");

        assert_eq!(report.schema_version, ESSENTIAL_PIPELINES_LOCAL_COMPLETE_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_ESSENTIAL_PIPELINES_LOCAL_COMPLETE_PATH);
        assert_eq!(report.pipeline_count, 10);
        assert_eq!(report.node_count, 93);
        assert_eq!(report.completed_node_count, 93);
        assert_eq!(report.failing_node_count, 0);
        assert_eq!(report.rendered_node_count, 93);
        assert_eq!(report.structured_skip_node_count, 0);
        assert_eq!(report.corpus_asset_row_count, 93);
        assert_eq!(report.rendered_command_row_count, 93);
        assert_eq!(report.report_map_row_count, 267);
        assert_eq!(report.declared_output_count, 267);
        assert_eq!(report.reported_output_count, 267);
        assert!(report.failing_nodes.is_empty());
        assert!(report.ok);
        assert_eq!(report.rows.len(), 93);
        assert!(report.rows.iter().all(|row| row.ok));
        assert!(report.rows.iter().any(|row| {
            row.pipeline_id == "bam-genotyping-to-vcf-downstream"
                && row.node_id == "bam.genotyping"
                && row.command_source == "local_stage_materialization"
        }));
    }
}
