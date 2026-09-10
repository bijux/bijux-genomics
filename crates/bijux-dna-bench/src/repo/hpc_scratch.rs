//! Owner: current scratch source links, destination admission and manifest persistence.
use crate::{
    validate_benchmark_local_hpc_scratch_layout, BenchmarkLocalHpcScratchInputLink as InputLink,
    BenchmarkLocalHpcScratchLayout as Layout, HpcDiscoveredSourceInput,
};
use anyhow::{anyhow, ensure, Context, Result};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
pub(crate) fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
pub(crate) fn destination(root: &Path, path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() { path.to_path_buf() } else { root.join(path) };
    let relative = path
        .strip_prefix(root.join("artifacts/runs/bench"))
        .context("HPC scratch layout output must resolve inside `artifacts/runs/bench`")?;
    ensure!(
        !relative.as_os_str().is_empty()
            && relative.components().all(|part| matches!(part, Component::Normal(_))),
        "HPC scratch layout output must be a normalized path inside `artifacts/runs/bench`"
    );
    Ok(path)
}
/// Publish admitted scratch evidence at its declared destination.
/// # Errors
/// Rejects unsafe destinations, invalid manifests, stored identity mismatch or write failure.
pub fn publish_hpc_scratch_layout(root: &Path, output: &Path, manifest: &Layout) -> Result<()> {
    let output = destination(root, output)?;
    validate_benchmark_local_hpc_scratch_layout(manifest)?;
    ensure!(
        manifest.output_path == relative(root, &output),
        "HPC scratch layout output_path differs from publication destination"
    );
    bijux_dna_infra::atomic_write_json(&output, manifest)?;
    Ok(())
}
/// Load admitted scratch evidence bound to its stored path.
/// # Errors
/// Returns read, decode, admission or stored-path identity failures.
pub fn load_hpc_scratch_layout(repo_root: &Path, manifest_path: &Path) -> Result<Layout> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let raw = fs::read_to_string(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let manifest = serde_json::from_str::<Layout>(&raw)
        .with_context(|| format!("parse {}", absolute_manifest_path.display()))?;
    validate_benchmark_local_hpc_scratch_layout(&manifest)?;
    let expected_output_path = relative(repo_root, &absolute_manifest_path);
    if manifest.output_path != expected_output_path {
        return Err(anyhow!(
            "HPC scratch layout `{}` declares output_path `{}` but is stored at `{}`",
            absolute_manifest_path.display(),
            manifest.output_path,
            expected_output_path
        ));
    }
    Ok(manifest)
}

pub(crate) fn input_link(
    repo_root: &Path,
    staging_root: &Path,
    input_links_root: &Path,
    input: HpcDiscoveredSourceInput,
) -> Result<InputLink> {
    let staged_path = relative(repo_root, &staging_root.join(&input.source_asset.source_path));
    let link_path = relative(repo_root, &input_links_root.join(&input.source_asset.source_path));
    Ok(InputLink {
        artifact_id: input.artifact_id,
        artifact_role: input.artifact_role,
        source_kind: input.source_asset.kind.as_str().to_string(),
        source_path: input.source_asset.source_path,
        staged_path,
        link_path,
        checksum_sha256: input.source_asset.checksum_sha256,
        size_bytes: input.source_asset.size_bytes,
        member_count: input.source_asset.member_count,
    })
}
