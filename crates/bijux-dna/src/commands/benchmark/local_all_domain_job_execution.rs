use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::readiness::all_domain_rendered_commands::collect_all_domain_rendered_command_rows;
use super::readiness::essential_pipeline_rendered_commands::collect_essential_pipeline_rendered_command_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_ALL_DOMAIN_BENCHMARK_RESULT_EXECUTION_SCHEMA_VERSION: &str =
    "bijux.bench.local_all_domain_benchmark_result_execution.v1";
const LOCAL_ESSENTIAL_PIPELINE_NODE_EXECUTION_SCHEMA_VERSION: &str =
    "bijux.bench.local_essential_pipeline_node_execution.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalAllDomainBenchmarkResultExecutionReport {
    pub(crate) schema_version: &'static str,
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) command_step_count: usize,
    pub(crate) executed_command_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalEssentialPipelineNodeExecutionReport {
    pub(crate) schema_version: &'static str,
    pub(crate) pipeline_id: String,
    pub(crate) node_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) command_step_count: usize,
    pub(crate) executed_command_count: usize,
}

pub(crate) fn run_execute_all_domain_benchmark_result(
    args: &parse::BenchLocalExecuteAllDomainBenchmarkResultArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let row = collect_all_domain_rendered_command_rows(&repo_root)?
        .into_iter()
        .find(|row| row.result_id == args.result_id)
        .ok_or_else(|| anyhow!("unknown all-domain benchmark result `{}`", args.result_id))?;
    execute_script_commands(&repo_root, &row.script_commands)
        .with_context(|| format!("execute all-domain benchmark result `{}`", args.result_id))?;
    let report = BenchLocalAllDomainBenchmarkResultExecutionReport {
        schema_version: LOCAL_ALL_DOMAIN_BENCHMARK_RESULT_EXECUTION_SCHEMA_VERSION,
        result_id: row.result_id,
        domain: row.domain,
        stage_id: row.stage_id,
        tool_id: row.tool_id,
        command_step_count: row.command_steps.len(),
        executed_command_count: row.script_commands.len(),
    };
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.result_id);
    }
    Ok(())
}

pub(crate) fn run_execute_essential_pipeline_node(
    args: &parse::BenchLocalExecuteEssentialPipelineNodeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let row = collect_essential_pipeline_rendered_command_rows(&repo_root)?
        .into_iter()
        .find(|row| row.pipeline_id == args.pipeline_id && row.node_id == args.node_id)
        .ok_or_else(|| {
            anyhow!("unknown essential pipeline node `{}` / `{}`", args.pipeline_id, args.node_id)
        })?;
    if row.render_status != "rendered" {
        return Err(anyhow!(
            "essential pipeline node `{}` / `{}` is not rendered: `{}`",
            args.pipeline_id,
            args.node_id,
            row.render_status
        ));
    }
    execute_script_commands(&repo_root, &row.script_commands).with_context(|| {
        format!("execute essential pipeline node `{}` / `{}`", args.pipeline_id, args.node_id)
    })?;
    let report = BenchLocalEssentialPipelineNodeExecutionReport {
        schema_version: LOCAL_ESSENTIAL_PIPELINE_NODE_EXECUTION_SCHEMA_VERSION,
        pipeline_id: row.pipeline_id,
        node_id: row.node_id,
        domain: row.domain,
        stage_id: row.stage_id,
        tool_id: row.tool_id,
        command_step_count: row.command_steps.len(),
        executed_command_count: row.script_commands.len(),
    };
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}::{}", report.pipeline_id, report.node_id);
    }
    Ok(())
}

fn execute_script_commands(repo_root: &Path, commands: &[String]) -> Result<()> {
    if commands.is_empty() {
        return Err(anyhow!("execution wrapper received an empty script command list"));
    }
    for command in commands {
        run_shell_command(repo_root, command)?;
    }
    Ok(())
}

fn run_shell_command(repo_root: &Path, command: &str) -> Result<()> {
    let output = Command::new("sh")
        .current_dir(repo_root)
        .arg("-c")
        .arg(command)
        .output()
        .with_context(|| format!("run shell command `{command}`"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "shell command failed with status {}: {}\nstdout:\n{}\nstderr:\n{}",
            output.status.code().map_or_else(|| "signal".to_string(), |value| value.to_string()),
            command,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub(crate) fn rendered_benchmark_result_execution_argv(result_id: &str) -> Vec<String> {
    vec![
        "bijux-dna".to_string(),
        "bench".to_string(),
        "local".to_string(),
        "execute-all-domain-benchmark-result".to_string(),
        "--result-id".to_string(),
        result_id.to_string(),
    ]
}

pub(crate) fn rendered_essential_pipeline_node_execution_argv(
    pipeline_id: &str,
    node_id: &str,
) -> Vec<String> {
    vec![
        "bijux-dna".to_string(),
        "bench".to_string(),
        "local".to_string(),
        "execute-essential-pipeline-node".to_string(),
        "--pipeline-id".to_string(),
        pipeline_id.to_string(),
        "--node-id".to_string(),
        node_id.to_string(),
    ]
}

pub(crate) fn render_shell_command(argv: &[String]) -> String {
    argv.iter().map(|segment| shell_quote(segment)).collect::<Vec<_>>().join(" ")
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':' | '+'))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
