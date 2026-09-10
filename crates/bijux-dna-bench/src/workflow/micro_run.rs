//! Owner: qualify micro-run declarations against current repository evidence.
use anyhow::Result;
use bijux_dna_bench_model::{
    validate_micro_benchmark_run_contract, MicroBenchmarkRunEvidence, MicroBenchmarkRunManifest,
};
use std::path::Path;

/// Record a passing verdict only after current file and model admission succeeds.
///
/// # Errors
/// Rejects missing evidence or invalid row contracts; clears any previous passing verdict.
pub fn qualify_micro_benchmark_run(
    root: &Path,
    manifest: &mut MicroBenchmarkRunManifest,
    evidence: MicroBenchmarkRunEvidence<'_>,
) -> Result<()> {
    manifest.passes_behavior_test = false;
    let present = crate::repo::micro_run::present_files(root, manifest, evidence);
    validate_micro_benchmark_run_contract(manifest, evidence, &present)?;
    crate::repo::micro_run::require_current_artifacts(root, evidence)?;
    manifest.passes_behavior_test = true;
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/contracts/micro_run.rs"]
mod tests;
