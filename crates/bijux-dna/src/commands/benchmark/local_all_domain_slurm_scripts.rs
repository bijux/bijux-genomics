use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_all_domain_job_execution::{
    render_shell_command, rendered_benchmark_result_execution_argv,
    rendered_essential_pipeline_node_execution_argv,
};
use super::local_all_domain_result_paths::{benchmark_result_root, essential_pipeline_result_root};
use super::local_pipeline_dag::{validate_pipeline_dag_path, LocalPipelineDagValidationNodeReport};
use super::local_slurm_run_paths::load_fixture_sample_scope_map;
use super::readiness::all_domain_expected_benchmark_results::{
    collect_all_domain_expected_benchmark_result_rows, AllDomainExpectedBenchmarkResultRow,
};
use super::readiness::all_domain_rendered_commands::{
    collect_all_domain_rendered_command_rows, AllDomainRenderedCommandRow,
};
use super::readiness::essential_pipeline_corpus_assets::ESSENTIAL_PIPELINE_IDS;
use super::readiness::essential_pipeline_rendered_commands::{
    collect_essential_pipeline_rendered_command_rows, EssentialPipelineRenderedCommandRow,
};
use super::readiness::stage_tool_resources::{
    StageToolResourcesConfig, DEFAULT_STAGE_TOOL_RESOURCES_PATH,
    LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_VERSION,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT: &str = "target/slurm-dry-run/all-domains";
const LOCAL_ALL_DOMAIN_SLURM_SCRIPTS_SCHEMA_VERSION: &str =
    "bijux.bench.local_all_domain_slurm_scripts.v1";
const DEFAULT_TIME_LIMIT: &str = "00:20:00";
const DEFAULT_FASTQ_CPUS: u32 = 4;
const DEFAULT_FASTQ_MEMORY_MB: u32 = 2048;
const DEFAULT_BAM_CPUS: u32 = 3;
const DEFAULT_BAM_MEMORY_MB: u32 = 2048;
const DEFAULT_VCF_CPUS: u32 = 1;
const DEFAULT_VCF_MEMORY_MB: u32 = 2048;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ResourceKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalAllDomainSlurmScriptEntry {
    pub(crate) job_id_local: String,
    pub(crate) job_kind: String,
    pub(crate) result_id: Option<String>,
    pub(crate) pipeline_id: Option<String>,
    pub(crate) node_id: Option<String>,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) dependency_count: usize,
    pub(crate) cpus_per_task: u32,
    pub(crate) memory_mb: u32,
    pub(crate) time_limit: String,
    pub(crate) script_path: String,
    pub(crate) stdout_path: String,
    pub(crate) stderr_path: String,
    pub(crate) command_step_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalAllDomainSlurmScriptsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_root: String,
    pub(crate) script_count: usize,
    pub(crate) benchmark_job_count: usize,
    pub(crate) essential_pipeline_job_count: usize,
    pub(crate) pipeline_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) benchmark_domain_counts: BTreeMap<String, usize>,
    pub(crate) essential_pipeline_domain_counts: BTreeMap<String, usize>,
    pub(crate) scripts: Vec<BenchLocalAllDomainSlurmScriptEntry>,
}

#[derive(Debug, Clone)]
struct SlurmResourceHint {
    cpus_per_task: u32,
    memory_mb: u32,
    time_limit: String,
}

#[derive(Debug, Clone)]
struct ScriptArtifacts {
    control_root: PathBuf,
    result_root: PathBuf,
    script_path: PathBuf,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

pub(crate) fn run_render_all_domain_slurm_scripts(
    args: &parse::BenchLocalRenderAllDomainSlurmScriptsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_slurm_scripts(
        &repo_root,
        args.output_root
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_root);
    }
    Ok(())
}

pub(crate) fn render_all_domain_slurm_scripts(
    repo_root: &Path,
    output_root: PathBuf,
) -> Result<BenchLocalAllDomainSlurmScriptsReport> {
    let absolute_output_root = repo_relative_path(repo_root, &output_root);
    fs::create_dir_all(&absolute_output_root)
        .with_context(|| format!("create {}", absolute_output_root.display()))?;

    let resource_hints = load_stage_tool_resource_hints(repo_root)?;
    let benchmark_expected_rows = collect_all_domain_expected_benchmark_result_rows(repo_root)?;
    let benchmark_command_rows = collect_all_domain_rendered_command_rows(repo_root)?;
    let essential_pipeline_command_rows =
        collect_essential_pipeline_rendered_command_rows(repo_root)?;

    let benchmark_expected_by_result = benchmark_expected_rows
        .into_iter()
        .map(|row| (row.result_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let benchmark_command_by_result = benchmark_command_rows
        .into_iter()
        .map(|row| (row.result_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    ensure_benchmark_alignment(&benchmark_expected_by_result, &benchmark_command_by_result)?;

    let essential_pipeline_command_by_node = essential_pipeline_command_rows
        .into_iter()
        .map(|row| ((row.pipeline_id.clone(), row.node_id.clone()), row))
        .collect::<BTreeMap<_, _>>();
    let fixture_sample_scopes = load_fixture_sample_scope_map(repo_root)?;

    let mut scripts = Vec::new();
    let mut benchmark_domain_counts = BTreeMap::<String, usize>::new();
    for expected in benchmark_expected_by_result.values() {
        let command = benchmark_command_by_result.get(&expected.result_id).ok_or_else(|| {
            anyhow!(
                "all-domain slurm script generation is missing rendered commands for `{}`",
                expected.result_id
            )
        })?;
        let resource_hint = benchmark_resource_hint(&resource_hints, expected, command);
        let artifacts = benchmark_script_artifacts(&absolute_output_root, expected)?;
        write_all_domain_slurm_script(
            repo_root,
            &artifacts,
            &build_benchmark_script_entry(expected, command, &resource_hint, repo_root, &artifacts),
            build_benchmark_script_body(repo_root, expected, command, &resource_hint, &artifacts),
        )?;
        scripts.push(build_benchmark_script_entry(
            expected,
            command,
            &resource_hint,
            repo_root,
            &artifacts,
        ));
        *benchmark_domain_counts.entry(expected.domain.clone()).or_default() += 1;
    }

    let mut essential_pipeline_domain_counts = BTreeMap::<String, usize>::new();
    let mut pipeline_count = 0usize;
    let mut essential_pipeline_job_count = 0usize;
    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        pipeline_count += 1;
        let config_path =
            repo_root.join("configs/pipelines/local").join(format!("{pipeline_id}.toml"));
        let report_path =
            repo_root.join("target/local-ready/pipeline-dag").join(format!("{pipeline_id}.json"));
        let dag_report = validate_pipeline_dag_path(repo_root, &config_path, &report_path)?;
        let nodes_by_id = dag_report
            .nodes
            .iter()
            .map(|node| (node.node_id.clone(), node))
            .collect::<BTreeMap<_, _>>();
        for node_id in &dag_report.topological_order {
            let node = nodes_by_id.get(node_id).ok_or_else(|| {
                anyhow!(
                    "all-domain slurm script generation is missing node `{node_id}` in pipeline `{pipeline_id}`"
                )
            })?;
            let command = essential_pipeline_command_by_node
                .get(&(pipeline_id.to_string(), node.node_id.clone()))
                .ok_or_else(|| {
                    anyhow!(
                        "all-domain slurm script generation is missing rendered command row for `{pipeline_id}` / `{}`",
                        node.node_id
                    )
                })?;
            if command.render_status != "rendered" {
                return Err(anyhow!(
                    "all-domain slurm script generation requires rendered essential pipeline commands, found `{}` for `{pipeline_id}` / `{}`",
                    command.render_status,
                    node.node_id
                ));
            }
            let resource_hint = essential_pipeline_resource_hint(&resource_hints, node, command);
            let artifacts = essential_pipeline_script_artifacts(
                &absolute_output_root,
                &command.domain,
                pipeline_id,
                &node.node_id,
                &command.tool_id,
                &dag_report.default_corpus_id,
                fixture_sample_scopes
                    .get(&dag_report.default_corpus_id)
                    .map(String::as_str)
                    .unwrap_or("sample-set"),
            );
            let entry = build_essential_pipeline_script_entry(
                pipeline_id,
                node,
                command,
                &dag_report.default_corpus_id,
                &resource_hint,
                repo_root,
                &artifacts,
            );
            let body = build_essential_pipeline_script_body(
                repo_root,
                pipeline_id,
                node,
                command,
                &dag_report.default_corpus_id,
                &resource_hint,
                &artifacts,
            );
            write_all_domain_slurm_script(repo_root, &artifacts, &entry, body)?;
            *essential_pipeline_domain_counts.entry(command.domain.clone()).or_default() += 1;
            essential_pipeline_job_count += 1;
            scripts.push(entry);
        }
    }

    let mut domain_counts = BTreeMap::<String, usize>::new();
    for entry in &scripts {
        *domain_counts.entry(entry.domain.clone()).or_default() += 1;
    }

    let report = BenchLocalAllDomainSlurmScriptsReport {
        schema_version: LOCAL_ALL_DOMAIN_SLURM_SCRIPTS_SCHEMA_VERSION,
        output_root: path_relative_to_repo(repo_root, &absolute_output_root),
        script_count: scripts.len(),
        benchmark_job_count: benchmark_expected_by_result.len(),
        essential_pipeline_job_count,
        pipeline_count,
        domain_counts,
        benchmark_domain_counts,
        essential_pipeline_domain_counts,
        scripts,
    };
    ensure_all_domain_slurm_script_contract(
        &report,
        benchmark_command_by_result.len(),
        expected_essential_pipeline_node_count(repo_root)?,
    )?;
    Ok(report)
}

fn ensure_benchmark_alignment(
    expected_by_result: &BTreeMap<String, AllDomainExpectedBenchmarkResultRow>,
    command_by_result: &BTreeMap<String, AllDomainRenderedCommandRow>,
) -> Result<()> {
    if expected_by_result.len() != command_by_result.len() {
        return Err(anyhow!(
            "all-domain slurm script generation requires one rendered command per expected benchmark result (expected {}, found {})",
            expected_by_result.len(),
            command_by_result.len()
        ));
    }
    for result_id in expected_by_result.keys() {
        if !command_by_result.contains_key(result_id) {
            return Err(anyhow!(
                "all-domain slurm script generation is missing rendered command coverage for `{result_id}`"
            ));
        }
    }
    Ok(())
}

fn expected_essential_pipeline_node_count(repo_root: &Path) -> Result<usize> {
    let mut total = 0usize;
    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        let config_path =
            repo_root.join("configs/pipelines/local").join(format!("{pipeline_id}.toml"));
        let report_path =
            repo_root.join("target/local-ready/pipeline-dag").join(format!("{pipeline_id}.json"));
        total += validate_pipeline_dag_path(repo_root, &config_path, &report_path)?.node_count;
    }
    Ok(total)
}

fn load_stage_tool_resource_hints(
    repo_root: &Path,
) -> Result<BTreeMap<ResourceKey, SlurmResourceHint>> {
    let path = repo_root.join(DEFAULT_STAGE_TOOL_RESOURCES_PATH);
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config = toml::from_str::<StageToolResourcesConfig>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected stage-tool resources schema `{}` in {}",
            config.schema_version,
            path.display()
        ));
    }
    Ok(config
        .rows
        .into_iter()
        .map(|row| {
            (
                ResourceKey { domain: row.domain, stage_id: row.stage_id, tool_id: row.tool_id },
                SlurmResourceHint {
                    cpus_per_task: row.threads.max(1),
                    memory_mb: row.memory_gb.max(1) * 1024,
                    time_limit: minutes_to_time_limit(row.walltime_minutes.max(1)),
                },
            )
        })
        .collect())
}

fn benchmark_resource_hint(
    resource_hints: &BTreeMap<ResourceKey, SlurmResourceHint>,
    expected: &AllDomainExpectedBenchmarkResultRow,
    command: &AllDomainRenderedCommandRow,
) -> SlurmResourceHint {
    let key = ResourceKey {
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
    };
    resource_hints.get(&key).cloned().unwrap_or_else(|| default_resource_hint(&command.domain))
}

fn essential_pipeline_resource_hint(
    resource_hints: &BTreeMap<ResourceKey, SlurmResourceHint>,
    node: &LocalPipelineDagValidationNodeReport,
    command: &EssentialPipelineRenderedCommandRow,
) -> SlurmResourceHint {
    let key = ResourceKey {
        domain: command.domain.clone(),
        stage_id: node.stage_id.clone(),
        tool_id: command.tool_id.clone(),
    };
    resource_hints.get(&key).cloned().unwrap_or_else(|| default_resource_hint(&command.domain))
}

fn default_resource_hint(domain: &str) -> SlurmResourceHint {
    match domain {
        "fastq" => SlurmResourceHint {
            cpus_per_task: DEFAULT_FASTQ_CPUS,
            memory_mb: DEFAULT_FASTQ_MEMORY_MB,
            time_limit: DEFAULT_TIME_LIMIT.to_string(),
        },
        "bam" => SlurmResourceHint {
            cpus_per_task: DEFAULT_BAM_CPUS,
            memory_mb: DEFAULT_BAM_MEMORY_MB,
            time_limit: DEFAULT_TIME_LIMIT.to_string(),
        },
        "vcf" => SlurmResourceHint {
            cpus_per_task: DEFAULT_VCF_CPUS,
            memory_mb: DEFAULT_VCF_MEMORY_MB,
            time_limit: DEFAULT_TIME_LIMIT.to_string(),
        },
        _ => SlurmResourceHint {
            cpus_per_task: 1,
            memory_mb: 1024,
            time_limit: DEFAULT_TIME_LIMIT.to_string(),
        },
    }
}

fn benchmark_script_artifacts(
    output_root: &Path,
    expected: &AllDomainExpectedBenchmarkResultRow,
) -> Result<ScriptArtifacts> {
    let control_root = output_root
        .join("benchmark-results")
        .join(&expected.domain)
        .join(&expected.corpus_id)
        .join(&expected.stage_id)
        .join(&expected.asset_profile_id)
        .join(&expected.tool_id);
    let result_root = benchmark_result_root(
        output_root,
        &expected.domain,
        &expected.stage_id,
        &expected.tool_id,
        &expected.corpus_id,
        &expected.asset_profile_id,
        &expected.result_id,
    )?;
    Ok(ScriptArtifacts {
        script_path: control_root.join("job.sbatch"),
        stdout_path: result_root.join("stdout.log"),
        stderr_path: result_root.join("stderr.log"),
        control_root,
        result_root,
    })
}

fn essential_pipeline_script_artifacts(
    output_root: &Path,
    domain: &str,
    pipeline_id: &str,
    node_id: &str,
    tool_id: &str,
    corpus_id: &str,
    sample_scope: &str,
) -> ScriptArtifacts {
    let control_root = output_root.join("essential-pipelines").join(pipeline_id).join(node_id);
    let result_root = essential_pipeline_result_root(
        output_root,
        domain,
        pipeline_id,
        node_id,
        tool_id,
        corpus_id,
        sample_scope,
    );
    ScriptArtifacts {
        script_path: control_root.join("job.sbatch"),
        stdout_path: result_root.join("stdout.log"),
        stderr_path: result_root.join("stderr.log"),
        control_root,
        result_root,
    }
}

fn build_benchmark_script_entry(
    expected: &AllDomainExpectedBenchmarkResultRow,
    command: &AllDomainRenderedCommandRow,
    resource_hint: &SlurmResourceHint,
    repo_root: &Path,
    artifacts: &ScriptArtifacts,
) -> BenchLocalAllDomainSlurmScriptEntry {
    BenchLocalAllDomainSlurmScriptEntry {
        job_id_local: format!("benchmark:{}", expected.result_id),
        job_kind: "benchmark_result".to_string(),
        result_id: Some(expected.result_id.clone()),
        pipeline_id: None,
        node_id: None,
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        dependency_count: 0,
        cpus_per_task: resource_hint.cpus_per_task,
        memory_mb: resource_hint.memory_mb,
        time_limit: resource_hint.time_limit.clone(),
        script_path: path_relative_to_repo(repo_root, &artifacts.script_path),
        stdout_path: path_relative_to_repo(repo_root, &artifacts.stdout_path),
        stderr_path: path_relative_to_repo(repo_root, &artifacts.stderr_path),
        command_step_count: command.command_steps.len(),
    }
}

fn build_essential_pipeline_script_entry(
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    command: &EssentialPipelineRenderedCommandRow,
    corpus_id: &str,
    resource_hint: &SlurmResourceHint,
    repo_root: &Path,
    artifacts: &ScriptArtifacts,
) -> BenchLocalAllDomainSlurmScriptEntry {
    BenchLocalAllDomainSlurmScriptEntry {
        job_id_local: format!("pipeline:{pipeline_id}:{}", node.node_id),
        job_kind: "essential_pipeline_node".to_string(),
        result_id: None,
        pipeline_id: Some(pipeline_id.to_string()),
        node_id: Some(node.node_id.clone()),
        domain: command.domain.clone(),
        stage_id: node.stage_id.clone(),
        tool_id: command.tool_id.clone(),
        corpus_id: corpus_id.to_string(),
        asset_profile_id: "pipeline_node".to_string(),
        dependency_count: node.depends_on.len(),
        cpus_per_task: resource_hint.cpus_per_task,
        memory_mb: resource_hint.memory_mb,
        time_limit: resource_hint.time_limit.clone(),
        script_path: path_relative_to_repo(repo_root, &artifacts.script_path),
        stdout_path: path_relative_to_repo(repo_root, &artifacts.stdout_path),
        stderr_path: path_relative_to_repo(repo_root, &artifacts.stderr_path),
        command_step_count: command.command_steps.len(),
    }
}

fn build_benchmark_script_body(
    repo_root: &Path,
    expected: &AllDomainExpectedBenchmarkResultRow,
    _command: &AllDomainRenderedCommandRow,
    resource_hint: &SlurmResourceHint,
    artifacts: &ScriptArtifacts,
) -> String {
    let command = render_shell_command(&rendered_benchmark_result_execution_argv(
        expected.result_id.as_str(),
    ));
    build_slurm_script_body(
        repo_root,
        &format!("benchmark:{}", expected.result_id),
        Some(expected.result_id.as_str()),
        None,
        None,
        &expected.domain,
        &expected.stage_id,
        &expected.tool_id,
        &expected.corpus_id,
        &expected.asset_profile_id,
        &[],
        resource_hint,
        &[command],
        artifacts,
    )
}

fn build_essential_pipeline_script_body(
    repo_root: &Path,
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
    command: &EssentialPipelineRenderedCommandRow,
    corpus_id: &str,
    resource_hint: &SlurmResourceHint,
    artifacts: &ScriptArtifacts,
) -> String {
    let execution_command = render_shell_command(&rendered_essential_pipeline_node_execution_argv(
        pipeline_id,
        node.node_id.as_str(),
    ));
    build_slurm_script_body(
        repo_root,
        &format!("pipeline:{pipeline_id}:{}", node.node_id),
        None,
        Some(pipeline_id),
        Some(node.node_id.as_str()),
        &command.domain,
        &node.stage_id,
        &command.tool_id,
        corpus_id,
        "pipeline_node",
        &node.depends_on,
        resource_hint,
        &[execution_command],
        artifacts,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_slurm_script_body(
    repo_root: &Path,
    job_id_local: &str,
    result_id: Option<&str>,
    pipeline_id: Option<&str>,
    node_id: Option<&str>,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    corpus_id: &str,
    asset_profile_id: &str,
    dependencies: &[String],
    resource_hint: &SlurmResourceHint,
    commands: &[String],
    artifacts: &ScriptArtifacts,
) -> String {
    let repo_root_absolute = repo_root.to_string_lossy().replace('\\', "/");
    let job_name = sanitize_job_name(job_id_local);
    let dependency_comment =
        if dependencies.is_empty() { "none".to_string() } else { dependencies.join(",") };
    let mut rendered = format!(
        "#!/usr/bin/env bash\n\
set -euo pipefail\n\
\n\
#SBATCH --job-name={job_name}\n\
#SBATCH --chdir={repo_root}\n\
#SBATCH --cpus-per-task={cpus_per_task}\n\
#SBATCH --mem={memory_mb}M\n\
#SBATCH --time={time_limit}\n\
#SBATCH --output={stdout_path}\n\
#SBATCH --error={stderr_path}\n\
\n\
# Governed all-domain local slurm dry-run script.\n\
# job_id_local: {job_id_local}\n\
# result_id: {result_id}\n\
# pipeline_id: {pipeline_id}\n\
# node_id: {node_id}\n\
# domain: {domain}\n\
# stage_id: {stage_id}\n\
# tool_id: {tool_id}\n\
# corpus_id: {corpus_id}\n\
# asset_profile_id: {asset_profile_id}\n\
# dependency_node_ids: {dependency_comment}\n\
\n\
REPO_ROOT={repo_root}\n\
RESULT_ROOT={result_root}\n\
STDOUT_PATH={stdout_path}\n\
STDERR_PATH={stderr_path}\n\
cd \"$REPO_ROOT\"\n\
mkdir -p \"$RESULT_ROOT\"\n",
        job_name = shell_quote(&job_name),
        repo_root = shell_quote(&repo_root_absolute),
        cpus_per_task = resource_hint.cpus_per_task,
        memory_mb = resource_hint.memory_mb,
        time_limit = resource_hint.time_limit,
        stdout_path = shell_quote(&path_relative_to_repo(repo_root, &artifacts.stdout_path)),
        stderr_path = shell_quote(&path_relative_to_repo(repo_root, &artifacts.stderr_path)),
        job_id_local = job_id_local,
        result_id = result_id.unwrap_or("none"),
        pipeline_id = pipeline_id.unwrap_or("none"),
        node_id = node_id.unwrap_or("none"),
        domain = domain,
        stage_id = stage_id,
        tool_id = tool_id,
        corpus_id = corpus_id,
        asset_profile_id = asset_profile_id,
        dependency_comment = dependency_comment,
        result_root = shell_quote(&path_relative_to_repo(repo_root, &artifacts.result_root)),
    );
    for command in commands {
        rendered.push('\n');
        rendered.push_str(command);
        rendered.push('\n');
    }
    rendered
}

fn write_all_domain_slurm_script(
    _repo_root: &Path,
    artifacts: &ScriptArtifacts,
    _entry: &BenchLocalAllDomainSlurmScriptEntry,
    body: String,
) -> Result<()> {
    fs::create_dir_all(&artifacts.control_root)
        .with_context(|| format!("create {}", artifacts.control_root.display()))?;
    fs::create_dir_all(&artifacts.result_root)
        .with_context(|| format!("create {}", artifacts.result_root.display()))?;
    fs::write(&artifacts.script_path, body)
        .with_context(|| format!("write {}", artifacts.script_path.display()))?;
    Ok(())
}

fn ensure_all_domain_slurm_script_contract(
    report: &BenchLocalAllDomainSlurmScriptsReport,
    expected_benchmark_job_count: usize,
    expected_essential_pipeline_job_count: usize,
) -> Result<()> {
    if report.benchmark_job_count != expected_benchmark_job_count {
        return Err(anyhow!(
            "all-domain slurm script generation must cover every expected benchmark result job (expected {}, found {})",
            expected_benchmark_job_count,
            report.benchmark_job_count
        ));
    }
    if report.essential_pipeline_job_count != expected_essential_pipeline_job_count {
        return Err(anyhow!(
            "all-domain slurm script generation must cover every essential pipeline node (expected {}, found {})",
            expected_essential_pipeline_job_count,
            report.essential_pipeline_job_count
        ));
    }
    if report.script_count != report.benchmark_job_count + report.essential_pipeline_job_count {
        return Err(anyhow!(
            "all-domain slurm script generation must keep script_count equal to benchmark jobs plus essential pipeline jobs"
        ));
    }
    if report.benchmark_domain_counts.get("vcf").copied().unwrap_or_default() == 0 {
        return Err(anyhow!(
            "all-domain slurm script generation must include benchmark-ready VCF jobs"
        ));
    }
    if report.essential_pipeline_job_count == 0 || report.pipeline_count == 0 {
        return Err(anyhow!(
            "all-domain slurm script generation must include essential pipeline jobs"
        ));
    }
    let unique_paths =
        report.scripts.iter().map(|entry| entry.script_path.as_str()).collect::<BTreeSet<_>>();
    if unique_paths.len() != report.scripts.len() {
        return Err(anyhow!(
            "all-domain slurm script generation must keep one unique script path per job"
        ));
    }
    let unique_job_ids =
        report.scripts.iter().map(|entry| entry.job_id_local.as_str()).collect::<BTreeSet<_>>();
    if unique_job_ids.len() != report.scripts.len() {
        return Err(anyhow!(
            "all-domain slurm script generation must keep one unique local job id per script"
        ));
    }
    Ok(())
}

fn minutes_to_time_limit(minutes: u32) -> String {
    let hours = minutes / 60;
    let remainder_minutes = minutes % 60;
    format!("{hours:02}:{remainder_minutes:02}:00")
}

fn sanitize_job_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') { ch } else { '-' })
        .collect()
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':'))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}
