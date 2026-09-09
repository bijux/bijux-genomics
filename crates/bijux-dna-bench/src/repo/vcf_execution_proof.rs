//! Owner: benchmark repository acquisition.
//! Acquire only the execution report declared by a validated VCF stage result.

use crate::artifacts::vcf_execution_proof;
use anyhow::{ensure, Context, Result};
use bijux_dna_bench_model::BenchmarkStageResultManifest;
use std::path::{Path, PathBuf};

/// Load and interpret the selected tool's versioned execution proof.
///
/// # Errors
/// Returns an error for missing, ambiguous, malformed or contradictory proof reports.
pub fn load_vcf_execution_blocker(
    repo_root: &Path,
    manifest: &BenchmarkStageResultManifest,
) -> Result<Option<String>> {
    let Some((artifact_id, _, _)) = vcf_execution_proof::report_contract(&manifest.stage_id) else {
        return Ok(Some(vcf_execution_proof::missing_adapter_blocker(
            &manifest.stage_id,
            &manifest.tool.id,
        )));
    };
    let outputs = manifest
        .outputs
        .iter()
        .filter(|output| output.artifact_id == artifact_id && output.exists)
        .collect::<Vec<_>>();
    ensure!(
        outputs.len() == 1,
        "VCF execution proof requires exactly one `{artifact_id}` report for `{}` / `{}`",
        manifest.stage_id,
        manifest.tool.id
    );
    let path = PathBuf::from(&outputs[0].realized_path);
    let path = if path.is_absolute() { path } else { repo_root.join(path) };
    let bytes = std::fs::read(&path)
        .with_context(|| format!("read VCF execution proof {}", path.display()))?;
    let report = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse VCF execution proof {}", path.display()))?;
    vcf_execution_proof::execution_blocker(&manifest.stage_id, &manifest.tool.id, &report)
}
