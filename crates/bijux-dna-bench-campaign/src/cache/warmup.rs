use super::layout::sortmerna_shared_index_seeded;
use crate::ports::{CampaignCommand, CampaignExecution, CampaignPaths};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
pub(crate) fn deplete_rrna_bind_root(workspace_config: &dyn CampaignPaths) -> Option<PathBuf> {
    workspace_config.apptainer_bind_root()
}

pub(crate) fn apptainer_container_input_path(bind_root: &Path, host_path: &Path) -> Result<String> {
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

pub(crate) fn supports_apptainer_warmup(platform: &str) -> bool {
    platform.contains("apptainer")
}

pub(crate) fn warm_sortmerna_shared_index_cache(
    workspace_config: &dyn CampaignPaths,
    platform: &str,
    rrna_db: &Path,
    seed_r1: &Path,
    shared_idx_dir: &Path,
    threads: u32,
    executor: &dyn CampaignExecution,
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
    let exit_code = executor
        .execute(CampaignCommand {
            program: "apptainer",
            arguments: &command_args,
            cwd: None,
            environment: None,
        })
        .context("run SortMeRNA shared-index warmup")?;
    if exit_code != 0 {
        return Err(anyhow!(
            "SortMeRNA shared-index warmup failed with exit code {} for {}",
            exit_code,
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

fn path_display(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
