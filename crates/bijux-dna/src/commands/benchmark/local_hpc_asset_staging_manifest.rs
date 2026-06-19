use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::local_stage_commands::local_stage_plans;
use super::path_resolution::{
    ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_HPC_ASSET_STAGING_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.local_hpc_asset_staging_manifest.v1";
pub(crate) const DEFAULT_HPC_ASSET_STAGING_MANIFEST_PATH: &str =
    "runs/bench/hpc-dry-run/asset-staging-manifest.json";
const DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_ARGV_PATH: &str =
    "benchmarks/readiness/rendered-commands-all-domains.argv.jsonl";

const SOURCE_ROOTS: [&str; 3] = ["assets/", "benchmarks/", "runs/"];
const OUTPUT_FLAG_ARGS: [&str; 12] = [
    "-o",
    "-O",
    "--out",
    "--output",
    "--output-dir",
    "--output-root",
    "--output-prefix",
    "--prefix",
    "--stdout",
    "--stderr",
    "--report",
    "--summary",
];

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalHpcAssetStagingManifest {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) staging_root: String,
    pub(crate) selected_job_count: usize,
    pub(crate) staged_input_count: usize,
    pub(crate) unique_source_path_count: usize,
    pub(crate) jobs: Vec<LocalHpcAssetStagingJob>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalHpcAssetStagingJob {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) staged_inputs: Vec<LocalHpcAssetStagingInput>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalHpcAssetStagingInput {
    pub(crate) artifact_id: Option<String>,
    pub(crate) artifact_role: Option<String>,
    pub(crate) source_kind: String,
    pub(crate) source_path: String,
    pub(crate) staged_path: String,
    pub(crate) checksum_sha256: String,
    pub(crate) size_bytes: u64,
    pub(crate) member_count: usize,
    pub(crate) consumer_result_id: String,
}

#[derive(Debug, Clone)]
struct LocalStageInputHint {
    artifact_id: String,
    artifact_role: String,
    source_path: String,
}

#[derive(Debug, Clone)]
struct ResolvedSourceAsset {
    source_kind: String,
    source_path: String,
    checksum_sha256: String,
    size_bytes: u64,
    member_count: usize,
}

#[derive(Debug, Default)]
struct StepPathUsage {
    consumed: BTreeSet<String>,
    produced: BTreeSet<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RenderedCommandArgvStepFileRow {
    argv: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RenderedCommandArgvFileRow {
    result_id: String,
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
    command_steps: Vec<RenderedCommandArgvStepFileRow>,
}

pub(crate) fn run_render_hpc_asset_staging_manifest(
    args: &parse::BenchLocalRenderHpcAssetStagingManifestArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_dry_run_root().join("asset-staging-manifest.json")
    });
    let manifest = render_hpc_asset_staging_manifest(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.output_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_asset_staging_manifest(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcAssetStagingManifest> {
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &absolute_output,
        "HPC asset staging manifest output",
    )?;
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let staging_root = benchmark_paths.benchmark_hpc_dry_run_root().join("staged");
    fs::create_dir_all(&staging_root).with_context(|| format!("create {}", staging_root.display()))?;
    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let command_rows = load_rendered_command_argv_rows(repo_root)?;
    let materialized_stage_ids = command_rows
        .iter()
        .flat_map(|row| row.command_steps.iter())
        .filter_map(|step| materialize_stage_id(&step.argv))
        .collect::<BTreeSet<_>>();
    let stage_input_hints = collect_local_stage_input_hints(repo_root, &materialized_stage_ids)?;
    let mut jobs = Vec::with_capacity(command_rows.len());
    let mut unique_source_paths = BTreeSet::new();
    let mut staged_input_count = 0usize;

    for row in command_rows {
        let staged_inputs = collect_job_staged_inputs(repo_root, &staging_root, &row, &stage_input_hints)
            .with_context(|| {
                format!(
                    "collect staged inputs for `{}` / `{}` / `{}`",
                    row.result_id, row.stage_id, row.tool_id
                )
            })?;
        staged_input_count += staged_inputs.len();
        unique_source_paths.extend(staged_inputs.iter().map(|entry| entry.source_path.clone()));
        jobs.push(LocalHpcAssetStagingJob {
            result_id: row.result_id,
            domain: row.domain,
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            corpus_id: row.corpus_id,
            asset_profile_id: row.asset_profile_id,
            staged_inputs,
        });
    }

    jobs.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let manifest = LocalHpcAssetStagingManifest {
        schema_version: LOCAL_HPC_ASSET_STAGING_MANIFEST_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output),
        staging_root: path_relative_to_repo(repo_root, &staging_root),
        selected_job_count: jobs.len(),
        staged_input_count,
        unique_source_path_count: unique_source_paths.len(),
        jobs,
    };
    ensure_manifest_contract(&manifest)?;
    bijux_dna_infra::atomic_write_json(&absolute_output, &manifest)?;
    Ok(manifest)
}

fn load_rendered_command_argv_rows(repo_root: &Path) -> Result<Vec<RenderedCommandArgvFileRow>> {
    let argv_path = repo_root.join(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_ARGV_PATH);
    let body =
        fs::read_to_string(&argv_path).with_context(|| format!("read {}", argv_path.display()))?;
    let mut rows = body
        .lines()
        .enumerate()
        .map(|(index, line)| {
            serde_json::from_str::<RenderedCommandArgvFileRow>(line).with_context(|| {
                format!(
                    "parse all-domain rendered command argv row {} from {}",
                    index + 1,
                    argv_path.display()
                )
            })
        })
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });
    Ok(rows)
}

fn collect_local_stage_input_hints(
    repo_root: &Path,
    stage_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, Vec<LocalStageInputHint>>> {
    let mut hints = BTreeMap::<String, Vec<LocalStageInputHint>>::new();
    for stage_id in stage_ids {
        let mut stage_hints = local_stage_plans(repo_root, stage_id)?
            .into_iter()
            .flat_map(|plan| plan.io.inputs.into_iter())
            .map(|input| LocalStageInputHint {
                artifact_id: input.name.as_str().to_string(),
                artifact_role: input.role.as_str().to_string(),
                source_path: path_relative_to_repo(repo_root, &input.path),
            })
            .collect::<Vec<_>>();
        stage_hints.sort_by(|left, right| {
            left.source_path
                .cmp(&right.source_path)
                .then_with(|| left.artifact_id.cmp(&right.artifact_id))
                .then_with(|| left.artifact_role.cmp(&right.artifact_role))
        });
        stage_hints.dedup_by(|left, right| {
            left.source_path == right.source_path
                && left.artifact_id == right.artifact_id
                && left.artifact_role == right.artifact_role
        });
        hints.insert(stage_id.clone(), stage_hints);
    }
    Ok(hints)
}

fn collect_job_staged_inputs(
    repo_root: &Path,
    staging_root: &Path,
    row: &RenderedCommandArgvFileRow,
    stage_input_hints: &BTreeMap<String, Vec<LocalStageInputHint>>,
) -> Result<Vec<LocalHpcAssetStagingInput>> {
    let mut staged_by_source = BTreeMap::<String, LocalHpcAssetStagingInput>::new();
    let mut produced_paths = BTreeSet::<String>::new();

    for step in &row.command_steps {
        if let Some(stage_id) = materialize_stage_id(&step.argv) {
            let hints = stage_input_hints.get(&stage_id).ok_or_else(|| {
                anyhow!("missing local stage input hints for materialized stage `{stage_id}`")
            })?;
            for hint in hints {
                let resolved = resolve_source_asset(repo_root, &hint.source_path)?.ok_or_else(|| {
                    anyhow!(
                        "materialized stage `{stage_id}` depends on unresolved source `{}`",
                        hint.source_path
                    )
                })?;
                merge_staged_input(
                    &mut staged_by_source,
                    row,
                    repo_root,
                    staging_root,
                    resolved,
                    Some(hint.artifact_id.clone()),
                    Some(hint.artifact_role.clone()),
                );
            }
            continue;
        }

        let usage = collect_step_path_usage(&step.argv)?;
        for source_path in usage.consumed.difference(&usage.produced) {
            if produced_paths.contains(source_path.as_str()) {
                continue;
            }
            if let Some(resolved) = resolve_source_asset(repo_root, source_path)? {
                merge_staged_input(
                    &mut staged_by_source,
                    row,
                    repo_root,
                    staging_root,
                    resolved,
                    None,
                    None,
                );
            }
        }
        produced_paths.extend(usage.produced);
    }

    if staged_by_source.is_empty() {
        return Err(anyhow!(
            "HPC asset staging manifest found no source inputs for `{}`",
            row.result_id
        ));
    }

    Ok(staged_by_source.into_values().collect())
}

fn merge_staged_input(
    staged_by_source: &mut BTreeMap<String, LocalHpcAssetStagingInput>,
    row: &RenderedCommandArgvFileRow,
    repo_root: &Path,
    staging_root: &Path,
    resolved: ResolvedSourceAsset,
    artifact_id: Option<String>,
    artifact_role: Option<String>,
) {
    let staged_path = path_relative_to_repo(repo_root, &staging_root.join(&resolved.source_path));
    staged_by_source
        .entry(resolved.source_path.clone())
        .and_modify(|entry| {
            if entry.artifact_id.is_none() {
                entry.artifact_id = artifact_id.clone();
            }
            if entry.artifact_role.is_none() {
                entry.artifact_role = artifact_role.clone();
            }
        })
        .or_insert(LocalHpcAssetStagingInput {
            artifact_id,
            artifact_role,
            source_kind: resolved.source_kind,
            source_path: resolved.source_path,
            staged_path,
            checksum_sha256: resolved.checksum_sha256,
            size_bytes: resolved.size_bytes,
            member_count: resolved.member_count,
            consumer_result_id: row.result_id.clone(),
        });
}

fn materialize_stage_id(argv: &[String]) -> Option<String> {
    if !argv.iter().any(|arg| arg == "materialize-stage") {
        return None;
    }
    argv.windows(2)
        .find(|window| window[0] == "--stage-id")
        .map(|window| window[1].clone())
}

fn collect_step_path_usage(argv: &[String]) -> Result<StepPathUsage> {
    if argv.len() >= 3 && argv[0] == "/bin/sh" && argv[1] == "-c" {
        return collect_shell_step_path_usage(&argv[2]);
    }
    Ok(collect_tokenized_step_path_usage(argv))
}

fn collect_tokenized_step_path_usage(argv: &[String]) -> StepPathUsage {
    let mut usage = StepPathUsage::default();
    let mut previous_flag: Option<&str> = None;

    for arg in argv {
        if let Some(path) = candidate_repo_path(arg) {
            if previous_flag.is_some_and(is_output_flag) {
                usage.produced.insert(path.to_string());
            } else {
                usage.consumed.insert(path.to_string());
            }
        }

        previous_flag = if arg.starts_with('-') { Some(arg.as_str()) } else { None };
    }

    usage
}

fn collect_shell_step_path_usage(shell_command: &str) -> Result<StepPathUsage> {
    let body = strip_heredoc_bodies(shell_command);
    let matcher = Regex::new(r"(?:assets|benchmarks|runs)/[A-Za-z0-9_./@+-]+")
        .context("compile HPC asset path matcher")?;
    let mut usage = StepPathUsage::default();

    for matched in matcher.find_iter(&body) {
        let source_path = matched.as_str().trim_end_matches('.');
        if !is_repo_source_path(source_path) {
            continue;
        }
        if is_output_context(&body, matched.start()) {
            usage.produced.insert(source_path.to_string());
        } else {
            usage.consumed.insert(source_path.to_string());
        }
    }

    Ok(usage)
}

fn strip_heredoc_bodies(shell_command: &str) -> String {
    let mut stripped = String::new();
    let mut active_delimiter: Option<String> = None;

    for line in shell_command.lines() {
        if let Some(delimiter) = active_delimiter.as_ref() {
            if line.trim() == delimiter {
                active_delimiter = None;
            }
            continue;
        }

        stripped.push_str(line);
        stripped.push('\n');
        if let Some(delimiter) = heredoc_delimiter(line) {
            active_delimiter = Some(delimiter);
        }
    }

    stripped
}

fn heredoc_delimiter(line: &str) -> Option<String> {
    let marker = line.find("<<")?;
    let remainder = line.get(marker + 2..)?.trim_start_matches('-').trim_start();
    let mut delimiter = String::new();
    let mut started = false;

    for ch in remainder.chars() {
        if !started && matches!(ch, '\'' | '"') {
            started = true;
            continue;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' {
            delimiter.push(ch);
            started = true;
            continue;
        }
        break;
    }

    if delimiter.is_empty() { None } else { Some(delimiter) }
}

fn is_output_context(body: &str, start: usize) -> bool {
    let prefix = body.get(..start).unwrap_or(body).trim_end();
    if prefix.ends_with('>') || prefix.ends_with("1>") || prefix.ends_with("2>") {
        return true;
    }

    last_shell_token(prefix).is_some_and(is_output_flag)
}

fn last_shell_token(prefix: &str) -> Option<&str> {
    prefix
        .rsplit(|ch: char| ch.is_whitespace())
        .find(|segment| !segment.is_empty())
}

fn is_output_flag(flag: &str) -> bool {
    OUTPUT_FLAG_ARGS.contains(&flag)
}

fn candidate_repo_path(value: &str) -> Option<&str> {
    if is_repo_source_path(value) { Some(value) } else { None }
}

fn is_repo_source_path(value: &str) -> bool {
    SOURCE_ROOTS.iter().any(|root| value.starts_with(root))
}

fn resolve_source_asset(repo_root: &Path, source_path: &str) -> Result<Option<ResolvedSourceAsset>> {
    let absolute_source = repo_root.join(source_path);
    if absolute_source.is_file() {
        let checksum_sha256 = bijux_dna_infra::hash_file_sha256(&absolute_source)
            .map_err(|err| anyhow!(err.to_string()))?;
        let size_bytes = absolute_source
            .metadata()
            .with_context(|| format!("read metadata for {}", absolute_source.display()))?
            .len();
        return Ok(Some(ResolvedSourceAsset {
            source_kind: "file".to_string(),
            source_path: source_path.to_string(),
            checksum_sha256,
            size_bytes,
            member_count: 1,
        }));
    }
    if absolute_source.is_dir() {
        let (checksum_sha256, size_bytes, member_count) =
            digest_directory_members(&absolute_source, &absolute_source)?;
        return Ok(Some(ResolvedSourceAsset {
            source_kind: "directory".to_string(),
            source_path: source_path.to_string(),
            checksum_sha256,
            size_bytes,
            member_count,
        }));
    }

    resolve_prefix_bundle(&absolute_source, source_path)
}

fn resolve_prefix_bundle(
    absolute_source: &Path,
    source_path: &str,
) -> Result<Option<ResolvedSourceAsset>> {
    let Some(parent) = absolute_source.parent() else {
        return Ok(None);
    };
    let Some(prefix) = absolute_source.file_name().and_then(|value| value.to_str()) else {
        return Ok(None);
    };

    let mut members = fs::read_dir(parent)
        .with_context(|| format!("read {}", parent.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with(&format!("{prefix}.")))
        })
        .collect::<Vec<_>>();
    members.sort();

    if members.is_empty() {
        return Ok(None);
    }

    let (checksum_sha256, size_bytes, member_count) = digest_explicit_members(&members, parent)?;
    Ok(Some(ResolvedSourceAsset {
        source_kind: "prefix_bundle".to_string(),
        source_path: source_path.to_string(),
        checksum_sha256,
        size_bytes,
        member_count,
    }))
}

fn digest_directory_members(root: &Path, path: &Path) -> Result<(String, u64, usize)> {
    let mut members = Vec::new();
    collect_directory_files(path, &mut members)?;
    members.sort();
    digest_explicit_members(&members, root)
}

fn collect_directory_files(path: &Path, members: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
        let entry = entry.with_context(|| format!("read entry in {}", path.display()))?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_directory_files(&entry_path, members)?;
        } else if entry_path.is_file() {
            members.push(entry_path);
        }
    }
    Ok(())
}

fn digest_explicit_members(members: &[PathBuf], relative_root: &Path) -> Result<(String, u64, usize)> {
    let mut hasher = Sha256::new();
    let mut size_bytes = 0u64;

    for member in members {
        let digest = bijux_dna_infra::hash_file_sha256(member).map_err(|err| anyhow!(err.to_string()))?;
        let metadata =
            member.metadata().with_context(|| format!("read metadata for {}", member.display()))?;
        let relative_member = member
            .strip_prefix(relative_root)
            .unwrap_or(member)
            .to_string_lossy()
            .replace('\\', "/");
        size_bytes += metadata.len();
        hasher.update(relative_member.as_bytes());
        hasher.update([0]);
        hasher.update(digest.as_bytes());
        hasher.update([0]);
        hasher.update(metadata.len().to_string().as_bytes());
        hasher.update([b'\n']);
    }

    Ok((hex_sha256_digest(hasher), size_bytes, members.len()))
}

fn hex_sha256_digest(hasher: Sha256) -> String {
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

fn ensure_manifest_contract(manifest: &LocalHpcAssetStagingManifest) -> Result<()> {
    if manifest.selected_job_count != manifest.jobs.len() {
        return Err(anyhow!(
            "HPC asset staging manifest must keep selected_job_count aligned with jobs"
        ));
    }
    if manifest.staged_input_count
        != manifest.jobs.iter().map(|job| job.staged_inputs.len()).sum::<usize>()
    {
        return Err(anyhow!(
            "HPC asset staging manifest must keep staged_input_count aligned with staged inputs"
        ));
    }

    let mut unique_sources = BTreeSet::new();
    for job in &manifest.jobs {
        if job.staged_inputs.is_empty() {
            return Err(anyhow!(
                "HPC asset staging manifest job `{}` must keep at least one staged input",
                job.result_id
            ));
        }
        for input in &job.staged_inputs {
            if input.consumer_result_id != job.result_id {
                return Err(anyhow!(
                    "HPC asset staging input `{}` drifted from consumer result `{}`",
                    input.source_path,
                    job.result_id
                ));
            }
            if input.source_path == input.staged_path {
                return Err(anyhow!(
                    "HPC asset staging input `{}` must resolve to a distinct staged path",
                    input.source_path
                ));
            }
            unique_sources.insert(input.source_path.as_str());
        }
    }

    if manifest.unique_source_path_count != unique_sources.len() {
        return Err(anyhow!(
            "HPC asset staging manifest must keep unique_source_path_count aligned with source paths"
        ));
    }

    Ok(())
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).map_or_else(
        |_| path.to_string_lossy().replace('\\', "/"),
        |relative| relative.to_string_lossy().replace('\\', "/"),
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        load_rendered_command_argv_rows, render_hpc_asset_staging_manifest,
        DEFAULT_HPC_ASSET_STAGING_MANIFEST_PATH, LOCAL_HPC_ASSET_STAGING_MANIFEST_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn rendered_hpc_asset_staging_manifest_covers_all_domain_jobs() {
        let root = repo_root();
        let manifest = render_hpc_asset_staging_manifest(
            &root,
            PathBuf::from(DEFAULT_HPC_ASSET_STAGING_MANIFEST_PATH),
        )
        .expect("render HPC asset staging manifest");
        let rows = load_rendered_command_argv_rows(&root).expect("load all-domain command argv");

        assert_eq!(
            manifest.schema_version,
            LOCAL_HPC_ASSET_STAGING_MANIFEST_SCHEMA_VERSION
        );
        assert_eq!(
            manifest.output_path,
            "runs/bench/hpc-dry-run/asset-staging-manifest.json"
        );
        assert_eq!(manifest.selected_job_count, rows.len());
        assert_eq!(manifest.jobs.len(), rows.len());
        assert!(manifest.staged_input_count >= manifest.jobs.len());
        assert!(manifest.unique_source_path_count > 0);
        assert!(manifest.jobs.iter().all(|job| !job.staged_inputs.is_empty()));

        let bowtie2 = manifest
            .jobs
            .iter()
            .find(|job| job.result_id == "bam:corpus-01-mini:bam.align:sample-set:bowtie2")
            .expect("locate bowtie2 benchmark job");
        assert!(bowtie2.staged_inputs.iter().any(|entry| {
            entry.source_path == "assets/reference/host/references/toy_host_reference"
                && entry.source_kind == "prefix_bundle"
                && entry.staged_path
                    == "runs/bench/hpc-dry-run/staged/assets/reference/host/references/toy_host_reference"
        }));
        assert!(bowtie2.staged_inputs.iter().any(|entry| {
            entry.source_path == "assets/toy/core-v1/fastq/reads_1.fastq"
                && entry.source_kind == "file"
        }));

        let bcftools = manifest
            .jobs
            .iter()
            .find(|job| {
                job.result_id == "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools"
            })
            .expect("locate bcftools benchmark job");
        assert!(bcftools.staged_inputs.iter().any(|entry| {
            entry.source_path
                == "benchmarks/readiness/adapters/bcftools/vcf.call/artifacts/reference/corpus_01_bam_reference.fasta"
                && entry.source_kind == "file"
        }));
        assert!(bcftools.staged_inputs.iter().any(|entry| {
            entry.source_path
                == "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam"
                && entry.source_kind == "file"
        }));
    }
}
