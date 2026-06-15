use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_all_domain_fake_runs::{declared_output_ids, output_relative_path};
use super::local_all_domain_result_paths::{
    benchmark_result_root, essential_pipeline_result_root, LOCAL_ALL_DOMAIN_SLURM_RUN_ID,
};
use super::local_all_domain_slurm_scripts::{
    render_all_domain_slurm_scripts, BenchLocalAllDomainSlurmScriptEntry,
};
use super::local_pipeline_dag::{validate_pipeline_dag_path, LocalPipelineDagValidationNodeReport};
use super::local_slurm_run_paths::load_fixture_sample_scope_map;
use super::readiness::all_domain_output_declarations::{
    collect_all_domain_output_declaration_rows, AllDomainOutputDeclarationRow,
};
use super::readiness::essential_pipeline_corpus_assets::ESSENTIAL_PIPELINE_IDS;
use crate::commands::benchmark::path_resolution::{
    ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.local_all_domain_slurm_submit_manifest.v1";

pub(crate) const DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH: &str =
    "runs/bench/slurm-dry-run/all-domains/submit-manifest.json";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalAllDomainSlurmSubmitManifest {
    pub(crate) schema_version: &'static str,
    pub(crate) root_path: String,
    pub(crate) manifest_path: String,
    pub(crate) run_id: String,
    pub(crate) job_count: usize,
    pub(crate) dependency_count: usize,
    pub(crate) benchmark_job_count: usize,
    pub(crate) essential_pipeline_job_count: usize,
    pub(crate) jobs: Vec<BenchLocalAllDomainSlurmSubmitJob>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalAllDomainSlurmSubmitJob {
    pub(crate) job_id_local: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) pipeline_id: Option<String>,
    pub(crate) node_id: Option<String>,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) result_id: Option<String>,
    pub(crate) script_path: String,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) outputs: Vec<String>,
    pub(crate) dependencies: Vec<String>,
    pub(crate) resources: BenchLocalAllDomainSlurmSubmitResources,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalAllDomainSlurmSubmitResources {
    pub(crate) cpus_per_task: u32,
    pub(crate) memory_mb: u32,
    pub(crate) time_limit: String,
}

struct PipelineNodeContext {
    default_corpus_id: String,
    node_by_id: BTreeMap<String, LocalPipelineDagValidationNodeReport>,
}

pub(crate) fn run_render_all_domain_slurm_submit_manifest(
    args: &parse::BenchLocalRenderAllDomainSlurmSubmitManifestArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let root_path = args
        .root
        .clone()
        .unwrap_or_else(|| benchmark_paths.benchmark_slurm_dry_run_root().join("all-domains"));
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_slurm_dry_run_root().join("all-domains/submit-manifest.json")
    });
    let manifest = render_all_domain_slurm_submit_manifest(&repo_root, root_path, output_path)?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.manifest_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_slurm_submit_manifest(
    repo_root: &Path,
    root_path: PathBuf,
    output_path: PathBuf,
) -> Result<BenchLocalAllDomainSlurmSubmitManifest> {
    let absolute_root = if root_path.is_absolute() { root_path } else { repo_root.join(root_path) };
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(output_path) };
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &absolute_root,
        "all-domain slurm dry-run root",
    )?;
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &absolute_output,
        "all-domain slurm submit manifest output",
    )?;
    fs::create_dir_all(&absolute_root)
        .with_context(|| format!("create {}", absolute_root.display()))?;

    let scripts = render_all_domain_slurm_scripts(repo_root, absolute_root.clone())?;
    let benchmark_outputs = collect_all_domain_output_declaration_rows(repo_root)?
        .into_iter()
        .map(|row| (row.result_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let fixture_sample_scopes = load_fixture_sample_scope_map(repo_root)?;
    let pipeline_contexts = load_pipeline_contexts(repo_root)?;
    let all_job_ids =
        scripts.scripts.iter().map(|entry| entry.job_id_local.clone()).collect::<BTreeSet<_>>();

    let jobs = scripts
        .scripts
        .iter()
        .map(|entry| {
            build_submit_job(
                repo_root,
                &absolute_root,
                entry,
                &benchmark_outputs,
                &fixture_sample_scopes,
                &pipeline_contexts,
                &all_job_ids,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    let dependency_count = jobs.iter().map(|job| job.dependencies.len()).sum::<usize>();
    let manifest = BenchLocalAllDomainSlurmSubmitManifest {
        schema_version: LOCAL_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_SCHEMA_VERSION,
        root_path: path_relative_to_repo(repo_root, &absolute_root),
        manifest_path: path_relative_to_repo(repo_root, &absolute_output),
        run_id: LOCAL_ALL_DOMAIN_SLURM_RUN_ID.to_string(),
        job_count: jobs.len(),
        dependency_count,
        benchmark_job_count: scripts.benchmark_job_count,
        essential_pipeline_job_count: scripts.essential_pipeline_job_count,
        jobs,
    };

    ensure_all_domain_slurm_submit_manifest_contract(&manifest)?;

    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output, &manifest)?;

    Ok(manifest)
}

fn load_pipeline_contexts(repo_root: &Path) -> Result<BTreeMap<String, PipelineNodeContext>> {
    let mut contexts = BTreeMap::new();
    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        let config_path =
            super::local_pipeline_dag::benchmark_local_pipeline_config_path(repo_root, pipeline_id);
        let report_path = repo_root
            .join("benchmarks/readiness/local-ready/pipeline-dag")
            .join(format!("{pipeline_id}.json"));
        let report = validate_pipeline_dag_path(repo_root, &config_path, &report_path)?;
        contexts.insert(
            (*pipeline_id).to_string(),
            PipelineNodeContext {
                default_corpus_id: report.default_corpus_id,
                node_by_id: report
                    .nodes
                    .into_iter()
                    .map(|node| (node.node_id.clone(), node))
                    .collect::<BTreeMap<_, _>>(),
            },
        );
    }
    Ok(contexts)
}

fn build_submit_job(
    repo_root: &Path,
    root_root: &Path,
    entry: &BenchLocalAllDomainSlurmScriptEntry,
    benchmark_outputs: &BTreeMap<String, AllDomainOutputDeclarationRow>,
    fixture_sample_scopes: &BTreeMap<String, String>,
    pipeline_contexts: &BTreeMap<String, PipelineNodeContext>,
    all_job_ids: &BTreeSet<String>,
) -> Result<BenchLocalAllDomainSlurmSubmitJob> {
    validate_script_dependency_header_absence(repo_root, &entry.script_path)?;
    match entry.job_kind.as_str() {
        "benchmark_result" => {
            build_benchmark_submit_job(repo_root, root_root, entry, benchmark_outputs, all_job_ids)
        }
        "essential_pipeline_node" => build_pipeline_submit_job(
            repo_root,
            root_root,
            entry,
            fixture_sample_scopes,
            pipeline_contexts,
            all_job_ids,
        ),
        other => Err(anyhow!(
            "all-domain slurm submit manifest encountered unsupported job kind `{other}`"
        )),
    }
}

fn build_benchmark_submit_job(
    repo_root: &Path,
    root_root: &Path,
    entry: &BenchLocalAllDomainSlurmScriptEntry,
    benchmark_outputs: &BTreeMap<String, AllDomainOutputDeclarationRow>,
    all_job_ids: &BTreeSet<String>,
) -> Result<BenchLocalAllDomainSlurmSubmitJob> {
    if !all_job_ids.contains(&entry.job_id_local) {
        return Err(anyhow!(
            "all-domain slurm submit manifest is missing local job id `{}`",
            entry.job_id_local
        ));
    }
    let result_id = entry.result_id.clone().ok_or_else(|| {
        anyhow!("benchmark job `{}` is missing canonical result_id", entry.job_id_local)
    })?;
    let outputs = benchmark_outputs
        .get(&result_id)
        .ok_or_else(|| anyhow!("missing output declarations for benchmark result `{result_id}`"))?
        .clone();
    let rendered_outputs =
        benchmark_output_paths(repo_root, root_root, entry, &result_id, &outputs)?;
    Ok(BenchLocalAllDomainSlurmSubmitJob {
        job_id_local: entry.job_id_local.clone(),
        domain: entry.domain.clone(),
        stage_id: entry.stage_id.clone(),
        pipeline_id: None,
        node_id: None,
        tool_id: entry.tool_id.clone(),
        corpus_id: entry.corpus_id.clone(),
        asset_profile_id: entry.asset_profile_id.clone(),
        result_id: Some(result_id),
        script_path: entry.script_path.clone(),
        stdout: entry.stdout_path.clone(),
        stderr: entry.stderr_path.clone(),
        outputs: rendered_outputs,
        dependencies: Vec::new(),
        resources: BenchLocalAllDomainSlurmSubmitResources {
            cpus_per_task: entry.cpus_per_task,
            memory_mb: entry.memory_mb,
            time_limit: entry.time_limit.clone(),
        },
    })
}

fn build_pipeline_submit_job(
    repo_root: &Path,
    root_root: &Path,
    entry: &BenchLocalAllDomainSlurmScriptEntry,
    fixture_sample_scopes: &BTreeMap<String, String>,
    pipeline_contexts: &BTreeMap<String, PipelineNodeContext>,
    all_job_ids: &BTreeSet<String>,
) -> Result<BenchLocalAllDomainSlurmSubmitJob> {
    let pipeline_id = entry.pipeline_id.clone().ok_or_else(|| {
        anyhow!("essential pipeline job `{}` is missing pipeline_id", entry.job_id_local)
    })?;
    let node_id = entry.node_id.clone().ok_or_else(|| {
        anyhow!("essential pipeline job `{}` is missing node_id", entry.job_id_local)
    })?;
    let context = pipeline_contexts.get(&pipeline_id).ok_or_else(|| {
        anyhow!("all-domain slurm submit manifest is missing pipeline context for `{pipeline_id}`")
    })?;
    let node = context.node_by_id.get(&node_id).ok_or_else(|| {
        anyhow!(
            "all-domain slurm submit manifest is missing node `{node_id}` in pipeline `{pipeline_id}`"
        )
    })?;

    let expected_node_dependencies = node.depends_on.clone();
    let script_header_dependencies =
        parse_dependency_node_ids_comment(repo_root, Path::new(&entry.script_path))?;
    if expected_node_dependencies != script_header_dependencies {
        return Err(anyhow!(
            "all-domain slurm submit manifest found inconsistent dependency ordering between pipeline DAG and script header for `{}`",
            entry.job_id_local
        ));
    }

    let dependencies = node
        .depends_on
        .iter()
        .map(|dependency_node_id| format!("pipeline:{pipeline_id}:{dependency_node_id}"))
        .collect::<Vec<_>>();
    for dependency in &dependencies {
        if !all_job_ids.contains(dependency) {
            return Err(anyhow!(
                "all-domain slurm submit manifest contains unknown dependency `{dependency}` for `{}`",
                entry.job_id_local
            ));
        }
    }

    let sample_scope = fixture_sample_scopes
        .get(&context.default_corpus_id)
        .cloned()
        .unwrap_or_else(|| "sample-set".to_string());
    let result_root = essential_pipeline_result_root(
        root_root,
        &entry.domain,
        &pipeline_id,
        node_id.as_str(),
        &entry.tool_id,
        &context.default_corpus_id,
        &sample_scope,
    );
    let outputs = node
        .outputs
        .iter()
        .map(|output_id| result_root.join("outputs").join(format!("{output_id}.json")))
        .map(|path| path_relative_to_repo(repo_root, &path))
        .collect::<Vec<_>>();

    Ok(BenchLocalAllDomainSlurmSubmitJob {
        job_id_local: entry.job_id_local.clone(),
        domain: entry.domain.clone(),
        stage_id: entry.stage_id.clone(),
        pipeline_id: Some(pipeline_id),
        node_id: Some(node_id),
        tool_id: entry.tool_id.clone(),
        corpus_id: context.default_corpus_id.clone(),
        asset_profile_id: entry.asset_profile_id.clone(),
        result_id: None,
        script_path: entry.script_path.clone(),
        stdout: entry.stdout_path.clone(),
        stderr: entry.stderr_path.clone(),
        outputs,
        dependencies,
        resources: BenchLocalAllDomainSlurmSubmitResources {
            cpus_per_task: entry.cpus_per_task,
            memory_mb: entry.memory_mb,
            time_limit: entry.time_limit.clone(),
        },
    })
}

fn benchmark_output_paths(
    repo_root: &Path,
    root_root: &Path,
    entry: &BenchLocalAllDomainSlurmScriptEntry,
    result_id: &str,
    row: &AllDomainOutputDeclarationRow,
) -> Result<Vec<String>> {
    let result_root = benchmark_result_root(
        root_root,
        &entry.domain,
        &entry.stage_id,
        &entry.tool_id,
        &entry.corpus_id,
        &entry.asset_profile_id,
        result_id,
    )?;
    let result_root = path_relative_to_repo(repo_root, &result_root);
    let mut outputs = declared_output_ids(row)
        .into_iter()
        .map(|artifact_id| {
            format!(
                "{result_root}/declared-outputs/{}",
                output_relative_path(&artifact_id).to_string_lossy().replace('\\', "/")
            )
        })
        .collect::<Vec<_>>();
    outputs.push(format!("{result_root}/stage-result.json"));
    Ok(outputs)
}

fn validate_script_dependency_header_absence(repo_root: &Path, script_path: &str) -> Result<()> {
    let absolute_script = repo_root.join(script_path);
    let body = fs::read_to_string(&absolute_script)
        .with_context(|| format!("read {}", absolute_script.display()))?;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#SBATCH") && trimmed.contains("--dependency") {
            return Err(anyhow!(
                "all-domain slurm submit manifest requires dependencies to live only in the manifest, found `#SBATCH --dependency` in {script_path}"
            ));
        }
    }
    Ok(())
}

fn parse_dependency_node_ids_comment(repo_root: &Path, script_path: &Path) -> Result<Vec<String>> {
    let absolute_script = repo_root.join(script_path);
    let body = fs::read_to_string(&absolute_script)
        .with_context(|| format!("read {}", absolute_script.display()))?;
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("# dependency_node_ids:") {
            let value = value.trim();
            if value == "none" {
                return Ok(Vec::new());
            }
            return Ok(value
                .split(',')
                .map(str::trim)
                .filter(|segment| !segment.is_empty())
                .map(ToString::to_string)
                .collect());
        }
    }
    Err(anyhow!(
        "all-domain slurm submit manifest requires a dependency header comment in {}",
        script_path.display()
    ))
}

fn ensure_all_domain_slurm_submit_manifest_contract(
    manifest: &BenchLocalAllDomainSlurmSubmitManifest,
) -> Result<()> {
    if manifest.job_count != manifest.benchmark_job_count + manifest.essential_pipeline_job_count {
        return Err(anyhow!(
            "all-domain slurm submit manifest must keep job_count equal to benchmark jobs plus essential pipeline jobs"
        ));
    }
    let unique_job_ids =
        manifest.jobs.iter().map(|job| job.job_id_local.as_str()).collect::<BTreeSet<_>>();
    if unique_job_ids.len() != manifest.jobs.len() {
        return Err(anyhow!(
            "all-domain slurm submit manifest must keep one unique local job id per job"
        ));
    }
    for job in &manifest.jobs {
        if job.outputs.is_empty() {
            return Err(anyhow!(
                "all-domain slurm submit manifest job `{}` must declare at least one output",
                job.job_id_local
            ));
        }
        for dependency in &job.dependencies {
            if !unique_job_ids.contains(dependency.as_str()) {
                return Err(anyhow!(
                    "all-domain slurm submit manifest job `{}` references unknown dependency `{dependency}`",
                    job.job_id_local
                ));
            }
        }
    }
    Ok(())
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).map_or_else(
        |_| path.to_string_lossy().replace('\\', "/"),
        |relative| relative.to_string_lossy().replace('\\', "/"),
    )
}
