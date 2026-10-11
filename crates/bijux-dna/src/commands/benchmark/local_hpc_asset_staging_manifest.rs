use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    validate_benchmark_local_hpc_asset_staging_manifest, BenchmarkLocalHpcAssetStagingInput,
    BenchmarkLocalHpcAssetStagingJob, BenchmarkLocalHpcAssetStagingManifest,
    BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1,
};
use serde::Deserialize;

use super::local_hpc_input_discovery::{
    collect_local_hpc_job_source_inputs, collect_local_hpc_stage_input_hints, materialize_stage_id,
    path_relative_to_repo, LocalHpcDiscoveredSourceInput, LocalHpcStageInputHint,
};
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use crate::commands::benchmark::readiness::all_domain_rendered_commands::render_all_domain_commands;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_HPC_ASSET_STAGING_MANIFEST_SCHEMA_VERSION: &str =
    BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1;
pub(crate) const DEFAULT_HPC_ASSET_STAGING_MANIFEST_PATH: &str =
    "artifacts/runs/bench/hpc-dry-run/asset-staging-manifest.json";
const DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_ARGV_PATH: &str =
    "artifacts/benchmark-smoke/readiness/hpc/rendered-commands-all-domains.argv.jsonl";
const DEFAULT_HPC_RENDERED_COMMANDS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/hpc/rendered-commands-all-domains.sh";

pub(crate) type LocalHpcAssetStagingManifest = BenchmarkLocalHpcAssetStagingManifest;
pub(crate) type LocalHpcAssetStagingJob = BenchmarkLocalHpcAssetStagingJob;
pub(crate) type LocalHpcAssetStagingInput = BenchmarkLocalHpcAssetStagingInput;

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

pub(crate) fn run_validate_hpc_asset_staging_manifest(
    args: &parse::BenchLocalValidateHpcAssetStagingManifestArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_dry_run_root().join("asset-staging-manifest.json")
    });
    let manifest = validate_hpc_asset_staging_manifest_path(&repo_root, &manifest_path)?;
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
    let manifest = build_hpc_asset_staging_manifest(repo_root, &absolute_output)?;
    bijux_dna_infra::atomic_write_json(&absolute_output, &manifest)?;
    Ok(manifest)
}

pub(crate) fn collect_hpc_asset_staging_manifest(
    repo_root: &Path,
) -> Result<LocalHpcAssetStagingManifest> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let output_path =
        benchmark_paths.benchmark_hpc_dry_run_root().join("asset-staging-manifest.json");
    build_hpc_asset_staging_manifest(repo_root, &output_path)
}

pub(crate) fn load_validated_hpc_asset_staging_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcAssetStagingManifest> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let raw = fs::read_to_string(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let manifest = serde_json::from_str::<LocalHpcAssetStagingManifest>(&raw)
        .with_context(|| format!("parse {}", absolute_manifest_path.display()))?;
    validate_benchmark_local_hpc_asset_staging_manifest(&manifest)?;
    let expected_output_path = path_relative_to_repo(repo_root, &absolute_manifest_path);
    if manifest.output_path != expected_output_path {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` declares output_path `{}` but is stored at `{}`",
            absolute_manifest_path.display(),
            manifest.output_path,
            expected_output_path
        ));
    }
    Ok(manifest)
}

pub(crate) fn validate_hpc_asset_staging_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcAssetStagingManifest> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let observed =
        load_validated_hpc_asset_staging_manifest_path(repo_root, &absolute_manifest_path)?;
    let expected = build_hpc_asset_staging_manifest(repo_root, &absolute_manifest_path)?;
    ensure_manifest_matches_expected(&absolute_manifest_path, &observed, &expected)?;
    Ok(observed)
}

fn build_hpc_asset_staging_manifest(
    repo_root: &Path,
    absolute_output: &Path,
) -> Result<LocalHpcAssetStagingManifest> {
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        absolute_output,
        "HPC asset staging manifest output",
    )?;
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let staging_root = benchmark_paths.benchmark_hpc_dry_run_root().join("staged");
    fs::create_dir_all(&staging_root)
        .with_context(|| format!("create {}", staging_root.display()))?;
    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let command_rows = load_rendered_command_argv_rows(repo_root)?;
    let stage_ids = command_rows
        .iter()
        .flat_map(|row| row.command_steps.iter())
        .filter_map(|step| materialize_stage_id(&step.argv))
        .chain(command_rows.iter().map(|row| row.stage_id.clone()))
        .collect::<BTreeSet<_>>();
    let stage_input_hints = collect_local_hpc_stage_input_hints(repo_root, &stage_ids)?;
    let mut jobs = Vec::with_capacity(command_rows.len());
    let mut unique_source_paths = BTreeSet::new();
    let mut staged_input_count = 0usize;

    for row in command_rows {
        let staged_inputs =
            collect_job_staged_inputs(repo_root, &staging_root, &row, &stage_input_hints)
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
        schema_version: LOCAL_HPC_ASSET_STAGING_MANIFEST_SCHEMA_VERSION.to_string(),
        output_path: path_relative_to_repo(repo_root, absolute_output),
        staging_root: path_relative_to_repo(repo_root, &staging_root),
        selected_job_count: jobs.len(),
        staged_input_count,
        unique_source_path_count: unique_source_paths.len(),
        jobs,
    };
    validate_benchmark_local_hpc_asset_staging_manifest(&manifest)?;
    Ok(manifest)
}

fn load_rendered_command_argv_rows(repo_root: &Path) -> Result<Vec<RenderedCommandArgvFileRow>> {
    let argv_path = repo_root.join(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_ARGV_PATH);
    render_all_domain_commands(repo_root, DEFAULT_HPC_RENDERED_COMMANDS_PATH.into())
        .with_context(|| format!("render {}", argv_path.display()))?;
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

fn collect_job_staged_inputs(
    repo_root: &Path,
    staging_root: &Path,
    row: &RenderedCommandArgvFileRow,
    stage_input_hints: &std::collections::BTreeMap<String, Vec<LocalHpcStageInputHint>>,
) -> Result<Vec<LocalHpcAssetStagingInput>> {
    collect_local_hpc_job_source_inputs(
        repo_root,
        &row.result_id,
        &row.stage_id,
        row.command_steps.iter().map(|step| step.argv.as_slice()),
        stage_input_hints,
    )?
    .into_iter()
    .map(|input| to_staged_input(repo_root, staging_root, &row.result_id, input))
    .collect()
}

fn to_staged_input(
    repo_root: &Path,
    staging_root: &Path,
    consumer_result_id: &str,
    input: LocalHpcDiscoveredSourceInput,
) -> Result<LocalHpcAssetStagingInput> {
    let staged_path =
        path_relative_to_repo(repo_root, &staging_root.join(&input.source_asset.source_path));
    Ok(LocalHpcAssetStagingInput {
        artifact_id: input.artifact_id,
        artifact_role: input.artifact_role,
        source_kind: input.source_asset.kind.as_str().to_string(),
        source_path: input.source_asset.source_path,
        staged_path,
        checksum_sha256: input.source_asset.checksum_sha256,
        size_bytes: input.source_asset.size_bytes,
        member_count: input.source_asset.member_count,
        consumer_result_id: consumer_result_id.to_string(),
    })
}

fn ensure_manifest_matches_expected(
    manifest_path: &Path,
    observed: &LocalHpcAssetStagingManifest,
    expected: &LocalHpcAssetStagingManifest,
) -> Result<()> {
    if observed.output_path != expected.output_path {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` drifted in output_path: observed `{}` expected `{}`",
            manifest_path.display(),
            observed.output_path,
            expected.output_path
        ));
    }
    if observed.staging_root != expected.staging_root {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` drifted in staging_root: observed `{}` expected `{}`",
            manifest_path.display(),
            observed.staging_root,
            expected.staging_root
        ));
    }
    if observed.selected_job_count != expected.selected_job_count {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` drifted in selected_job_count: observed {} expected {}",
            manifest_path.display(),
            observed.selected_job_count,
            expected.selected_job_count
        ));
    }
    if observed.staged_input_count != expected.staged_input_count {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` drifted in staged_input_count: observed {} expected {}",
            manifest_path.display(),
            observed.staged_input_count,
            expected.staged_input_count
        ));
    }
    if observed.unique_source_path_count != expected.unique_source_path_count {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` drifted in unique_source_path_count: observed {} expected {}",
            manifest_path.display(),
            observed.unique_source_path_count,
            expected.unique_source_path_count
        ));
    }
    if observed.jobs.len() != expected.jobs.len() {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` drifted in job count: observed {} expected {}",
            manifest_path.display(),
            observed.jobs.len(),
            expected.jobs.len()
        ));
    }

    for (index, (observed_job, expected_job)) in
        observed.jobs.iter().zip(expected.jobs.iter()).enumerate()
    {
        if observed_job.result_id != expected_job.result_id
            || observed_job.domain != expected_job.domain
            || observed_job.stage_id != expected_job.stage_id
            || observed_job.tool_id != expected_job.tool_id
            || observed_job.corpus_id != expected_job.corpus_id
            || observed_job.asset_profile_id != expected_job.asset_profile_id
        {
            return Err(anyhow!(
                "HPC asset staging manifest `{}` drifted in job {}: observed `{}` / `{}` / `{}` / `{}` / `{}` / `{}` expected `{}` / `{}` / `{}` / `{}` / `{}` / `{}`",
                manifest_path.display(),
                index,
                observed_job.result_id,
                observed_job.domain,
                observed_job.stage_id,
                observed_job.tool_id,
                observed_job.corpus_id,
                observed_job.asset_profile_id,
                expected_job.result_id,
                expected_job.domain,
                expected_job.stage_id,
                expected_job.tool_id,
                expected_job.corpus_id,
                expected_job.asset_profile_id
            ));
        }
        if observed_job.staged_inputs.len() != expected_job.staged_inputs.len() {
            return Err(anyhow!(
                "HPC asset staging manifest `{}` drifted in staged input count for `{}`: observed {} expected {}",
                manifest_path.display(),
                observed_job.result_id,
                observed_job.staged_inputs.len(),
                expected_job.staged_inputs.len()
            ));
        }
        for (input_index, (observed_input, expected_input)) in
            observed_job.staged_inputs.iter().zip(expected_job.staged_inputs.iter()).enumerate()
        {
            if observed_input != expected_input {
                return Err(anyhow!(
                    "HPC asset staging manifest `{}` drifted in staged input {} for `{}` at source `{}`",
                    manifest_path.display(),
                    input_index,
                    observed_job.result_id,
                    expected_input.source_path
                ));
            }
        }
    }

    Ok(())
}

#[cfg(all(test, feature = "bam_downstream"))]
mod tests {
    use std::path::PathBuf;

    use super::{
        load_rendered_command_argv_rows, load_validated_hpc_asset_staging_manifest_path,
        render_hpc_asset_staging_manifest, validate_hpc_asset_staging_manifest_path,
        LOCAL_HPC_ASSET_STAGING_MANIFEST_SCHEMA_VERSION,
    };
    use crate::commands::benchmark::repo_locking::{acquire_cwd_lock, RepoProcessLock};

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    fn manifest_path(repo_root: &std::path::Path, label: &str) -> (tempfile::TempDir, PathBuf) {
        let temp_root = repo_root.join("artifacts/runs/bench/test-sandboxes");
        std::fs::create_dir_all(&temp_root).expect("create HPC asset staging test root");
        let temp_dir = tempfile::Builder::new()
            .prefix(label)
            .tempdir_in(temp_root)
            .expect("create temp manifest directory");
        let manifest_path = temp_dir.path().join("asset-staging-manifest.json");
        (temp_dir, manifest_path)
    }

    #[test]
    fn rendered_hpc_asset_staging_manifest_covers_all_domain_jobs() {
        let root = repo_root();
        let _cwd_guard = acquire_cwd_lock();
        let _repo_lock =
            RepoProcessLock::acquire(&root, "benchmark-readiness-mutators").expect("repo lock");
        let (_temp_dir, manifest_path) = manifest_path(&root, "rendered-hpc-asset-staging-");
        let manifest = render_hpc_asset_staging_manifest(&root, manifest_path)
            .expect("render HPC asset staging manifest");
        let rows = load_rendered_command_argv_rows(&root).expect("load all-domain command argv");

        assert_eq!(manifest.schema_version, LOCAL_HPC_ASSET_STAGING_MANIFEST_SCHEMA_VERSION);
        assert!(manifest.output_path.starts_with("artifacts/runs/bench/test-sandboxes/"));
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
                    == "artifacts/runs/bench/hpc-dry-run/staged/assets/reference/host/references/toy_host_reference"
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
                == "artifacts/benchmark-smoke/readiness/adapters/bcftools/vcf.call/artifacts/reference/corpus_01_bam_reference.fasta"
                && entry.source_kind == "file"
        }));
        assert!(bcftools.staged_inputs.iter().any(|entry| {
            entry.source_path
                == "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam"
                && entry.source_kind == "file"
        }));
    }

    #[test]
    fn loaded_hpc_asset_staging_manifest_matches_governed_render() {
        let root = repo_root();
        let _cwd_guard = acquire_cwd_lock();
        let _repo_lock =
            RepoProcessLock::acquire(&root, "benchmark-readiness-mutators").expect("repo lock");
        let (_temp_dir, manifest_path) = manifest_path(&root, "loaded-hpc-asset-staging-");
        let rendered = render_hpc_asset_staging_manifest(&root, manifest_path.clone())
            .expect("render HPC asset staging manifest");
        let loaded = load_validated_hpc_asset_staging_manifest_path(&root, &manifest_path)
            .expect("load validated HPC asset staging manifest");

        assert_eq!(loaded, rendered);
        assert!(loaded.output_path.starts_with("artifacts/runs/bench/test-sandboxes/"));
    }

    #[test]
    fn validated_hpc_asset_staging_manifest_rejects_stale_selected_job_count() {
        let root = repo_root();
        let _cwd_guard = acquire_cwd_lock();
        let _repo_lock =
            RepoProcessLock::acquire(&root, "benchmark-readiness-mutators").expect("repo lock");
        let (_temp_dir, manifest_path) = manifest_path(&root, "validated-hpc-asset-staging-");
        let rendered = render_hpc_asset_staging_manifest(&root, manifest_path.clone())
            .expect("render HPC asset staging manifest");
        let stale_body =
            std::fs::read_to_string(&manifest_path).expect("read manifest body").replacen(
                &format!("\"selected_job_count\": {}", rendered.selected_job_count),
                &format!(
                    "\"selected_job_count\": {}",
                    rendered.selected_job_count.saturating_sub(1)
                ),
                1,
            );
        bijux_dna_infra::write_payload(&manifest_path, stale_body)
            .expect("write stale manifest body");

        let error = validate_hpc_asset_staging_manifest_path(&root, &manifest_path)
            .expect_err("stale manifest must fail validation");
        assert!(
            error.to_string().contains("selected_job_count"),
            "stale manifest error must name the drifted field: {error:#}"
        );
    }
}
