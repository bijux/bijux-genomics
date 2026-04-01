use anyhow::{Context, Result};
use std::{fs, path::Path};

use bijux_dna_db_ena::EnaRunManifest;

pub(crate) fn write_manifest(path: &Path, manifest: &EnaRunManifest) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create manifest directory {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(manifest)?;
    fs::write(path, json).with_context(|| format!("write manifest {}", path.display()))
}
