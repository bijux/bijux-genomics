use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};

use bijux_engine::bench::ExecutionManifest;

pub fn replay_run(run_id: &str, search_root: &Path) -> Result<()> {
    let manifest_path = find_manifest(search_root, run_id)?
        .ok_or_else(|| anyhow!("run_id {run_id} not found under {}", search_root.display()))?;
    let manifest_bytes = std::fs::read(&manifest_path)
        .with_context(|| format!("read manifest {}", manifest_path.display()))?;
    let manifest: ExecutionManifest = serde_json::from_slice(&manifest_bytes)
        .with_context(|| format!("parse manifest {}", manifest_path.display()))?;
    println!(
        "replay: {} {} (run_id={})",
        manifest.stage, manifest.tool, manifest.run_id
    );
    let status = Command::new("sh")
        .arg("-lc")
        .arg(&manifest.command)
        .status()
        .context("run replay command")?;
    if !status.success() {
        return Err(anyhow!("replay failed with status {status}"));
    }
    Ok(())
}

fn find_manifest(root: &Path, run_id: &str) -> Result<Option<PathBuf>> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|s| s.to_str()) == Some("manifest.json") {
                let bytes = std::fs::read(&path)
                    .with_context(|| format!("read manifest {}", path.display()))?;
                if let Ok(manifest) = serde_json::from_slice::<ExecutionManifest>(&bytes) {
                    if manifest.run_id == run_id {
                        return Ok(Some(path));
                    }
                }
            }
        }
    }
    Ok(None)
}
