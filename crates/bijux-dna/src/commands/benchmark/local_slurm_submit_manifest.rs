use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_corpus_fixture::{amplicon, bam, damage, edna, fastq};
use crate::commands::benchmark::local_corpus_stage_compatibility::{
    validate_corpus_stage_compatibility_path, DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
};
use crate::commands::benchmark::local_slurm_dry_run::{
    render_local_slurm_scripts, BenchLocalSlurmScriptEntry,
};
use crate::commands::benchmark::local_stage_commands::{
    collect_local_stage_command_entries, collect_local_stage_plan_bundles,
    BenchLocalStageArtifactEntry, BenchLocalStageCommandEntry,
};
use crate::commands::benchmark::local_stage_inventory::BenchLocalDomain;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.local_slurm_submit_manifest.v1";
const DEFAULT_SLURM_DRY_RUN_ROOT: &str = "target/slurm-dry-run";
const DEFAULT_SLURM_SUBMIT_MANIFEST_PATH: &str = "target/slurm-dry-run/submit-manifest.json";
const LOCAL_SLURM_DRY_RUN_RUN_ID: &str = "local-benchmark-dry-run";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalSlurmSubmitManifest {
    pub(crate) schema_version: &'static str,
    pub(crate) root_path: String,
    pub(crate) manifest_path: String,
    pub(crate) run_id: String,
    pub(crate) job_count: usize,
    pub(crate) dependency_count: usize,
    pub(crate) jobs: Vec<BenchLocalSlurmSubmitJob>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalSlurmSubmitJob {
    pub(crate) job_name: String,
    pub(crate) domain: String,
    pub(crate) stage_id: Option<String>,
    pub(crate) pipeline_id: Option<String>,
    pub(crate) corpus_id: Option<String>,
    pub(crate) sample_id: Option<String>,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) tool_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) resources: BenchLocalSlurmSubmitResources,
    pub(crate) script_path: String,
    pub(crate) logs: BenchLocalSlurmSubmitLogs,
    pub(crate) result_root: String,
    pub(crate) outputs: Vec<BenchLocalStageArtifactEntry>,
    pub(crate) dependencies: Vec<String>,
    pub(crate) compatibility_kind: String,
    pub(crate) compatibility_note: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalSlurmSubmitResources {
    pub(crate) cpus_per_task: u32,
    pub(crate) memory_mb: u32,
    pub(crate) time_limit: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalSlurmSubmitLogs {
    pub(crate) stdout_path: String,
    pub(crate) stderr_path: String,
}

#[derive(Debug, Clone)]
struct BenchLocalSlurmSubmitInputs {
    domain: BenchLocalDomain,
    command: BenchLocalStageCommandEntry,
    plans: Vec<bijux_dna_stage_contract::StagePlanV1>,
    script: BenchLocalSlurmScriptEntry,
    compatibility_kind: String,
    compatibility_note: String,
    fixture_id: Option<String>,
}

#[derive(Debug, Clone)]
struct FixtureSampleIndex {
    sample_ids: Vec<String>,
}

pub(crate) fn run_render_slurm_submit_manifest(
    args: &parse::BenchLocalRenderSlurmSubmitManifestArgs,
) -> Result<()> {
    validate_slurm_submit_manifest_feature_gate()?;

    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let root_path = args.root.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT));
    let output_path =
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_SLURM_SUBMIT_MANIFEST_PATH));
    let manifest = render_slurm_submit_manifest(&repo_root, root_path, output_path)?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.manifest_path);
    }
    Ok(())
}

pub(crate) fn render_slurm_submit_manifest(
    repo_root: &Path,
    root_path: PathBuf,
    output_path: PathBuf,
) -> Result<BenchLocalSlurmSubmitManifest> {
    validate_slurm_submit_manifest_feature_gate()?;

    let absolute_root = if root_path.is_absolute() { root_path } else { repo_root.join(root_path) };
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(output_path) };
    fs::create_dir_all(&absolute_root).with_context(|| format!("create {}", absolute_root.display()))?;

    let fastq_scripts = render_local_slurm_scripts(
        repo_root,
        BenchLocalDomain::Fastq,
        absolute_root.join(BenchLocalDomain::Fastq.as_str()),
    )?;
    let bam_scripts = render_local_slurm_scripts(
        repo_root,
        BenchLocalDomain::Bam,
        absolute_root.join(BenchLocalDomain::Bam.as_str()),
    )?;

    let command_entries = collect_local_stage_command_entries(repo_root, None)?;
    let stage_plan_bundles = collect_local_stage_plan_bundles(repo_root, None)?;
    let compatibility = validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    )?;
    let fixture_sample_index = load_fixture_sample_index(repo_root)?;

    let script_index = fastq_scripts
        .scripts
        .into_iter()
        .chain(bam_scripts.scripts)
        .map(|script| (script.stage_id.clone(), script))
        .collect::<BTreeMap<_, _>>();
    let command_index = command_entries
        .into_iter()
        .map(|entry| (entry.stage_id.clone(), entry))
        .collect::<BTreeMap<_, _>>();
    let plan_index = stage_plan_bundles
        .into_iter()
        .map(|bundle| (bundle.stage_id.clone(), bundle))
        .collect::<BTreeMap<_, _>>();
    let compatibility_index = compatibility
        .stages
        .into_iter()
        .map(|stage| (stage.stage_id.clone(), stage))
        .collect::<BTreeMap<_, _>>();

    let mut jobs = Vec::new();
    for domain in [BenchLocalDomain::Fastq, BenchLocalDomain::Bam] {
        let inventory = crate::commands::benchmark::local_stage_inventory::load_local_stage_inventory(
            repo_root,
            domain,
        )?;
        for stage in inventory.stages {
            let stage_id = stage.stage_id;
            let command = command_index
                .get(&stage_id)
                .cloned()
                .ok_or_else(|| anyhow!("missing stage command entry for `{stage_id}`"))?;
            let bundle = plan_index
                .get(&stage_id)
                .cloned()
                .ok_or_else(|| anyhow!("missing stage plan bundle for `{stage_id}`"))?;
            let script = script_index
                .get(&stage_id)
                .cloned()
                .ok_or_else(|| anyhow!("missing slurm script entry for `{stage_id}`"))?;
            let compatibility = compatibility_index
                .get(&stage_id)
                .ok_or_else(|| anyhow!("missing corpus compatibility entry for `{stage_id}`"))?;

            jobs.push(BenchLocalSlurmSubmitInputs {
                domain,
                command,
                plans: bundle.plans,
                script,
                compatibility_kind: compatibility.compatibility_kind.clone(),
                compatibility_note: compatibility.compatibility_note.clone(),
                fixture_id: compatibility.fixture_id.clone(),
            });
        }
    }

    let dependencies = derive_job_dependencies(&jobs);
    let rendered_jobs = jobs
        .into_iter()
        .map(|job| {
            let stage_id = job.script.stage_id.clone();
            build_submit_job(
                repo_root,
                &absolute_root,
                job,
                dependencies.get(stage_id.as_str()).cloned().unwrap_or_default(),
                &fixture_sample_index,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    let dependency_count = rendered_jobs.iter().map(|job| job.dependencies.len()).sum();
    let manifest = BenchLocalSlurmSubmitManifest {
        schema_version: LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_VERSION,
        root_path: path_relative_to_repo(repo_root, &absolute_root),
        manifest_path: path_relative_to_repo(repo_root, &absolute_output),
        run_id: LOCAL_SLURM_DRY_RUN_RUN_ID.to_string(),
        job_count: rendered_jobs.len(),
        dependency_count,
        jobs: rendered_jobs,
    };

    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output, &manifest)?;

    Ok(manifest)
}

#[cfg(feature = "bam_downstream")]
fn validate_slurm_submit_manifest_feature_gate() -> Result<()> {
    Ok(())
}

#[cfg(not(feature = "bam_downstream"))]
fn validate_slurm_submit_manifest_feature_gate() -> Result<()> {
    Err(anyhow!(
        "slurm submit manifest rendering requires the `bam_downstream` feature; rerun with `cargo run -p bijux-dna --features bam_downstream -- bench local render-slurm-submit-manifest`"
    ))
}

fn build_submit_job(
    repo_root: &Path,
    root_path: &Path,
    job: BenchLocalSlurmSubmitInputs,
    dependencies: Vec<String>,
    fixture_sample_index: &BTreeMap<String, FixtureSampleIndex>,
) -> Result<BenchLocalSlurmSubmitJob> {
    let mut sample_ids = collect_plan_sample_ids(&job.plans, &job.command.tool_id);
    if sample_ids.is_empty() {
        if let Some(fixture_id) = &job.fixture_id {
            if let Some(index) = fixture_sample_index.get(fixture_id) {
                if index.sample_ids.len() == 1 {
                    sample_ids = index.sample_ids.clone();
                }
            }
        }
    }
    let sample_id = (sample_ids.len() == 1).then(|| sample_ids[0].clone());
    let corpus_id = job.fixture_id.clone();
    let job_name = slurm_job_name(&job.command.stage_id);
    let scope_root = slurm_job_scope_root(root_path, &job.command, corpus_id.as_deref(), sample_id.as_deref());
    let stdout_path = scope_root.join("stdout.log");
    let stderr_path = scope_root.join("stderr.log");

    Ok(BenchLocalSlurmSubmitJob {
        job_name,
        domain: job.domain.as_str().to_string(),
        stage_id: Some(job.command.stage_id.clone()),
        pipeline_id: None,
        corpus_id,
        sample_id,
        sample_ids,
        tool_id: job.command.tool_id.clone(),
        readiness_kind: job.command.readiness_kind.as_str().to_string(),
        resources: BenchLocalSlurmSubmitResources {
            cpus_per_task: job.script.cpus_per_task,
            memory_mb: job.script.memory_mb,
            time_limit: job.script.time_limit.clone(),
        },
        script_path: job.script.script_path,
        logs: BenchLocalSlurmSubmitLogs {
            stdout_path: path_relative_to_repo(repo_root, &stdout_path),
            stderr_path: path_relative_to_repo(repo_root, &stderr_path),
        },
        result_root: path_relative_to_repo(repo_root, &scope_root),
        outputs: job.command.outputs,
        dependencies,
        compatibility_kind: job.compatibility_kind,
        compatibility_note: job.compatibility_note,
    })
}

fn derive_job_dependencies(
    jobs: &[BenchLocalSlurmSubmitInputs],
) -> BTreeMap<String, Vec<String>> {
    let mut output_to_stage = BTreeMap::<String, String>::new();
    for job in jobs {
        for output in &job.command.outputs {
            output_to_stage.insert(output.path.clone(), job.command.stage_id.clone());
        }
    }

    jobs.iter()
        .map(|job| {
            let dependencies = job
                .command
                .inputs
                .iter()
                .filter_map(|input| output_to_stage.get(&input.path))
                .filter(|dependency_stage_id| *dependency_stage_id != &job.command.stage_id)
                .map(|dependency_stage_id| slurm_job_name(dependency_stage_id))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            (job.command.stage_id.clone(), dependencies)
        })
        .collect()
}

fn collect_plan_sample_ids(
    plans: &[bijux_dna_stage_contract::StagePlanV1],
    tool_id: &str,
) -> Vec<String> {
    let mut sample_ids = plans
        .iter()
        .filter_map(|plan| sample_id_from_out_dir(&plan.out_dir, tool_id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    sample_ids.sort();
    sample_ids
}

fn sample_id_from_out_dir(out_dir: &Path, tool_id: &str) -> Option<String> {
    let tool_segment = out_dir.file_name()?.to_str()?;
    if tool_segment != tool_id {
        return None;
    }
    out_dir.parent()?.file_name()?.to_str().map(ToString::to_string)
}

fn load_fixture_sample_index(repo_root: &Path) -> Result<BTreeMap<String, FixtureSampleIndex>> {
    let matrix = validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    )?;
    let mut index = BTreeMap::new();
    for fixture in matrix.fixtures {
        let manifest_path = repo_root.join(&fixture.fixture_manifest);
        let sample_ids = match detect_fixture_schema(&manifest_path)? {
            FixtureSchema::Fastq => fastq::validate_fastq_corpus_fixture_manifest_path(repo_root, &manifest_path)?
                .samples
                .into_iter()
                .map(|sample| sample.sample_id)
                .collect(),
            FixtureSchema::Bam => bam::validate_bam_corpus_fixture_manifest_path(repo_root, &manifest_path)?
                .samples
                .into_iter()
                .map(|sample| sample.sample_id)
                .collect(),
            FixtureSchema::Edna => edna::validate_edna_corpus_fixture_manifest_path(repo_root, &manifest_path)?
                .samples
                .into_iter()
                .map(|sample| sample.sample_id)
                .collect(),
            FixtureSchema::Amplicon => {
                amplicon::validate_amplicon_corpus_fixture_manifest_path(repo_root, &manifest_path)?
                    .samples
                    .into_iter()
                    .map(|sample| sample.sample_id)
                    .collect()
            }
            FixtureSchema::Damage => {
                vec![damage::validate_bam_damage_fixture_manifest_path(repo_root, &manifest_path)?
                    .sample_id]
            }
        };
        index.insert(fixture.fixture_id, FixtureSampleIndex { sample_ids });
    }
    Ok(index)
}

enum FixtureSchema {
    Fastq,
    Bam,
    Edna,
    Amplicon,
    Damage,
}

fn detect_fixture_schema(manifest_path: &Path) -> Result<FixtureSchema> {
    let raw =
        fs::read_to_string(manifest_path).with_context(|| format!("read {}", manifest_path.display()))?;
    let parsed = raw
        .parse::<toml::Table>()
        .with_context(|| format!("parse {}", manifest_path.display()))?;
    let schema_version = parsed
        .get("schema_version")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| anyhow!("fixture manifest is missing `schema_version`: {}", manifest_path.display()))?;
    match schema_version {
        fastq::FASTQ_CORPUS_FIXTURE_SCHEMA_VERSION => Ok(FixtureSchema::Fastq),
        bam::BAM_CORPUS_FIXTURE_SCHEMA_VERSION => Ok(FixtureSchema::Bam),
        edna::EDNA_CORPUS_FIXTURE_SCHEMA_VERSION => Ok(FixtureSchema::Edna),
        amplicon::AMPLICON_CORPUS_FIXTURE_SCHEMA_VERSION => Ok(FixtureSchema::Amplicon),
        damage::BAM_DAMAGE_FIXTURE_SCHEMA_VERSION => Ok(FixtureSchema::Damage),
        other => Err(anyhow!(
            "unsupported corpus fixture schema `{other}` in {}",
            manifest_path.display()
        )),
    }
}

fn slurm_job_scope_root(
    root_path: &Path,
    command: &BenchLocalStageCommandEntry,
    corpus_id: Option<&str>,
    sample_id: Option<&str>,
) -> PathBuf {
    root_path
        .join("runs")
        .join(LOCAL_SLURM_DRY_RUN_RUN_ID)
        .join(corpus_id.unwrap_or("planner-only"))
        .join(&command.stage_id)
        .join(sample_id.unwrap_or("sample-set"))
        .join(&command.tool_id)
}

fn slurm_job_name(stage_id: &str) -> String {
    stage_id.replace('.', "-")
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}
