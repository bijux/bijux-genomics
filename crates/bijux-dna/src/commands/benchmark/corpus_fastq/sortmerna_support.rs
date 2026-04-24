use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use super::{parse_cli_arg_pairs, path_display};
use crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig;

const DEPLETE_RRNA_DEFAULT_MIN_IDENTITY: f64 = 0.95;

#[derive(Debug, Clone)]
pub(super) struct DepleteRrnaStageOptions {
    pub(super) rrna_db: PathBuf,
    pub(super) rrna_bundle_id: String,
    pub(super) min_identity: f64,
}

pub(super) fn resolve_deplete_rrna_stage_options(
    stage_args: &[String],
) -> Result<DepleteRrnaStageOptions> {
    let stage_values = parse_cli_arg_pairs("stage-arg", stage_args)?;
    let rrna_db = stage_values
        .get("rrna_db")
        .ok_or_else(|| anyhow!("fastq.deplete_rrna requires --rrna-db in stage arguments"))?;
    let rrna_db = PathBuf::from(rrna_db);
    if !rrna_db.is_file() {
        return Err(anyhow!("missing rRNA bundle FASTA: {}", rrna_db.display()));
    }
    Ok(DepleteRrnaStageOptions {
        rrna_db,
        rrna_bundle_id: stage_values.get("rrna_bundle_id").cloned().ok_or_else(|| {
            anyhow!("fastq.deplete_rrna requires --rrna-bundle-id in stage arguments")
        })?,
        min_identity: stage_values
            .get("min_identity")
            .map(|row| row.parse::<f64>())
            .transpose()
            .with_context(|| "parse min_identity from stage arguments")?
            .unwrap_or(DEPLETE_RRNA_DEFAULT_MIN_IDENTITY),
    })
}

fn sortmerna_sample_workdir(out_root: &Path, sample_id: &str) -> PathBuf {
    out_root
        .join("bench")
        .join("deplete_rrna")
        .join(sample_id)
        .join("tools")
        .join("sortmerna")
        .join("sortmerna_workdir")
}

pub(super) fn sortmerna_shared_index_dir(out_root: &Path, rrna_bundle_id: &str) -> PathBuf {
    out_root
        .join("_reference_cache")
        .join("fastq.deplete_rrna")
        .join(rrna_bundle_id)
        .join("sortmerna_workdir")
        .join("idx")
}

pub(super) fn sortmerna_shared_index_seeded(shared_idx_dir: &Path) -> bool {
    shared_idx_dir.is_dir()
        && shared_idx_dir.read_dir().ok().is_some_and(|mut row| row.next().is_some())
}

fn clone_index_cache(source_idx_dir: &Path, dest_idx_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_idx_dir)
        .with_context(|| format!("create {}", dest_idx_dir.display()))?;
    for entry in fs::read_dir(source_idx_dir)
        .with_context(|| format!("read {}", source_idx_dir.display()))?
    {
        let source_path = entry?.path();
        let dest_path = dest_idx_dir.join(
            source_path
                .file_name()
                .ok_or_else(|| anyhow!("missing file name for {}", source_path.display()))?,
        );
        if dest_path.exists() {
            if dest_path.is_dir() {
                fs::remove_dir_all(&dest_path)
                    .with_context(|| format!("remove {}", dest_path.display()))?;
            } else {
                fs::remove_file(&dest_path)
                    .with_context(|| format!("remove {}", dest_path.display()))?;
            }
        }
        if source_path.is_dir() {
            copy_dir_hardlink_fallback(&source_path, &dest_path)?;
        } else {
            match fs::hard_link(&source_path, &dest_path) {
                Ok(()) => {}
                Err(_) => {
                    fs::copy(&source_path, &dest_path).with_context(|| {
                        format!("copy {} -> {}", source_path.display(), dest_path.display())
                    })?;
                }
            }
        }
    }
    Ok(())
}

fn copy_dir_hardlink_fallback(source_dir: &Path, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir).with_context(|| format!("create {}", dest_dir.display()))?;
    for entry in
        fs::read_dir(source_dir).with_context(|| format!("read {}", source_dir.display()))?
    {
        let source_path = entry?.path();
        let dest_path = dest_dir.join(
            source_path
                .file_name()
                .ok_or_else(|| anyhow!("missing file name for {}", source_path.display()))?,
        );
        if source_path.is_dir() {
            copy_dir_hardlink_fallback(&source_path, &dest_path)?;
        } else {
            match fs::hard_link(&source_path, &dest_path) {
                Ok(()) => {}
                Err(_) => {
                    fs::copy(&source_path, &dest_path).with_context(|| {
                        format!("copy {} -> {}", source_path.display(), dest_path.display())
                    })?;
                }
            }
        }
    }
    Ok(())
}

pub(super) fn prepare_sortmerna_sample_workdir(
    out_root: &Path,
    sample_id: &str,
    rrna_bundle_id: &str,
) -> Result<PathBuf> {
    let shared_idx_dir = sortmerna_shared_index_dir(out_root, rrna_bundle_id);
    fs::create_dir_all(&shared_idx_dir)
        .with_context(|| format!("create {}", shared_idx_dir.display()))?;
    let sample_workdir = sortmerna_sample_workdir(out_root, sample_id);
    fs::create_dir_all(&sample_workdir)
        .with_context(|| format!("create {}", sample_workdir.display()))?;
    let sample_idx_dir = sample_workdir.join("idx");
    if sample_idx_dir.is_symlink() || sample_idx_dir.is_file() {
        fs::remove_file(&sample_idx_dir)
            .with_context(|| format!("remove {}", sample_idx_dir.display()))?;
    } else if sample_idx_dir.is_dir() {
        fs::remove_dir_all(&sample_idx_dir)
            .with_context(|| format!("remove {}", sample_idx_dir.display()))?;
    }
    if sortmerna_shared_index_seeded(&shared_idx_dir) {
        clone_index_cache(&shared_idx_dir, &sample_idx_dir)?;
    } else {
        fs::create_dir_all(&sample_idx_dir)
            .with_context(|| format!("create {}", sample_idx_dir.display()))?;
    }
    Ok(shared_idx_dir)
}

pub(super) fn promote_sortmerna_sample_index_cache(
    out_root: &Path,
    sample_id: &str,
    rrna_bundle_id: &str,
) -> Result<PathBuf> {
    let shared_idx_dir = sortmerna_shared_index_dir(out_root, rrna_bundle_id);
    let sample_idx_dir = sortmerna_sample_workdir(out_root, sample_id).join("idx");
    if !sample_idx_dir.is_dir() {
        return Err(anyhow!("missing SortMeRNA sample idx dir: {}", sample_idx_dir.display()));
    }
    if !sortmerna_shared_index_seeded(&shared_idx_dir) {
        clone_index_cache(&sample_idx_dir, &shared_idx_dir)?;
    }
    Ok(shared_idx_dir)
}

pub(super) fn prune_sortmerna_sample_payload(out_root: &Path, sample_id: &str) -> Result<()> {
    let sample_workdir = sortmerna_sample_workdir(out_root, sample_id);
    if sample_workdir.is_dir() {
        fs::remove_dir_all(&sample_workdir)
            .with_context(|| format!("remove {}", sample_workdir.display()))?;
    }
    Ok(())
}

fn deplete_rrna_bind_root(workspace_config: &BenchmarkWorkspaceConfig) -> Option<PathBuf> {
    workspace_config
        .remote
        .as_ref()
        .and_then(|row| row.repo_root.as_ref())
        .and_then(|row| Path::new(row).parent())
        .map(Path::to_path_buf)
}

fn apptainer_container_input_path(bind_root: &Path, host_path: &Path) -> Result<String> {
    let resolved = host_path.canonicalize().unwrap_or_else(|_| host_path.to_path_buf());
    let relative = resolved.strip_prefix(bind_root).with_context(|| {
        format!(
            "{} must live under {} for Apptainer warmup",
            resolved.display(),
            bind_root.display()
        )
    })?;
    Ok(format!("/data/input/{}", path_display(relative)))
}

pub(super) fn supports_apptainer_warmup(platform: &str) -> bool {
    platform.contains("apptainer")
}

pub(super) fn warm_sortmerna_shared_index_cache(
    workspace_config: &BenchmarkWorkspaceConfig,
    platform: &str,
    rrna_db: &Path,
    seed_r1: &Path,
    shared_idx_dir: &Path,
    threads: u32,
) -> Result<()> {
    if !supports_apptainer_warmup(platform) {
        return Ok(());
    }
    fs::create_dir_all(shared_idx_dir)
        .with_context(|| format!("create {}", shared_idx_dir.display()))?;
    if sortmerna_shared_index_seeded(shared_idx_dir) {
        return Ok(());
    }
    let cache_workdir = shared_idx_dir
        .parent()
        .ok_or_else(|| anyhow!("missing SortMeRNA shared workdir parent"))?;
    fs::create_dir_all(cache_workdir)
        .with_context(|| format!("create {}", cache_workdir.display()))?;
    let bind_root = deplete_rrna_bind_root(workspace_config).ok_or_else(|| {
        anyhow!("workspace config is missing remote.repo_root for SortMeRNA warmup")
    })?;
    let sif_path =
        bind_root.join("bijux-dna-container").join("apptainer").join("sif").join("sortmerna.sif");
    if !sif_path.is_file() {
        return Err(anyhow!("missing SortMeRNA Apptainer image: {}", sif_path.display()));
    }
    let rrna_input = apptainer_container_input_path(&bind_root, rrna_db)?;
    let seed_input = apptainer_container_input_path(&bind_root, seed_r1)?;
    let warm_threads = threads.clamp(1, 4);
    let command_args = vec![
        "exec".to_string(),
        "--cleanenv".to_string(),
        "--no-home".to_string(),
        "--containall".to_string(),
        "--bind".to_string(),
        format!("{}:/data/input:ro", bind_root.display()),
        "--bind".to_string(),
        format!("{}:/data/output", cache_workdir.display()),
        "--pwd".to_string(),
        "/data/output".to_string(),
        sif_path.display().to_string(),
        "/usr/local/bin/sortmerna-bin".to_string(),
        "--ref".to_string(),
        rrna_input,
        "--reads".to_string(),
        seed_input,
        "--workdir".to_string(),
        "/data/output/".to_string(),
        "--index".to_string(),
        "1".to_string(),
        "--threads".to_string(),
        warm_threads.to_string(),
    ];
    let output = bijux_dna_api::v1::api::run::run_command("apptainer", &command_args)
        .context("run SortMeRNA shared-index warmup")?;
    if output.exit_code != 0 {
        return Err(anyhow!(
            "SortMeRNA shared-index warmup failed with exit code {} for {}",
            output.exit_code,
            shared_idx_dir.display()
        ));
    }
    if !sortmerna_shared_index_seeded(shared_idx_dir) {
        return Err(anyhow!(
            "SortMeRNA warmup did not materialize idx files at {}",
            shared_idx_dir.display()
        ));
    }
    Ok(())
}
