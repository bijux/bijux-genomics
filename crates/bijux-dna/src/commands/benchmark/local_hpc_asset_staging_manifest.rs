use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_api::v1::api::bench::{
    collect_hpc_asset_staging_manifest as collect_manifest,
    compare_hpc_asset_staging_manifest as ensure_manifest_matches_expected,
    publish_hpc_asset_staging_manifest, AllDomainRenderedCommandRow,
    BenchmarkLocalHpcAssetStagingManifest, HpcAssetStagingRequest,
};

use super::local_hpc_input_discovery::{collect_local_hpc_stage_input_hints, materialize_stage_id};
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use crate::commands::benchmark::readiness::all_domain_rendered_commands::render_all_domain_commands;
use crate::commands::cli::parse;
use crate::commands::cli::render;

#[cfg(all(test, feature = "bam_downstream"))]
const LOCAL_HPC_ASSET_STAGING_MANIFEST_SCHEMA_VERSION: &str =
    bijux_dna_api::v1::api::bench::BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1;
pub(crate) const DEFAULT_HPC_ASSET_STAGING_MANIFEST_PATH: &str =
    "artifacts/runs/bench/hpc-dry-run/asset-staging-manifest.json";
const DEFAULT_HPC_RENDERED_COMMANDS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/hpc/rendered-commands-all-domains.sh";

pub(crate) type LocalHpcAssetStagingManifest = BenchmarkLocalHpcAssetStagingManifest;

use super::readiness::benchmark_command_rows::DEFAULT_STAGE_TOOL_COMMAND_ROOT;
pub(crate) use bijux_dna_api::v1::api::bench::load_hpc_asset_staging_manifest as load_validated_hpc_asset_staging_manifest_path;
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
    publish_hpc_asset_staging_manifest(repo_root, &absolute_output, &manifest)?;
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
    let paths = BenchmarkPathResolver::new(repo_root, None);
    let staging_root = paths.benchmark_hpc_dry_run_root().join("staged");
    let rows = load_rendered_command_argv_rows(repo_root)?;
    let stage_ids = rows
        .iter()
        .flat_map(|row| row.command_steps.iter())
        .filter_map(|step| materialize_stage_id(&step.argv))
        .chain(rows.iter().map(|row| row.stage_id.clone()))
        .collect::<BTreeSet<_>>();
    let hints = collect_local_hpc_stage_input_hints(repo_root, &stage_ids)?;
    collect_manifest(HpcAssetStagingRequest {
        repository_root: repo_root,
        output_path: absolute_output,
        staging_root: &staging_root,
        command_rows: rows,
        stage_input_hints: &hints,
        generated_output_root: DEFAULT_STAGE_TOOL_COMMAND_ROOT,
    })
}
fn load_rendered_command_argv_rows(repo_root: &Path) -> Result<Vec<AllDomainRenderedCommandRow>> {
    let report = render_all_domain_commands(repo_root, DEFAULT_HPC_RENDERED_COMMANDS_PATH.into())?;
    Ok(report.rows)
}
#[cfg(all(test, feature = "bam_downstream"))]
mod tests {
    use std::path::PathBuf;

    use super::{
        load_rendered_command_argv_rows, load_validated_hpc_asset_staging_manifest_path,
        render_hpc_asset_staging_manifest, validate_hpc_asset_staging_manifest_path,
        LOCAL_HPC_ASSET_STAGING_MANIFEST_SCHEMA_VERSION,
    };
    use crate::commands::benchmark::scratch_repository::materialize_repository_sandbox;

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
        let sandbox = materialize_repository_sandbox("rendered-hpc-asset-staging-");
        let root = sandbox.path().to_path_buf();
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
        let sandbox = materialize_repository_sandbox("loaded-hpc-asset-staging-");
        let root = sandbox.path().to_path_buf();
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
        let sandbox = materialize_repository_sandbox("validated-hpc-asset-staging-");
        let root = sandbox.path().to_path_buf();
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
