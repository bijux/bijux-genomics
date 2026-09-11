use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::Path;
pub(crate) fn clone_index_cache(source_idx_dir: &Path, dest_idx_dir: &Path) -> Result<()> {
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

pub(crate) fn copy_dir_hardlink_fallback(source_dir: &Path, dest_dir: &Path) -> Result<()> {
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
