use super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob;
use super::local_hpc_input_discovery::collect_local_hpc_stage_input_hints;
use super::local_hpc_job_resources::load_local_hpc_job_resource_hints;
use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use super::readiness::all_domain_rendered_commands::render_all_domain_commands;
use super::readiness::benchmark_command_rows::DEFAULT_STAGE_TOOL_COMMAND_ROOT;
use super::readiness::essential_pipeline_rendered_commands::{
    render_essential_pipeline_commands, DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH,
};
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_api::v1::api::bench::load_hpc_scratch_layout as load_validated_hpc_scratch_layout_path;
use bijux_dna_api::v1::api::bench::{
    collect_hpc_scratch_layout as collect_layout, compare_hpc_scratch_layout,
    publish_hpc_scratch_layout, BenchmarkLocalHpcScratchLayout, HpcScratchCommands,
    HpcScratchPipelineCommands, HpcScratchRequest,
};
use std::path::{Path, PathBuf};
#[cfg(all(test, feature = "bam_downstream"))]
const LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_VERSION: &str =
    bijux_dna_api::v1::api::bench::BENCHMARK_LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_V1;
pub(crate) const DEFAULT_HPC_SCRATCH_LAYOUT_PATH: &str =
    "artifacts/runs/bench/hpc-dry-run/scratch-layout.json";
const DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH_FOR_HPC: &str =
    "artifacts/benchmark-smoke/readiness/hpc/rendered-commands-all-domains.sh";
pub(crate) type LocalHpcScratchLayout = BenchmarkLocalHpcScratchLayout;
pub(crate) fn run_render_hpc_scratch_layout(
    args: &parse::BenchLocalRenderHpcScratchLayoutArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_dry_run_root().join("scratch-layout.json")
    });
    let manifest = render_hpc_scratch_layout(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_scratch_layout(
    args: &parse::BenchLocalValidateHpcScratchLayoutArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_dry_run_root().join("scratch-layout.json")
    });
    let manifest = validate_hpc_scratch_layout_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.output_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_scratch_layout(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcScratchLayout> {
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let manifest = build_hpc_scratch_layout(repo_root, &absolute_output)?;
    publish_hpc_scratch_layout(repo_root, &absolute_output, &manifest)?;
    Ok(manifest)
}

pub(crate) fn collect_hpc_scratch_layout(repo_root: &Path) -> Result<LocalHpcScratchLayout> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let output_path = benchmark_paths.benchmark_hpc_dry_run_root().join("scratch-layout.json");
    build_hpc_scratch_layout(repo_root, &output_path)
}

pub(crate) fn collect_hpc_scratch_layout_for_jobs(
    repo_root: &Path,
    jobs: &[BenchLocalAllDomainSlurmSubmitJob],
) -> Result<LocalHpcScratchLayout> {
    let paths = BenchmarkPathResolver::new(repo_root, None);
    build_hpc_scratch_layout_for_jobs(
        repo_root,
        &paths.benchmark_hpc_dry_run_root().join("scratch-layout.json"),
        jobs,
    )
}

pub(crate) fn validate_hpc_scratch_layout_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcScratchLayout> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let observed = load_validated_hpc_scratch_layout_path(repo_root, &absolute_manifest_path)?;
    let expected = build_hpc_scratch_layout(repo_root, &absolute_manifest_path)?;
    compare_hpc_scratch_layout(&absolute_manifest_path, &observed, &expected)?;
    Ok(observed)
}

fn build_hpc_scratch_layout(
    repo_root: &Path,
    absolute_output: &Path,
) -> Result<LocalHpcScratchLayout> {
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        absolute_output,
        "HPC scratch layout output",
    )?;
    let jobs = load_local_hpc_selected_jobs(repo_root)?;
    build_hpc_scratch_layout_for_jobs(repo_root, absolute_output, &jobs)
}
fn build_hpc_scratch_layout_for_jobs(
    repo_root: &Path,
    absolute_output: &Path,
    submit_jobs: &[BenchLocalAllDomainSlurmSubmitJob],
) -> Result<LocalHpcScratchLayout> {
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        absolute_output,
        "HPC scratch layout output",
    )?;
    let paths = BenchmarkPathResolver::new(repo_root, None);
    let benchmark = render_all_domain_commands(
        repo_root,
        DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH_FOR_HPC.into(),
    )?;
    let pipeline = render_essential_pipeline_commands(
        repo_root,
        DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH.into(),
    )?;
    let commands = HpcScratchCommands {
        benchmark_rows: benchmark.rows,
        pipeline_rows: pipeline
            .rows
            .into_iter()
            .map(|row| HpcScratchPipelineCommands {
                pipeline_id: row.pipeline_id,
                node_id: row.node_id,
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                domain: row.domain,
                render_status: row.render_status,
                command_argv: row.command_steps.into_iter().map(|step| step.argv).collect(),
            })
            .collect(),
    };
    let hints = collect_local_hpc_stage_input_hints(repo_root, &commands.stage_ids(submit_jobs))?;
    let resources = load_local_hpc_job_resource_hints(repo_root)?;
    collect_layout(HpcScratchRequest {
        repository_root: repo_root,
        output_path: absolute_output,
        staging_root: &paths.benchmark_hpc_dry_run_root().join("staged"),
        scratch_root: &paths.benchmark_hpc_dry_run_root().join("scratch"),
        jobs: submit_jobs,
        commands: &commands,
        stage_input_hints: &hints,
        resource_hints: &resources,
        generated_output_root: DEFAULT_STAGE_TOOL_COMMAND_ROOT,
    })
}
#[cfg(all(test, feature = "bam_downstream"))]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        load_validated_hpc_scratch_layout_path, render_hpc_scratch_layout,
        validate_hpc_scratch_layout_path, LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    fn manifest_path(repo_root: &Path, label: &str) -> (tempfile::TempDir, PathBuf) {
        let temp_root = repo_root.join("artifacts/runs/bench/hpc-dry-run");
        std::fs::create_dir_all(&temp_root).expect("create HPC scratch-layout test root");
        let temp_dir = tempfile::Builder::new()
            .prefix(label)
            .tempdir_in(temp_root)
            .expect("temporary HPC dry-run directory");
        let manifest_path = temp_dir.path().join("scratch-layout.json");
        (temp_dir, manifest_path)
    }

    #[test]
    fn scratch_layout_test_sandbox_creates_clean_checkout_parent() {
        let clean_checkout = tempfile::tempdir().expect("clean checkout root");
        assert!(!clean_checkout.path().join("runs").exists());

        let (_temp_dir, manifest_path) =
            manifest_path(clean_checkout.path(), "clean-checkout-scratch-layout-");

        assert!(manifest_path.parent().is_some_and(Path::exists));
    }

    #[test]
    fn rendered_hpc_scratch_layout_covers_all_generated_jobs() {
        let root = repo_root();
        let (_temp_dir, manifest_path) = manifest_path(&root, "rendered-hpc-scratch-layout-");
        let manifest =
            render_hpc_scratch_layout(&root, manifest_path).expect("render HPC scratch layout");

        assert_eq!(manifest.schema_version, LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_VERSION);
        assert!(manifest.output_path.ends_with("/scratch-layout.json"));
        assert!(manifest.selected_job_count > 0);
        assert_eq!(
            manifest.selected_job_count,
            manifest.benchmark_job_count + manifest.essential_pipeline_job_count
        );
        assert!(manifest.input_link_count >= manifest.selected_job_count);
        assert!(manifest.jobs.iter().all(|job| !job.input_links.is_empty()));

        let benchmark_job = manifest
            .jobs
            .iter()
            .find(|job| {
                job.result_id.as_deref() == Some("bam:corpus-01-mini:bam.align:sample-set:bowtie2")
            })
            .expect("governed BAM benchmark job");
        assert!(benchmark_job
            .scratch_root
            .starts_with("artifacts/runs/bench/hpc-dry-run/scratch/benchmark-results/"));
        assert_eq!(
            benchmark_job.cleanup_policy.policy_id,
            "copy_back_outputs_remove_successful_scratch"
        );
        assert!(benchmark_job.input_links.iter().any(|input| {
            input.source_path == "assets/reference/host/references/toy_host_reference"
                && input.staged_path
                    == "artifacts/runs/bench/hpc-dry-run/staged/assets/reference/host/references/toy_host_reference"
                && input.link_path.starts_with(&benchmark_job.input_links_root)
        }));

        let pipeline_job = manifest
            .jobs
            .iter()
            .find(|job| job.pipeline_id.is_some() && job.node_id.is_some())
            .expect("essential pipeline job");
        assert!(pipeline_job
            .scratch_root
            .starts_with("artifacts/runs/bench/hpc-dry-run/scratch/essential-pipelines/"));
        assert!(!pipeline_job.output_root.is_empty());
        assert!(!pipeline_job.log_root.is_empty());
        assert!(pipeline_job.resources.scratch_gb > 0);
    }

    #[test]
    fn loaded_hpc_scratch_layout_matches_governed_render() {
        let root = repo_root();
        let (_temp_dir, manifest_path) = manifest_path(&root, "loaded-hpc-scratch-layout-");
        let rendered = render_hpc_scratch_layout(&root, manifest_path.clone())
            .expect("render HPC scratch layout");
        let loaded = load_validated_hpc_scratch_layout_path(&root, &manifest_path)
            .expect("load validated HPC scratch layout");

        assert_eq!(loaded, rendered);
    }

    #[test]
    fn validated_hpc_scratch_layout_rejects_stale_input_link_count() {
        let root = repo_root();
        let (_temp_dir, manifest_path) = manifest_path(&root, "validated-hpc-scratch-layout-");
        let rendered = render_hpc_scratch_layout(&root, manifest_path.clone())
            .expect("render HPC scratch layout");
        let stale_body =
            std::fs::read_to_string(&manifest_path).expect("read manifest body").replacen(
                &format!("\"input_link_count\": {}", rendered.input_link_count),
                &format!("\"input_link_count\": {}", rendered.input_link_count.saturating_sub(1)),
                1,
            );
        bijux_dna_infra::write_payload(&manifest_path, stale_body)
            .expect("write stale manifest body");

        let error = validate_hpc_scratch_layout_path(&root, &manifest_path)
            .expect_err("stale scratch layout must fail validation");
        assert!(
            error.to_string().contains("input_link_count aligned with input links"),
            "stale scratch layout error must report the input-link invariant: {error:#}"
        );
    }
}
