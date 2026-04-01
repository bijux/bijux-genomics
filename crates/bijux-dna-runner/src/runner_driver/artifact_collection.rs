use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_runtime::Artifact;

pub(super) fn collect_existing_artifacts(paths: Vec<PathBuf>) -> Result<Vec<Artifact>> {
    let mut artifacts = Vec::new();
    for path in paths {
        if path.exists() {
            let sha256 = bijux_dna_infra::hash_file_sha256(&path)?;
            artifacts.push(Artifact { path, sha256 });
        }
    }
    Ok(artifacts)
}
