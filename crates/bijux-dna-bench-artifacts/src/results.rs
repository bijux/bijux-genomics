//! Owner: loading and admitting persisted stage execution results.
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_stage_result_manifest, BenchmarkStageResultManifest,
};
use std::{fs, path::Path};
/// Load and structurally validate one stage execution result manifest.
///
/// # Errors
/// Returns an error when the manifest cannot be read, parsed, or validated.
pub fn load_benchmark_stage_result_manifest(
    manifest_path: &Path,
) -> Result<BenchmarkStageResultManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let manifest: BenchmarkStageResultManifest =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))?;
    validate_benchmark_stage_result_manifest(&manifest)
        .with_context(|| format!("validate {}", manifest_path.display()))?;
    Ok(manifest)
}
