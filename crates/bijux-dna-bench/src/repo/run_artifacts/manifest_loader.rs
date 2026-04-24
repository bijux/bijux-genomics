use std::path::PathBuf;

use anyhow::{Context, Result};

#[allow(dead_code)]
pub fn load_manifest(path: &PathBuf) -> Result<bijux_dna_core::contract::ExecutionManifest> {
    let bytes = std::fs::read(path).with_context(|| format!("read manifest {}", path.display()))?;
    let manifest: bijux_dna_core::contract::ExecutionManifest = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse manifest {}", path.display()))?;
    Ok(manifest)
}
