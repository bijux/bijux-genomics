//! Owner: staging destinations, current source observations and persistence.
use crate::{
    collect_hpc_job_source_inputs, validate_benchmark_local_hpc_asset_staging_manifest,
    AllDomainRenderedCommandRow, BenchmarkLocalHpcAssetStagingInput,
    BenchmarkLocalHpcAssetStagingJob, BenchmarkLocalHpcAssetStagingManifest,
    HpcDiscoveredSourceInput, HpcStageInputHint,
};
use anyhow::{anyhow, ensure, Context, Result};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};
pub(crate) fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
pub(crate) fn prepare(root: &Path, output: &Path, staging: &Path) -> Result<(PathBuf, PathBuf)> {
    let output = if output.is_absolute() { output.to_path_buf() } else { root.join(output) };
    let staging = if staging.is_absolute() { staging.to_path_buf() } else { root.join(staging) };
    for (path, label) in
        [(&output, "HPC asset staging manifest output"), (&staging, "HPC asset staging root")]
    {
        let relative = path
            .strip_prefix(root.join("artifacts/runs/bench"))
            .with_context(|| format!("{label} must resolve inside `artifacts/runs/bench`"))?;
        ensure!(
            !relative.as_os_str().is_empty()
                && relative.components().all(|part| matches!(part, Component::Normal(_))),
            "{label} must be a normalized path inside `artifacts/runs/bench`"
        );
    }
    ensure!(
        !output.starts_with(&staging) && !staging.starts_with(&output),
        "HPC asset staging manifest and staging root must not overlap"
    );
    Ok((output, staging))
}
pub(crate) fn prepare_directories(output: &Path, staging: &Path) -> Result<()> {
    bijux_dna_infra::ensure_dir(staging)
        .with_context(|| format!("create {}", staging.display()))?;
    if let Some(parent) = output.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    Ok(())
}
pub(crate) fn observe_jobs(
    root: &Path,
    staging: &Path,
    rows: Vec<AllDomainRenderedCommandRow>,
    hints: &BTreeMap<String, Vec<HpcStageInputHint>>,
    generated: &str,
) -> Result<Vec<BenchmarkLocalHpcAssetStagingJob>> {
    rows.into_iter()
        .map(|row| {
            let inputs = collect_hpc_job_source_inputs(
                root,
                &row.result_id,
                &row.stage_id,
                row.command_steps.iter().map(|step| step.argv.as_slice()),
                hints,
                generated,
            )
            .with_context(|| {
                format!(
                    "collect staged inputs for `{}` / `{}` / `{}`",
                    row.result_id, row.stage_id, row.tool_id
                )
            })?;
            let staged_inputs = inputs
                .into_iter()
                .map(|input| staged_input(root, staging, &row.result_id, input))
                .collect();
            Ok(BenchmarkLocalHpcAssetStagingJob {
                result_id: row.result_id,
                domain: row.domain,
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                corpus_id: row.corpus_id,
                asset_profile_id: row.asset_profile_id,
                staged_inputs,
            })
        })
        .collect()
}
fn staged_input(
    root: &Path,
    staging: &Path,
    consumer: &str,
    input: HpcDiscoveredSourceInput,
) -> BenchmarkLocalHpcAssetStagingInput {
    BenchmarkLocalHpcAssetStagingInput {
        staged_path: relative(root, &staging.join(&input.source_asset.source_path)),
        artifact_id: input.artifact_id,
        artifact_role: input.artifact_role,
        source_kind: input.source_asset.kind.as_str().to_string(),
        source_path: input.source_asset.source_path,
        checksum_sha256: input.source_asset.checksum_sha256,
        size_bytes: input.source_asset.size_bytes,
        member_count: input.source_asset.member_count,
        consumer_result_id: consumer.to_string(),
    }
}
/// Publish an admitted manifest at its declared repository-relative destination.
/// # Errors
/// Returns destination, schema or atomic publication failures.
pub fn publish_hpc_asset_staging_manifest(
    root: &Path,
    output: &Path,
    manifest: &BenchmarkLocalHpcAssetStagingManifest,
) -> Result<()> {
    validate_benchmark_local_hpc_asset_staging_manifest(manifest)?;
    let (output, _) = prepare(root, output, Path::new(&manifest.staging_root))?;
    ensure!(
        manifest.output_path == relative(root, &output),
        "HPC asset staging manifest output_path differs from publication destination"
    );
    bijux_dna_infra::atomic_write_json(&output, manifest)?;
    Ok(())
}
/// Load an admitted manifest bound to its stored path.
/// # Errors
/// Returns read, decode, schema or stored-path identity failures.
pub fn load_hpc_asset_staging_manifest(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BenchmarkLocalHpcAssetStagingManifest> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let raw = fs::read_to_string(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let manifest = serde_json::from_str::<BenchmarkLocalHpcAssetStagingManifest>(&raw)
        .with_context(|| format!("parse {}", absolute_manifest_path.display()))?;
    validate_benchmark_local_hpc_asset_staging_manifest(&manifest)?;
    let expected_output_path = relative(repo_root, &absolute_manifest_path);
    if manifest.output_path != expected_output_path {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` declares output_path `{}` but is stored at `{}`",
            absolute_manifest_path.display(),
            manifest.output_path,
            expected_output_path
        ));
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn staging_destinations_reject_escape_and_overlap_before_directory_effects() -> Result<()> {
        let root = bijux_dna_testkit::temp::tempdir_for("staging-destinations");
        let output = Path::new("artifacts/runs/bench/request/manifest.json");
        let staging = Path::new("artifacts/runs/bench/request/staged");
        let (prepared, prepared_staging) = prepare(root.path(), output, staging)?;
        assert_eq!(prepared, root.path().join(output));
        assert_eq!(prepared_staging, root.path().join(staging));
        for invalid in [
            "outside.json",
            "artifacts/runs/bench/../outside.json",
            "artifacts/runs/bench/request/staged/manifest.json",
        ] {
            assert!(prepare(root.path(), Path::new(invalid), staging).is_err());
        }
        assert!(!root.path().join("artifacts").exists());
        Ok(())
    }
}
