use super::cloning::clone_index_cache;
use super::layout::{
    sortmerna_sample_workdir, sortmerna_shared_index_dir, sortmerna_shared_index_seeded,
};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
pub(crate) fn prepare_sortmerna_sample_workdir(
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

pub(crate) fn promote_sortmerna_sample_index_cache(
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

pub(crate) fn prune_sortmerna_sample_payload(out_root: &Path, sample_id: &str) -> Result<()> {
    let sample_workdir = sortmerna_sample_workdir(out_root, sample_id);
    if sample_workdir.is_dir() {
        fs::remove_dir_all(&sample_workdir)
            .with_context(|| format!("remove {}", sample_workdir.display()))?;
    }
    Ok(())
}
