use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::commands::benchmark::local_corpus_fixture::{amplicon, bam, damage, edna, fastq};
use crate::commands::benchmark::local_corpus_stage_compatibility::{
    validate_corpus_stage_compatibility_path, DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
};
use crate::commands::benchmark::local_stage_commands::{
    collect_local_stage_command_entries, collect_local_stage_plan_bundles,
    BenchLocalStageCommandEntry,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain,
};

pub(crate) const LOCAL_SLURM_DRY_RUN_RUN_ID: &str = "local-benchmark-dry-run";

#[derive(Debug, Clone)]
pub(crate) struct BenchLocalSlurmRunPaths {
    pub(crate) result_root: PathBuf,
    pub(crate) stdout_path: PathBuf,
    pub(crate) stderr_path: PathBuf,
    pub(crate) stage_result_manifest_path: PathBuf,
}

#[derive(Debug, Clone)]
struct BenchLocalSlurmRunPathInputs {
    command: BenchLocalStageCommandEntry,
    plans: Vec<bijux_dna_stage_contract::StagePlanV1>,
    fixture_id: Option<String>,
}

#[derive(Debug, Clone)]
struct FixtureSampleIndex {
    sample_ids: Vec<String>,
}

pub(crate) fn collect_local_slurm_run_paths(
    repo_root: &Path,
    domain: BenchLocalDomain,
    root_path: &Path,
) -> Result<BTreeMap<String, BenchLocalSlurmRunPaths>> {
    let command_entries = collect_local_stage_command_entries(repo_root, Some(domain))?;
    let stage_plan_bundles = collect_local_stage_plan_bundles(repo_root, Some(domain))?;
    let compatibility = validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    )?;
    let fixture_sample_index = load_fixture_sample_index(repo_root)?;

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

    let inventory = load_local_stage_inventory(repo_root, domain)?;
    let mut paths = BTreeMap::new();
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
        let compatibility = compatibility_index
            .get(&stage_id)
            .ok_or_else(|| anyhow!("missing corpus compatibility entry for `{stage_id}`"))?;
        let run_paths = build_run_paths(
            root_path,
            BenchLocalSlurmRunPathInputs {
                command,
                plans: bundle.plans,
                fixture_id: compatibility.fixture_id.clone(),
            },
            &fixture_sample_index,
        );
        paths.insert(stage_id, run_paths);
    }
    Ok(paths)
}

pub(crate) fn load_fixture_sample_scope_map(repo_root: &Path) -> Result<BTreeMap<String, String>> {
    Ok(load_fixture_sample_index(repo_root)?
        .into_iter()
        .map(|(fixture_id, index)| (fixture_id, sample_scope_from_sample_ids(&index.sample_ids)))
        .collect())
}

fn build_run_paths(
    root_path: &Path,
    job: BenchLocalSlurmRunPathInputs,
    fixture_sample_index: &BTreeMap<String, FixtureSampleIndex>,
) -> BenchLocalSlurmRunPaths {
    let mut sample_ids = collect_plan_sample_ids(&job.plans, &job.command.tool_id);
    if sample_ids.is_empty() {
        if let Some(fixture_id) = &job.fixture_id {
            if let Some(index) = fixture_sample_index.get(fixture_id) {
                sample_ids = index.sample_ids.clone();
            }
        }
    }
    let sample_scope = sample_scope_from_sample_ids(&sample_ids);
    let corpus_scope = job.fixture_id.unwrap_or_else(|| "planner-only".to_string());

    let result_root = root_path
        .join("runs")
        .join(LOCAL_SLURM_DRY_RUN_RUN_ID)
        .join(corpus_scope)
        .join(&job.command.stage_id)
        .join(sample_scope)
        .join(&job.command.tool_id);
    let stdout_path = result_root.join("stdout.log");
    let stderr_path = result_root.join("stderr.log");
    let stage_result_manifest_path = result_root.join("stage-result.json");

    BenchLocalSlurmRunPaths { result_root, stdout_path, stderr_path, stage_result_manifest_path }
}

pub(crate) fn sample_scope_from_sample_ids(sample_ids: &[String]) -> String {
    if sample_ids.len() == 1 {
        sample_ids[0].clone()
    } else {
        "sample-set".to_string()
    }
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
            FixtureSchema::Fastq => {
                fastq::validate_fastq_corpus_fixture_manifest_path(repo_root, &manifest_path)?
                    .samples
                    .into_iter()
                    .map(|sample| sample.sample_id)
                    .collect()
            }
            FixtureSchema::Bam => {
                bam::validate_bam_corpus_fixture_manifest_path(repo_root, &manifest_path)?
                    .samples
                    .into_iter()
                    .map(|sample| sample.sample_id)
                    .collect()
            }
            FixtureSchema::Edna => {
                edna::validate_edna_corpus_fixture_manifest_path(repo_root, &manifest_path)?
                    .samples
                    .into_iter()
                    .map(|sample| sample.sample_id)
                    .collect()
            }
            FixtureSchema::Amplicon => {
                amplicon::validate_amplicon_corpus_fixture_manifest_path(repo_root, &manifest_path)?
                    .samples
                    .into_iter()
                    .map(|sample| sample.sample_id)
                    .collect()
            }
            FixtureSchema::Damage => {
                vec![
                    damage::validate_bam_damage_fixture_manifest_path(repo_root, &manifest_path)?
                        .sample_id,
                ]
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
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let parsed =
        raw.parse::<toml::Table>().with_context(|| format!("parse {}", manifest_path.display()))?;
    let schema_version =
        parsed.get("schema_version").and_then(toml::Value::as_str).ok_or_else(|| {
            anyhow!("fixture manifest is missing `schema_version`: {}", manifest_path.display())
        })?;
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
