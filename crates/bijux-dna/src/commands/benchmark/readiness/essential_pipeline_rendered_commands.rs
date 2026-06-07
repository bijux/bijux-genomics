use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::benchmark_command_rows::{
    collect_selected_bam_command_rows, collect_selected_fastq_command_rows, render_shell_command,
    BenchmarkCommandRow,
};
use super::essential_pipeline_corpus_assets::ESSENTIAL_PIPELINE_IDS;
use super::vcf_rendered_command_rows::{
    collect_vcf_adapter_command_row_map, VcfRenderedCommandRow, VcfRenderedCommandStep,
};
use crate::commands::benchmark::local_pipeline_dag::{
    validate_pipeline_dag_path, LocalPipelineDagValidationNodeReport,
};
use crate::commands::benchmark::local_stage_commands::rendered_stage_materialize_argv;
use crate::commands::benchmark::local_vcf_stage_catalog::build_vcf_stage_catalog_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH: &str =
    "benchmarks/readiness/essential-pipelines-rendered-commands.sh";
const ESSENTIAL_PIPELINE_RENDERED_COMMANDS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.essential_pipeline_rendered_commands.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelineRenderedCommandStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) consumes_previous_stdout: bool,
    pub(crate) argv: Vec<String>,
    pub(crate) command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelineStructuredSkip {
    pub(crate) condition_id: String,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelineRenderedCommandRow {
    pub(crate) pipeline_id: String,
    pub(crate) node_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) domain: String,
    pub(crate) readiness_kind: String,
    pub(crate) render_status: String,
    pub(crate) command_source: String,
    pub(crate) command_steps: Vec<EssentialPipelineRenderedCommandStep>,
    pub(crate) script_commands: Vec<String>,
    pub(crate) skip: Option<EssentialPipelineStructuredSkip>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelineRenderedCommandArgvStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) consumes_previous_stdout: bool,
    pub(crate) argv: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelineRenderedCommandArgvRow {
    pub(crate) pipeline_id: String,
    pub(crate) node_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) domain: String,
    pub(crate) readiness_kind: String,
    pub(crate) render_status: String,
    pub(crate) command_source: String,
    pub(crate) command_steps: Vec<EssentialPipelineRenderedCommandArgvStep>,
    pub(crate) skip: Option<EssentialPipelineStructuredSkip>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EssentialPipelineRenderedCommandsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) argv_output_path: String,
    pub(crate) pipeline_count: usize,
    pub(crate) row_count: usize,
    pub(crate) rendered_row_count: usize,
    pub(crate) structured_skip_row_count: usize,
    pub(crate) domain_row_counts: BTreeMap<String, usize>,
    pub(crate) render_status_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<EssentialPipelineRenderedCommandRow>,
}

#[derive(Default)]
struct EssentialPipelineCommandCaches {
    fastq_rows: BTreeMap<String, std::result::Result<BenchmarkCommandRow, String>>,
    bam_rows: BTreeMap<String, std::result::Result<BenchmarkCommandRow, String>>,
    vcf_default_tool_by_stage: BTreeMap<String, String>,
    vcf_rows_by_stage: BTreeMap<String, VcfRenderedCommandRow>,
}

pub(crate) fn run_render_essential_pipeline_commands(
    args: &parse::BenchReadinessRenderEssentialPipelineCommandsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_essential_pipeline_commands(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_essential_pipeline_commands(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<EssentialPipelineRenderedCommandsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let argv_output_path = companion_argv_output_path(&output_path);
    let rows = collect_essential_pipeline_rendered_command_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if let Some(parent) = argv_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let rendered_script = render_essential_pipeline_commands_shell_script(&rows);
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered_script.as_bytes())?;
    let argv_jsonl = render_essential_pipeline_command_argv_jsonl(&rows)
        .context("render essential pipeline command argv JSONL")?;
    bijux_dna_infra::atomic_write_bytes(&argv_output_path, argv_jsonl.as_bytes())?;

    let mut domain_row_counts = BTreeMap::<String, usize>::new();
    let mut render_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_row_counts.entry(row.domain.clone()).or_default() += 1;
        *render_status_counts.entry(row.render_status.clone()).or_default() += 1;
    }
    let rendered_row_count = render_status_counts.get("rendered").copied().unwrap_or_default();
    let structured_skip_row_count =
        render_status_counts.get("structured_skip").copied().unwrap_or_default();

    Ok(EssentialPipelineRenderedCommandsReport {
        schema_version: ESSENTIAL_PIPELINE_RENDERED_COMMANDS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        argv_output_path: path_relative_to_repo(repo_root, &argv_output_path),
        pipeline_count: ESSENTIAL_PIPELINE_IDS.len(),
        row_count: rows.len(),
        rendered_row_count,
        structured_skip_row_count,
        domain_row_counts,
        render_status_counts,
        rows,
    })
}

pub(crate) fn collect_essential_pipeline_rendered_command_rows(
    repo_root: &Path,
) -> Result<Vec<EssentialPipelineRenderedCommandRow>> {
    let mut caches = initialize_command_caches(repo_root)?;
    let mut rows = Vec::new();

    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        let config_path =
            crate::commands::benchmark::local_pipeline_dag::benchmark_local_pipeline_config_path(
                repo_root,
                pipeline_id,
            );
        let report_path =
            repo_root.join("target/local-ready/pipeline-dag").join(format!("{pipeline_id}.json"));
        let report = validate_pipeline_dag_path(repo_root, &config_path, &report_path)?;
        let nodes_by_id = report
            .nodes
            .iter()
            .map(|node| (node.node_id.clone(), node))
            .collect::<BTreeMap<_, _>>();

        for node_id in &report.topological_order {
            let node = nodes_by_id.get(node_id).ok_or_else(|| {
                anyhow!(
                    "essential pipeline rendered command report is missing node `{node_id}` in pipeline `{pipeline_id}`"
                )
            })?;
            rows.push(build_row(&report.pipeline_id, node, &mut caches)?);
        }
    }

    ensure_essential_pipeline_rendered_command_contract(&rows)?;
    Ok(rows)
}

fn initialize_command_caches(repo_root: &Path) -> Result<EssentialPipelineCommandCaches> {
    let mut fastq_stage_ids = BTreeSet::new();
    let mut bam_stage_ids = BTreeSet::new();
    let mut vcf_stage_ids = BTreeSet::new();

    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        let config_path =
            crate::commands::benchmark::local_pipeline_dag::benchmark_local_pipeline_config_path(
                repo_root,
                pipeline_id,
            );
        let report_path =
            repo_root.join("target/local-ready/pipeline-dag").join(format!("{pipeline_id}.json"));
        let report = validate_pipeline_dag_path(repo_root, &config_path, &report_path)?;
        for node in &report.nodes {
            match stage_domain(node.stage_id.as_str())? {
                "fastq" => {
                    fastq_stage_ids.insert(node.stage_id.clone());
                }
                "bam" => {
                    bam_stage_ids.insert(node.stage_id.clone());
                }
                "vcf" => {
                    vcf_stage_ids.insert(node.stage_id.clone());
                }
                other => {
                    return Err(anyhow!(
                        "essential pipeline rendered command report encountered unsupported domain `{other}` for stage `{}`",
                        node.stage_id
                    ));
                }
            }
        }
    }

    let mut caches = EssentialPipelineCommandCaches::default();
    for stage_id in fastq_stage_ids {
        caches.fastq_rows.insert(
            stage_id.clone(),
            load_selected_fastq_row(repo_root, stage_id.as_str())
                .map_err(|error| error.to_string()),
        );
    }
    for stage_id in bam_stage_ids {
        caches.bam_rows.insert(
            stage_id.clone(),
            load_selected_bam_row(repo_root, stage_id.as_str()).map_err(|error| error.to_string()),
        );
    }

    let vcf_stage_catalog = build_vcf_stage_catalog_rows()?;
    for row in vcf_stage_catalog {
        if vcf_stage_ids.contains(&row.stage_id) {
            caches.vcf_default_tool_by_stage.insert(row.stage_id, row.default_tool_id);
        }
    }

    let vcf_adapter_rows = collect_vcf_adapter_command_row_map(repo_root)?;
    for stage_id in vcf_stage_ids {
        let tool_id = caches.vcf_default_tool_by_stage.get(&stage_id).ok_or_else(|| {
            anyhow!(
                "essential pipeline rendered command report is missing a governed VCF default tool for stage `{stage_id}`"
            )
        })?;
        if let Some(row) = vcf_adapter_rows.get(&(stage_id.clone(), tool_id.clone())) {
            caches.vcf_rows_by_stage.insert(stage_id, row.clone());
        }
    }

    Ok(caches)
}

fn build_row(
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    caches: &mut EssentialPipelineCommandCaches,
) -> Result<EssentialPipelineRenderedCommandRow> {
    match stage_domain(node.stage_id.as_str())? {
        "fastq" => {
            let row = caches.fastq_rows.get(&node.stage_id).ok_or_else(|| {
                anyhow!(
                    "essential pipeline rendered command report is missing a FASTQ cache row for `{}`",
                    node.stage_id
                )
            })?;
            Ok(match row {
                Ok(command) => rendered_row_from_benchmark(
                    pipeline_id,
                    node,
                    "fastq",
                    "fastq_governed_stage_command",
                    command,
                ),
                Err(detail) => fallback_or_skip_row(
                    pipeline_id,
                    node,
                    "fastq",
                    "fastq_governed_stage_command",
                    detail.clone(),
                ),
            })
        }
        "bam" => {
            let row = caches.bam_rows.get(&node.stage_id).ok_or_else(|| {
                anyhow!(
                    "essential pipeline rendered command report is missing a BAM cache row for `{}`",
                    node.stage_id
                )
            })?;
            Ok(match row {
                Ok(command) => rendered_row_from_benchmark(
                    pipeline_id,
                    node,
                    "bam",
                    "bam_governed_stage_command",
                    command,
                ),
                Err(detail) => fallback_or_skip_row(
                    pipeline_id,
                    node,
                    "bam",
                    "bam_governed_stage_command",
                    detail.clone(),
                ),
            })
        }
        "vcf" => {
            let tool_id = caches
                .vcf_default_tool_by_stage
                .get(&node.stage_id)
                .cloned()
                .ok_or_else(|| {
                    anyhow!(
                        "essential pipeline rendered command report is missing a VCF default tool for `{}`",
                        node.stage_id
                    )
                })?;
            Ok(match caches.vcf_rows_by_stage.get(&node.stage_id) {
                Some(command) => rendered_row_from_vcf(
                    pipeline_id,
                    node,
                    "vcf",
                    "vcf_default_tool_adapter",
                    command,
                ),
                None => structured_skip_row(
                    pipeline_id,
                    node,
                    "vcf",
                    tool_id.clone(),
                    "vcf_default_tool_adapter",
                    "default_tool_command_missing",
                    format!(
                        "stage `{}` default tool `{tool_id}` has no rendered command row in the governed VCF adapter command map",
                        node.stage_id
                    ),
                ),
            })
        }
        other => Err(anyhow!(
            "essential pipeline rendered command report encountered unsupported domain `{other}` for stage `{}`",
            node.stage_id
        )),
    }
}

fn rendered_row_from_benchmark(
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    domain: &str,
    command_source: &str,
    command: &BenchmarkCommandRow,
) -> EssentialPipelineRenderedCommandRow {
    let command_text = render_shell_command(&command.argv);
    EssentialPipelineRenderedCommandRow {
        pipeline_id: pipeline_id.to_string(),
        node_id: node.node_id.clone(),
        stage_id: node.stage_id.clone(),
        tool_id: command.tool_id.clone(),
        domain: domain.to_string(),
        readiness_kind: node.readiness_kind.clone(),
        render_status: "rendered".to_string(),
        command_source: command_source.to_string(),
        command_steps: vec![EssentialPipelineRenderedCommandStep {
            step_id: "main".to_string(),
            step_kind: "argv".to_string(),
            consumes_previous_stdout: false,
            argv: command.argv.clone(),
            command: command_text.clone(),
        }],
        script_commands: vec![command_text],
        skip: None,
        reason: format!(
            "pipeline node `{}` / `{}` renders a governed {} command through `{}`",
            pipeline_id, node.node_id, domain, command_source
        ),
    }
}

fn rendered_row_from_vcf(
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    domain: &str,
    command_source: &str,
    command: &VcfRenderedCommandRow,
) -> EssentialPipelineRenderedCommandRow {
    EssentialPipelineRenderedCommandRow {
        pipeline_id: pipeline_id.to_string(),
        node_id: node.node_id.clone(),
        stage_id: node.stage_id.clone(),
        tool_id: command.tool_id.clone(),
        domain: domain.to_string(),
        readiness_kind: node.readiness_kind.clone(),
        render_status: "rendered".to_string(),
        command_source: command_source.to_string(),
        command_steps: command.command_steps.iter().map(rendered_step_from_vcf).collect(),
        script_commands: command.script_commands.clone(),
        skip: None,
        reason: format!(
            "pipeline node `{}` / `{}` renders the governed VCF default tool command through `{}`",
            pipeline_id, node.node_id, command_source
        ),
    }
}

fn rendered_step_from_vcf(step: &VcfRenderedCommandStep) -> EssentialPipelineRenderedCommandStep {
    EssentialPipelineRenderedCommandStep {
        step_id: step.step_id.clone(),
        step_kind: step.step_kind.clone(),
        consumes_previous_stdout: step.consumes_previous_stdout,
        argv: step.argv.clone(),
        command: step.command.clone(),
    }
}

fn structured_skip_row(
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    domain: &str,
    tool_id: impl Into<String>,
    command_source: &str,
    condition_id: &str,
    detail: String,
) -> EssentialPipelineRenderedCommandRow {
    EssentialPipelineRenderedCommandRow {
        pipeline_id: pipeline_id.to_string(),
        node_id: node.node_id.clone(),
        stage_id: node.stage_id.clone(),
        tool_id: tool_id.into(),
        domain: domain.to_string(),
        readiness_kind: node.readiness_kind.clone(),
        render_status: "structured_skip".to_string(),
        command_source: command_source.to_string(),
        command_steps: Vec::new(),
        script_commands: Vec::new(),
        skip: Some(EssentialPipelineStructuredSkip {
            condition_id: condition_id.to_string(),
            detail: detail.clone(),
        }),
        reason: detail,
    }
}

fn fallback_or_skip_row(
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    domain: &str,
    command_source: &str,
    detail: String,
) -> EssentialPipelineRenderedCommandRow {
    if stage_uses_materialize_stage_fallback(node.stage_id.as_str())
        || detail.contains("requires the `bam_downstream` feature")
    {
        return rendered_row_from_materialize_stage(
            pipeline_id,
            node,
            domain,
            "local_stage_materialization",
        );
    }
    structured_skip_row(
        pipeline_id,
        node,
        domain,
        "unknown_tool",
        command_source,
        "governed_stage_command_unavailable",
        detail,
    )
}

fn rendered_row_from_materialize_stage(
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    domain: &str,
    command_source: &str,
) -> EssentialPipelineRenderedCommandRow {
    let argv = rendered_stage_materialize_argv(node.stage_id.as_str());
    let command = render_shell_command(&argv);
    EssentialPipelineRenderedCommandRow {
        pipeline_id: pipeline_id.to_string(),
        node_id: node.node_id.clone(),
        stage_id: node.stage_id.clone(),
        tool_id: "bijux-dna".to_string(),
        domain: domain.to_string(),
        readiness_kind: node.readiness_kind.clone(),
        render_status: "rendered".to_string(),
        command_source: command_source.to_string(),
        command_steps: vec![EssentialPipelineRenderedCommandStep {
            step_id: "main".to_string(),
            step_kind: "argv".to_string(),
            consumes_previous_stdout: false,
            argv,
            command: command.clone(),
        }],
        script_commands: vec![command],
        skip: None,
        reason: format!(
            "pipeline node `{}` / `{}` renders through the governed local stage materialization command",
            pipeline_id, node.node_id
        ),
    }
}

fn load_selected_fastq_row(repo_root: &Path, stage_id: &str) -> Result<BenchmarkCommandRow> {
    let stage_ids = BTreeSet::from([stage_id.to_string()]);
    collect_selected_fastq_command_rows(repo_root, &stage_ids)?
        .remove(stage_id)
        .ok_or_else(|| anyhow!("missing selected FASTQ command row for `{stage_id}`"))
}

fn load_selected_bam_row(repo_root: &Path, stage_id: &str) -> Result<BenchmarkCommandRow> {
    let stage_ids = BTreeSet::from([stage_id.to_string()]);
    collect_selected_bam_command_rows(repo_root, &stage_ids)?
        .remove(stage_id)
        .ok_or_else(|| anyhow!("missing selected BAM command row for `{stage_id}`"))
}

fn render_essential_pipeline_commands_shell_script(
    rows: &[EssentialPipelineRenderedCommandRow],
) -> String {
    let mut rendered = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    rendered.push_str("repo_root=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/../..\" && pwd)\"\n");
    rendered.push_str("cd \"$repo_root\"\n\n");
    for row in rows {
        rendered.push_str(&format!(
            "# {} / {} / {} / {}\n",
            row.pipeline_id, row.node_id, row.stage_id, row.tool_id
        ));
        if let Some(skip) = &row.skip {
            rendered.push_str(&format!("# skip: {} - {}\n\n", skip.condition_id, skip.detail));
            continue;
        }
        for command in &row.script_commands {
            rendered.push_str(command);
            rendered.push('\n');
        }
        rendered.push('\n');
    }
    rendered
}

fn render_essential_pipeline_command_argv_jsonl(
    rows: &[EssentialPipelineRenderedCommandRow],
) -> Result<String> {
    let mut rendered = String::new();
    for row in rows {
        let payload = EssentialPipelineRenderedCommandArgvRow {
            pipeline_id: row.pipeline_id.clone(),
            node_id: row.node_id.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            domain: row.domain.clone(),
            readiness_kind: row.readiness_kind.clone(),
            render_status: row.render_status.clone(),
            command_source: row.command_source.clone(),
            command_steps: row
                .command_steps
                .iter()
                .map(|step| EssentialPipelineRenderedCommandArgvStep {
                    step_id: step.step_id.clone(),
                    step_kind: step.step_kind.clone(),
                    consumes_previous_stdout: step.consumes_previous_stdout,
                    argv: step.argv.clone(),
                })
                .collect(),
            skip: row.skip.clone(),
        };
        let line = serde_json::to_string(&payload)
            .context("serialize essential pipeline rendered command argv row")?;
        rendered.push_str(&line);
        rendered.push('\n');
    }
    Ok(rendered)
}

fn ensure_essential_pipeline_rendered_command_contract(
    rows: &[EssentialPipelineRenderedCommandRow],
) -> Result<()> {
    let unique_nodes = rows
        .iter()
        .map(|row| format!("{}:{}", row.pipeline_id, row.node_id))
        .collect::<BTreeSet<_>>();
    if unique_nodes.len() != rows.len() {
        return Err(anyhow!(
            "essential pipeline rendered command report must keep exactly one row per pipeline node"
        ));
    }

    let covered_pipelines =
        rows.iter().map(|row| row.pipeline_id.as_str()).collect::<BTreeSet<_>>();
    if covered_pipelines.len() != ESSENTIAL_PIPELINE_IDS.len() {
        return Err(anyhow!(
            "essential pipeline rendered command report must cover all {} governed essential pipelines",
            ESSENTIAL_PIPELINE_IDS.len()
        ));
    }

    for row in rows {
        if row.pipeline_id.trim().is_empty()
            || row.node_id.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.domain.trim().is_empty()
            || row.readiness_kind.trim().is_empty()
            || row.command_source.trim().is_empty()
        {
            return Err(anyhow!(
                "essential pipeline rendered command row `{}` / `{}` is missing required columns",
                row.pipeline_id,
                row.node_id
            ));
        }
        match row.render_status.as_str() {
            "rendered" => {
                if row.skip.is_some() {
                    return Err(anyhow!(
                        "essential pipeline rendered command row `{}` / `{}` cannot be rendered and skipped at the same time",
                        row.pipeline_id,
                        row.node_id
                    ));
                }
                if row.command_steps.is_empty() || row.script_commands.is_empty() {
                    return Err(anyhow!(
                        "essential pipeline rendered command row `{}` / `{}` must keep command steps and script commands",
                        row.pipeline_id,
                        row.node_id
                    ));
                }
                for step in &row.command_steps {
                    let executable =
                        step.argv.first().map(|value| value.trim()).filter(|value| !value.is_empty()).ok_or_else(|| {
                            anyhow!(
                                "essential pipeline rendered command row `{}` / `{}` step `{}` has an empty executable",
                                row.pipeline_id,
                                row.node_id,
                                step.step_id
                            )
                        })?;
                    let command_lower = step.command.to_ascii_lowercase();
                    if command_lower.contains("todo")
                        || command_lower.contains("placeholder")
                        || (executable.eq_ignore_ascii_case("echo")
                            && command_lower.contains("echo execute"))
                    {
                        return Err(anyhow!(
                            "essential pipeline rendered command row `{}` / `{}` step `{}` still contains placeholder execution text",
                            row.pipeline_id,
                            row.node_id,
                            step.step_id
                        ));
                    }
                }
            }
            "structured_skip" => {
                if !row.command_steps.is_empty() || !row.script_commands.is_empty() {
                    return Err(anyhow!(
                        "essential pipeline rendered command row `{}` / `{}` cannot keep executable commands when it is a structured skip",
                        row.pipeline_id,
                        row.node_id
                    ));
                }
                let Some(skip) = &row.skip else {
                    return Err(anyhow!(
                        "essential pipeline rendered command row `{}` / `{}` must keep a structured skip payload",
                        row.pipeline_id,
                        row.node_id
                    ));
                };
                if skip.condition_id.trim().is_empty() || skip.detail.trim().is_empty() {
                    return Err(anyhow!(
                        "essential pipeline rendered command row `{}` / `{}` has an incomplete structured skip payload",
                        row.pipeline_id,
                        row.node_id
                    ));
                }
            }
            other => {
                return Err(anyhow!(
                    "essential pipeline rendered command row `{}` / `{}` has unsupported render status `{other}`",
                    row.pipeline_id,
                    row.node_id
                ));
            }
        }
    }

    Ok(())
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
            "essential pipeline rendered command report cannot derive a domain from stage `{stage_id}`"
        ))
    }
}

fn stage_uses_materialize_stage_fallback(stage_id: &str) -> bool {
    matches!(stage_id, "fastq.report_qc" | "bam.genotyping")
}

fn companion_argv_output_path(output_path: &Path) -> PathBuf {
    let file_name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH);
    let argv_name = if let Some(stem) = file_name.strip_suffix(".sh") {
        format!("{stem}.argv.jsonl")
    } else {
        format!("{file_name}.argv.jsonl")
    };
    output_path.with_file_name(argv_name)
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
        render_essential_pipeline_commands, DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn essential_pipeline_rendered_commands_report_tracks_governed_nodes() {
        let root = repo_root();
        let report = render_essential_pipeline_commands(
            &root,
            PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH),
        )
        .expect("render essential pipeline commands");

        assert_eq!(
            report.schema_version,
            "bijux.bench.readiness.essential_pipeline_rendered_commands.v1"
        );
        assert_eq!(
            report.output_path,
            "benchmarks/readiness/essential-pipelines-rendered-commands.sh"
        );
        assert_eq!(
            report.argv_output_path,
            "benchmarks/readiness/essential-pipelines-rendered-commands.argv.jsonl"
        );
        assert_eq!(report.pipeline_count, 10);
        assert_eq!(report.row_count, report.rows.len());
        assert!(report.row_count >= 89);
        assert_eq!(report.rendered_row_count + report.structured_skip_row_count, report.row_count);
    }
}
