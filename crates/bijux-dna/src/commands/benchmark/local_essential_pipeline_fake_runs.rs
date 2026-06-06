use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_pipeline_dag::{validate_pipeline_dag_path, LocalPipelineDagValidationNodeReport};
use super::local_stage_result_manifest::{
    validate_stage_result_manifest, BenchStageResultCommandV1, BenchStageResultManifestV1,
    BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::readiness::essential_pipeline_corpus_assets::{
    collect_essential_pipeline_corpus_asset_rows, EssentialPipelineCorpusAssetsRow,
    ESSENTIAL_PIPELINE_IDS,
};
use super::readiness::essential_pipeline_rendered_commands::{
    collect_essential_pipeline_rendered_command_rows, EssentialPipelineRenderedCommandRow,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ESSENTIAL_PIPELINE_FAKE_RUN_ROOT: &str =
    "target/local-fake-runs/pipelines/essential";
const ESSENTIAL_PIPELINE_FAKE_RUNS_SCHEMA_VERSION: &str =
    "bijux.bench.local_essential_pipeline_fake_runs.v1";
const ESSENTIAL_PIPELINE_FAKE_RUN_PIPELINE_SCHEMA_VERSION: &str =
    "bijux.bench.local_essential_pipeline_fake_run_pipeline.v1";
const ESSENTIAL_PIPELINE_FAKE_RUN_NODE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_essential_pipeline_fake_run_node_metrics.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelineFakeRunOutputEntry {
    pub(crate) artifact_id: String,
    pub(crate) declared_output: String,
    pub(crate) fake_run_path: String,
    pub(crate) role: String,
    pub(crate) exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelineFakeRunInputBinding {
    pub(crate) input_id: String,
    pub(crate) source: String,
    pub(crate) producer_node_id: Option<String>,
    pub(crate) fake_run_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EssentialPipelineFakeRunNodeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) pipeline_id: String,
    pub(crate) node_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) dependency_count: usize,
    pub(crate) external_input_count: usize,
    pub(crate) upstream_input_count: usize,
    pub(crate) command_step_count: usize,
    pub(crate) output_count: usize,
    pub(crate) materialized_byte_count: u64,
    pub(crate) simulated_elapsed_seconds: f64,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) upstream_bindings: Vec<EssentialPipelineFakeRunInputBinding>,
    pub(crate) external_bindings: Vec<EssentialPipelineFakeRunInputBinding>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EssentialPipelineFakeRunNodeReport {
    pub(crate) pipeline_id: String,
    pub(crate) node_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) domain: String,
    pub(crate) readiness_kind: String,
    pub(crate) command_source: String,
    pub(crate) dependency_count: usize,
    pub(crate) command_step_count: usize,
    pub(crate) declared_output_count: usize,
    pub(crate) created_output_count: usize,
    pub(crate) command_script_path: String,
    pub(crate) stdout_path: String,
    pub(crate) stderr_path: String,
    pub(crate) metrics_path: String,
    pub(crate) stage_result_path: String,
    pub(crate) outputs: Vec<EssentialPipelineFakeRunOutputEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EssentialPipelineFakeRunPipelineReport {
    pub(crate) schema_version: &'static str,
    pub(crate) pipeline_id: String,
    pub(crate) pipeline_manifest_path: String,
    pub(crate) node_count: usize,
    pub(crate) created_output_count: usize,
    pub(crate) topological_order: Vec<String>,
    pub(crate) nodes: Vec<EssentialPipelineFakeRunNodeReport>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EssentialPipelineFakeRunsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fake_run_root: String,
    pub(crate) root_manifest_path: String,
    pub(crate) pipeline_count: usize,
    pub(crate) node_count: usize,
    pub(crate) created_output_count: usize,
    pub(crate) pipelines: Vec<EssentialPipelineFakeRunPipelineReport>,
}

#[derive(Clone)]
struct ProducedOutput {
    producer_node_id: String,
    fake_run_path: String,
}

struct NodeFakeRunArtifacts {
    command_script_path: PathBuf,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    metrics_path: PathBuf,
    stage_result_path: PathBuf,
}

struct NodeFakeRunContext<'a> {
    pipeline_id: &'a str,
    node: &'a LocalPipelineDagValidationNodeReport,
    command_row: &'a EssentialPipelineRenderedCommandRow,
    corpus_assets: &'a EssentialPipelineCorpusAssetsRow,
    node_root: &'a Path,
}

pub(crate) fn run_fake_run_essential_pipelines(
    args: &parse::BenchLocalFakeRunEssentialPipelinesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let manifest = fake_run_essential_pipelines(
        &repo_root,
        args.output_root
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_FAKE_RUN_ROOT)),
    )?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.fake_run_root);
    }
    Ok(())
}

pub(crate) fn fake_run_essential_pipelines(
    repo_root: &Path,
    output_root: PathBuf,
) -> Result<EssentialPipelineFakeRunsReport> {
    let absolute_output_root = repo_relative_path(repo_root, &output_root);
    fs::create_dir_all(&absolute_output_root)
        .with_context(|| format!("create {}", absolute_output_root.display()))?;

    let command_rows = collect_essential_pipeline_rendered_command_rows(repo_root)?;
    ensure_all_nodes_rendered(&command_rows)?;
    let corpus_asset_rows = collect_essential_pipeline_corpus_asset_rows(repo_root)?;

    let command_rows_by_node = command_rows
        .into_iter()
        .map(|row| ((row.pipeline_id.clone(), row.node_id.clone()), row))
        .collect::<BTreeMap<_, _>>();
    let corpus_asset_rows_by_node = corpus_asset_rows
        .into_iter()
        .map(|row| ((row.pipeline_id.clone(), row.node_id.clone()), row))
        .collect::<BTreeMap<_, _>>();

    let mut pipeline_reports = Vec::with_capacity(ESSENTIAL_PIPELINE_IDS.len());
    let mut total_nodes = 0usize;
    let mut total_created_outputs = 0usize;

    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        let pipeline_report = fake_run_pipeline(
            repo_root,
            &absolute_output_root,
            pipeline_id,
            &command_rows_by_node,
            &corpus_asset_rows_by_node,
        )?;
        total_nodes += pipeline_report.node_count;
        total_created_outputs += pipeline_report.created_output_count;
        pipeline_reports.push(pipeline_report);
    }

    let root_manifest_path = absolute_output_root.join("manifest.json");
    let manifest = EssentialPipelineFakeRunsReport {
        schema_version: ESSENTIAL_PIPELINE_FAKE_RUNS_SCHEMA_VERSION,
        fake_run_root: path_relative_to_repo(repo_root, &absolute_output_root),
        root_manifest_path: path_relative_to_repo(repo_root, &root_manifest_path),
        pipeline_count: pipeline_reports.len(),
        node_count: total_nodes,
        created_output_count: total_created_outputs,
        pipelines: pipeline_reports,
    };
    ensure_essential_pipeline_fake_run_contract(&manifest)?;
    bijux_dna_infra::atomic_write_json(&root_manifest_path, &manifest)?;
    Ok(manifest)
}

fn fake_run_pipeline(
    repo_root: &Path,
    fake_run_root: &Path,
    pipeline_id: &str,
    command_rows_by_node: &BTreeMap<(String, String), EssentialPipelineRenderedCommandRow>,
    corpus_asset_rows_by_node: &BTreeMap<(String, String), EssentialPipelineCorpusAssetsRow>,
) -> Result<EssentialPipelineFakeRunPipelineReport> {
    let config_path = repo_root.join("configs/pipelines/local").join(format!("{pipeline_id}.toml"));
    let pipeline_report_path =
        repo_root.join("target/local-ready/pipeline-dag").join(format!("{pipeline_id}.json"));
    let dag_report = validate_pipeline_dag_path(repo_root, &config_path, &pipeline_report_path)?;

    let pipeline_root = fake_run_root.join(pipeline_id);
    fs::create_dir_all(&pipeline_root)
        .with_context(|| format!("create {}", pipeline_root.display()))?;

    let nodes_by_id = dag_report
        .nodes
        .iter()
        .map(|node| (node.node_id.clone(), node))
        .collect::<BTreeMap<_, _>>();
    let mut produced_outputs = BTreeMap::<String, ProducedOutput>::new();
    let mut node_reports = Vec::with_capacity(dag_report.nodes.len());
    let mut created_output_count = 0usize;

    for node_id in &dag_report.topological_order {
        let node = nodes_by_id.get(node_id).ok_or_else(|| {
            anyhow!("essential pipeline fake-runner is missing node `{node_id}` in pipeline `{pipeline_id}`")
        })?;
        let key = (pipeline_id.to_string(), node.node_id.clone());
        let command_row = command_rows_by_node.get(&key).ok_or_else(|| {
            anyhow!(
                "essential pipeline fake-runner is missing a rendered command row for `{}` / `{}`",
                pipeline_id,
                node.node_id
            )
        })?;
        let corpus_assets = corpus_asset_rows_by_node.get(&key).ok_or_else(|| {
            anyhow!(
                "essential pipeline fake-runner is missing a corpus/assets row for `{}` / `{}`",
                pipeline_id,
                node.node_id
            )
        })?;
        let node_report = fake_run_node(
            repo_root,
            &pipeline_root,
            &NodeFakeRunContext {
                pipeline_id,
                node,
                command_row,
                corpus_assets,
                node_root: &pipeline_root.join(&node.node_id),
            },
            &mut produced_outputs,
        )?;
        created_output_count += node_report.created_output_count;
        node_reports.push(node_report);
    }

    let pipeline_manifest_path = pipeline_root.join("manifest.json");
    let pipeline_manifest = EssentialPipelineFakeRunPipelineReport {
        schema_version: ESSENTIAL_PIPELINE_FAKE_RUN_PIPELINE_SCHEMA_VERSION,
        pipeline_id: pipeline_id.to_string(),
        pipeline_manifest_path: path_relative_to_repo(repo_root, &pipeline_manifest_path),
        node_count: node_reports.len(),
        created_output_count,
        topological_order: dag_report.topological_order.clone(),
        nodes: node_reports,
    };
    bijux_dna_infra::atomic_write_json(&pipeline_manifest_path, &pipeline_manifest)?;
    Ok(pipeline_manifest)
}

fn fake_run_node(
    repo_root: &Path,
    pipeline_root: &Path,
    context: &NodeFakeRunContext<'_>,
    produced_outputs: &mut BTreeMap<String, ProducedOutput>,
) -> Result<EssentialPipelineFakeRunNodeReport> {
    fs::create_dir_all(context.node_root)
        .with_context(|| format!("create {}", context.node_root.display()))?;
    let artifacts = NodeFakeRunArtifacts {
        command_script_path: context.node_root.join("command.sh"),
        stdout_path: context.node_root.join("stdout.txt"),
        stderr_path: context.node_root.join("stderr.txt"),
        metrics_path: context.node_root.join("metrics.json"),
        stage_result_path: context.node_root.join("stage-result.json"),
    };

    let upstream_bindings =
        resolve_upstream_bindings(context.pipeline_id, context.node, produced_outputs)?;
    let external_bindings = context
        .node
        .external_inputs
        .iter()
        .map(|input_id| EssentialPipelineFakeRunInputBinding {
            input_id: input_id.clone(),
            source: "external".to_string(),
            producer_node_id: None,
            fake_run_path: None,
        })
        .collect::<Vec<_>>();

    fs::write(&artifacts.command_script_path, render_node_command_script(context.command_row))
        .with_context(|| format!("write {}", artifacts.command_script_path.display()))?;
    fs::write(
        &artifacts.stdout_path,
        render_node_stdout(
            context.pipeline_id,
            context.node,
            context.command_row,
            &upstream_bindings,
            &external_bindings,
        ),
    )
    .with_context(|| format!("write {}", artifacts.stdout_path.display()))?;
    fs::write(
        &artifacts.stderr_path,
        format!(
            "fake local essential pipeline run produced no stderr\npipeline_id={}\nnode_id={}\nstage_id={}\n",
            context.pipeline_id, context.node.node_id, context.node.stage_id
        ),
    )
    .with_context(|| format!("write {}", artifacts.stderr_path.display()))?;

    let mut outputs = Vec::with_capacity(context.node.outputs.len());
    let mut materialized_byte_count = 0u64;
    for output_id in &context.node.outputs {
        let output_path = node_output_path(context.node_root, output_id);
        materialize_pipeline_output(&output_path, context.pipeline_id, context.node, output_id)
            .with_context(|| {
                format!(
                    "materialize fake output `{output_id}` for pipeline `{}` node `{}`",
                    context.pipeline_id, context.node.node_id
                )
            })?;
        let exists = output_path.exists();
        if exists {
            materialized_byte_count += materialized_path_size(&output_path)?;
        }
        let fake_run_path = path_relative_to_repo(repo_root, &output_path);
        produced_outputs.insert(
            output_id.clone(),
            ProducedOutput {
                producer_node_id: context.node.node_id.clone(),
                fake_run_path: fake_run_path.clone(),
            },
        );
        outputs.push(EssentialPipelineFakeRunOutputEntry {
            artifact_id: output_id.clone(),
            declared_output: output_id.clone(),
            fake_run_path,
            role: output_role(output_id).to_string(),
            exists,
        });
    }

    let metrics = EssentialPipelineFakeRunNodeMetrics {
        schema_version: ESSENTIAL_PIPELINE_FAKE_RUN_NODE_METRICS_SCHEMA_VERSION,
        pipeline_id: context.pipeline_id.to_string(),
        node_id: context.node.node_id.clone(),
        stage_id: context.node.stage_id.clone(),
        tool_id: context.command_row.tool_id.clone(),
        dependency_count: context.node.depends_on.len(),
        external_input_count: context.node.external_inputs.len(),
        upstream_input_count: context.node.upstream_inputs.len(),
        command_step_count: context.command_row.command_steps.len(),
        output_count: outputs.len(),
        materialized_byte_count,
        simulated_elapsed_seconds: simulated_elapsed_seconds(
            context.command_row.command_steps.len(),
        ),
        corpus_id: context.corpus_assets.corpus_id.clone(),
        asset_profile_id: context.corpus_assets.asset_profile_id.clone(),
        upstream_bindings: upstream_bindings.clone(),
        external_bindings: external_bindings.clone(),
    };
    bijux_dna_infra::atomic_write_json(&artifacts.metrics_path, &metrics)?;

    let stage_result = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: context.node.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: context.command_row.tool_id.clone() },
        command: BenchStageResultCommandV1 {
            rendered: context.command_row.script_commands.join("\n"),
        },
        runtime: BenchStageResultRuntimeV1 {
            mode: "pipeline_fake_run".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: "1970-01-01T00:00:00Z".to_string(),
            finished_at: "1970-01-01T00:00:01Z".to_string(),
            elapsed_seconds: metrics.simulated_elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs: outputs
            .iter()
            .map(|output| BenchStageResultOutputV1 {
                artifact_id: output.artifact_id.clone(),
                declared_path: output.declared_output.clone(),
                realized_path: output.fake_run_path.clone(),
                role: output.role.clone(),
                optional: false,
                exists: output.exists,
            })
            .collect(),
    };
    validate_stage_result_manifest(&stage_result)?;
    bijux_dna_infra::atomic_write_json(&artifacts.stage_result_path, &stage_result)?;

    let report = EssentialPipelineFakeRunNodeReport {
        pipeline_id: context.pipeline_id.to_string(),
        node_id: context.node.node_id.clone(),
        stage_id: context.node.stage_id.clone(),
        tool_id: context.command_row.tool_id.clone(),
        domain: stage_domain(context.node.stage_id.as_str())?.to_string(),
        readiness_kind: context.node.readiness_kind.clone(),
        command_source: context.command_row.command_source.clone(),
        dependency_count: context.node.depends_on.len(),
        command_step_count: context.command_row.command_steps.len(),
        declared_output_count: outputs.len(),
        created_output_count: outputs.iter().filter(|output| output.exists).count(),
        command_script_path: path_relative_to_repo(repo_root, &artifacts.command_script_path),
        stdout_path: path_relative_to_repo(repo_root, &artifacts.stdout_path),
        stderr_path: path_relative_to_repo(repo_root, &artifacts.stderr_path),
        metrics_path: path_relative_to_repo(repo_root, &artifacts.metrics_path),
        stage_result_path: path_relative_to_repo(repo_root, &artifacts.stage_result_path),
        outputs,
    };
    ensure_node_report_contract(pipeline_root, &report)?;
    Ok(report)
}

fn ensure_all_nodes_rendered(rows: &[EssentialPipelineRenderedCommandRow]) -> Result<()> {
    let skipped = rows
        .iter()
        .filter(|row| row.render_status != "rendered")
        .map(|row| format!("{}:{}", row.pipeline_id, row.node_id))
        .collect::<Vec<_>>();
    if !skipped.is_empty() {
        return Err(anyhow!(
            "essential pipeline fake-runner requires executable commands for every governed node; structured skip rows remain for: {}",
            skipped.join(", ")
        ));
    }
    Ok(())
}

fn resolve_upstream_bindings(
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    produced_outputs: &BTreeMap<String, ProducedOutput>,
) -> Result<Vec<EssentialPipelineFakeRunInputBinding>> {
    let mut bindings = Vec::with_capacity(node.upstream_inputs.len());
    for input_id in &node.upstream_inputs {
        let produced = produced_outputs.get(input_id).ok_or_else(|| {
            anyhow!(
                "essential pipeline fake-runner cannot resolve upstream input `{input_id}` for `{}` / `{}`",
                pipeline_id,
                node.node_id
            )
        })?;
        bindings.push(EssentialPipelineFakeRunInputBinding {
            input_id: input_id.clone(),
            source: "upstream".to_string(),
            producer_node_id: Some(produced.producer_node_id.clone()),
            fake_run_path: Some(produced.fake_run_path.clone()),
        });
    }
    Ok(bindings)
}

fn render_node_command_script(row: &EssentialPipelineRenderedCommandRow) -> String {
    let mut rendered = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    rendered.push_str(&format!(
        "# pipeline command fake-run script\n# pipeline_id={}\n# node_id={}\n# stage_id={}\n# tool_id={}\n\n",
        row.pipeline_id, row.node_id, row.stage_id, row.tool_id
    ));
    for command in &row.script_commands {
        rendered.push_str(command);
        rendered.push('\n');
    }
    rendered
}

fn render_node_stdout(
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    row: &EssentialPipelineRenderedCommandRow,
    upstream_bindings: &[EssentialPipelineFakeRunInputBinding],
    external_bindings: &[EssentialPipelineFakeRunInputBinding],
) -> String {
    let mut rendered = format!(
        "fake local essential pipeline run\npipeline_id={pipeline_id}\nnode_id={}\nstage_id={}\ntool_id={}\ncommand_source={}\nreadiness_kind={}\n",
        node.node_id, node.stage_id, row.tool_id, row.command_source, node.readiness_kind
    );
    if !external_bindings.is_empty() {
        rendered.push_str("external_inputs=\n");
        for binding in external_bindings {
            rendered.push_str(&format!("  - {}\n", binding.input_id));
        }
    }
    if !upstream_bindings.is_empty() {
        rendered.push_str("upstream_inputs=\n");
        for binding in upstream_bindings {
            rendered.push_str(&format!(
                "  - {} <= {} ({})\n",
                binding.input_id,
                binding.producer_node_id.as_deref().unwrap_or("unknown_node"),
                binding.fake_run_path.as_deref().unwrap_or("missing_path")
            ));
        }
    }
    rendered.push_str("commands=\n");
    for command in &row.script_commands {
        rendered.push_str(&format!("  - {command}\n"));
    }
    rendered
}

fn node_output_path(node_root: &Path, output_id: &str) -> PathBuf {
    node_root.join("declared-outputs").join(output_relative_path(output_id))
}

fn output_relative_path(output_id: &str) -> PathBuf {
    let file_name = if output_id.ends_with("_bundle") {
        return PathBuf::from(output_id);
    } else if let Some(stem) = output_id.strip_suffix("_vcf_tbi") {
        format!("{stem}.vcf.tbi")
    } else if let Some(stem) = output_id.strip_suffix("_vcf") {
        format!("{stem}.vcf")
    } else if let Some(stem) = output_id.strip_suffix("_bam") {
        format!("{stem}.bam")
    } else if let Some(stem) = output_id.strip_suffix("_bai") {
        format!("{stem}.bai")
    } else if let Some(stem) = output_id.strip_suffix("_json") {
        format!("{stem}.json")
    } else if let Some(stem) = output_id.strip_suffix("_tsv") {
        format!("{stem}.tsv")
    } else if let Some(stem) = output_id.strip_suffix("_r1_path") {
        format!("{stem}.R1.fastq")
    } else if let Some(stem) = output_id.strip_suffix("_r2_path") {
        format!("{stem}.R2.fastq")
    } else if output_id.contains("reads") {
        format!("{output_id}.fastq")
    } else if output_id.contains("representatives") {
        format!("{output_id}.fasta")
    } else if output_id.contains("classification") || output_id.ends_with("_table") {
        format!("{output_id}.tsv")
    } else if output_id.contains("report")
        || output_id.contains("metrics")
        || output_id.contains("summary")
        || output_id.contains("decision")
        || output_id.contains("manifest")
    {
        format!("{output_id}.json")
    } else {
        format!("{output_id}.txt")
    };
    PathBuf::from(file_name)
}

fn materialize_pipeline_output(
    output_path: &Path,
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    output_id: &str,
) -> Result<()> {
    if output_path_is_directory(output_id) {
        fs::create_dir_all(output_path)
            .with_context(|| format!("create {}", output_path.display()))?;
        let sentinel = output_path.join(".bijux-pipeline-fake-run-placeholder");
        fs::write(
            &sentinel,
            format!(
                "pipeline fake-run directory placeholder\npipeline_id={pipeline_id}\nnode_id={}\nstage_id={}\noutput_id={output_id}\n",
                node.node_id, node.stage_id
            ),
        )
        .with_context(|| format!("write {}", sentinel.display()))?;
        return Ok(());
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(output_path, fake_output_bytes(pipeline_id, node, output_id, output_path)?)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(())
}

fn fake_output_bytes(
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    output_id: &str,
    output_path: &Path,
) -> Result<Vec<u8>> {
    if binary_output_extension(output_path) {
        return Ok(Vec::new());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("json") {
        return serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.bench.local_essential_pipeline_fake_output.v1",
            "pipeline_id": pipeline_id,
            "node_id": node.node_id,
            "stage_id": node.stage_id,
            "output_id": output_id,
        }))
        .context("serialize essential pipeline fake JSON output");
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("tsv") {
        return Ok(format!(
            "pipeline_id\tnode_id\tstage_id\toutput_id\n{pipeline_id}\t{}\t{}\t{output_id}\n",
            node.node_id, node.stage_id
        )
        .into_bytes());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("html") {
        return Ok(format!(
            "<html><body><h1>fake local essential pipeline output</h1><p>{pipeline_id}</p><p>{}</p><p>{output_id}</p></body></html>\n",
            node.node_id
        )
        .into_bytes());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("fastq") {
        return Ok(format!(
            "@{}_{}_{}\nACGT\n+\nIIII\n",
            pipeline_id.replace('-', "_"),
            node.node_id.replace('.', "_"),
            output_id.replace('.', "_")
        )
        .into_bytes());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("fasta") {
        return Ok(format!(
            ">{}_{}_{}\nACGTACGT\n",
            pipeline_id.replace('-', "_"),
            node.node_id.replace('.', "_"),
            output_id.replace('.', "_")
        )
        .into_bytes());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("vcf") {
        return Ok(format!(
            "##fileformat=VCFv4.3\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\nchr1\t1\t{}\tA\tG\t.\tPASS\tPIPELINE={pipeline_id};NODE={}\n",
            output_id.replace('.', "_"),
            node.node_id
        )
        .into_bytes());
    }

    Ok(format!(
        "fake local essential pipeline output\npipeline_id={pipeline_id}\nnode_id={}\nstage_id={}\noutput_id={output_id}\n",
        node.node_id, node.stage_id
    )
    .into_bytes())
}

fn output_path_is_directory(output_id: &str) -> bool {
    output_id.ends_with("_bundle")
}

fn binary_output_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext, "bam" | "bai" | "tbi"))
}

fn materialized_path_size(path: &Path) -> Result<u64> {
    if path.is_dir() {
        let mut total = 0u64;
        for entry in fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
            let entry = entry?;
            total += materialized_path_size(&entry.path())?;
        }
        return Ok(total);
    }
    Ok(fs::metadata(path).with_context(|| format!("stat {}", path.display()))?.len())
}

fn output_role(output_id: &str) -> &'static str {
    if output_id.contains("manifest") {
        "manifest"
    } else if output_id.contains("metrics") {
        "metrics"
    } else if output_id.contains("report") || output_id.contains("summary") {
        "report"
    } else if output_id.ends_with("_vcf") {
        "vcf"
    } else if output_id.ends_with("_vcf_tbi") || output_id.ends_with("_bai") {
        "index"
    } else if output_id.ends_with("_bam") {
        "bam"
    } else if output_id.contains("reads") {
        "reads"
    } else if output_id.contains("table") || output_id.contains("classification") {
        "table"
    } else if output_id.contains("representatives") {
        "sequences"
    } else if output_id.ends_with("_id") || output_id.contains("decision") {
        "metadata"
    } else if output_id.ends_with("_bundle") {
        "bundle"
    } else {
        "artifact"
    }
}

fn simulated_elapsed_seconds(command_step_count: usize) -> f64 {
    1.0 + (command_step_count as f64 * 0.25)
}

fn stage_domain(stage_id: &str) -> Result<&'static str> {
    if stage_id.starts_with("fastq.") {
        Ok("fastq")
    } else if stage_id.starts_with("bam.") {
        Ok("bam")
    } else if stage_id.starts_with("vcf.") {
        Ok("vcf")
    } else {
        Err(anyhow!(
            "essential pipeline fake-runner cannot derive a domain from stage `{stage_id}`"
        ))
    }
}

fn ensure_node_report_contract(
    _pipeline_root: &Path,
    report: &EssentialPipelineFakeRunNodeReport,
) -> Result<()> {
    if report.declared_output_count == 0
        || report.created_output_count != report.declared_output_count
    {
        return Err(anyhow!(
            "essential pipeline fake-runner node `{}` / `{}` did not materialize every declared output",
            report.pipeline_id,
            report.node_id
        ));
    }
    for relative_path in [
        &report.command_script_path,
        &report.stdout_path,
        &report.stderr_path,
        &report.metrics_path,
        &report.stage_result_path,
    ] {
        if relative_path.trim().is_empty() {
            return Err(anyhow!(
                "essential pipeline fake-runner node `{}` / `{}` has an empty artifact path",
                report.pipeline_id,
                report.node_id
            ));
        }
    }
    if report.outputs.iter().any(|output| !output.exists || output.fake_run_path.trim().is_empty())
    {
        return Err(anyhow!(
            "essential pipeline fake-runner node `{}` / `{}` is missing a materialized output path",
            report.pipeline_id,
            report.node_id
        ));
    }
    Ok(())
}

fn ensure_essential_pipeline_fake_run_contract(
    report: &EssentialPipelineFakeRunsReport,
) -> Result<()> {
    if report.pipeline_count != ESSENTIAL_PIPELINE_IDS.len() {
        return Err(anyhow!(
            "essential pipeline fake-runner must cover all {} governed pipelines",
            ESSENTIAL_PIPELINE_IDS.len()
        ));
    }
    let unique_pipelines = report
        .pipelines
        .iter()
        .map(|pipeline| pipeline.pipeline_id.as_str())
        .collect::<BTreeSet<_>>();
    if unique_pipelines.len() != report.pipelines.len() {
        return Err(anyhow!("essential pipeline fake-runner cannot repeat pipeline identifiers"));
    }
    let unique_nodes = report
        .pipelines
        .iter()
        .flat_map(|pipeline| {
            pipeline.nodes.iter().map(|node| format!("{}:{}", pipeline.pipeline_id, node.node_id))
        })
        .collect::<BTreeSet<_>>();
    if unique_nodes.len() != report.node_count {
        return Err(anyhow!(
            "essential pipeline fake-runner must keep exactly one node record per governed pipeline node"
        ));
    }
    if report.pipelines.iter().map(|pipeline| pipeline.node_count).sum::<usize>()
        != report.node_count
    {
        return Err(anyhow!(
            "essential pipeline fake-runner root counts must match pipeline node counts"
        ));
    }
    Ok(())
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
    use super::{output_relative_path, output_role};

    #[test]
    fn output_relative_path_keeps_expected_artifact_shapes() {
        assert_eq!(
            output_relative_path("validated_reads_r1_path").to_string_lossy(),
            "validated_reads.R1.fastq"
        );
        assert_eq!(output_relative_path("called_vcf").to_string_lossy(), "called.vcf");
        assert_eq!(output_relative_path("stats_json").to_string_lossy(), "stats.json");
        assert_eq!(output_relative_path("qc_bundle").to_string_lossy(), "qc_bundle");
    }

    #[test]
    fn output_role_classifies_pipeline_symbols() {
        assert_eq!(output_role("called_vcf"), "vcf");
        assert_eq!(output_role("called_vcf_tbi"), "index");
        assert_eq!(output_role("trim_metrics"), "metrics");
        assert_eq!(output_role("prepared_panel_panel_id"), "metadata");
        assert_eq!(output_role("qc_bundle"), "bundle");
    }
}
