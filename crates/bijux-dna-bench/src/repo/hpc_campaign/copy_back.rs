//! Owner: persisted campaign copy-back manifest publication.
use anyhow::{Context, Result};
use bijux_dna_bench_model::CopyBackManifestReport;
use std::path::Path;

pub(crate) fn publish_copy_back_manifest(manifest: &CopyBackManifestReport) -> Result<()> {
    let path = Path::new(&manifest.manifest_path);
    let payload = serde_json::to_vec_pretty(manifest).context("serialize copy-back manifest")?;
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_bytes(path, &payload)
        .with_context(|| format!("write {}", path.display()))
}
