//! Owner: observe the current required micro-run component and result files.
use anyhow::{bail, Result};
use bijux_dna_bench_model::{
    MicroBenchmarkExecutionStatus, MicroBenchmarkRunEvidence, MicroBenchmarkRunManifest,
};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

pub(crate) fn present_files(
    root: &Path,
    manifest: &MicroBenchmarkRunManifest,
    evidence: MicroBenchmarkRunEvidence<'_>,
) -> BTreeSet<String> {
    manifest
        .component_reports
        .iter()
        .map(|row| row.report_path.as_str())
        .chain(
            evidence
                .results
                .iter()
                .filter(|row| row.status == MicroBenchmarkExecutionStatus::Succeeded)
                .filter_map(|row| row.evidence_path.as_deref()),
        )
        .filter(|path| artifact_path(root, path).is_ok_and(|path| path.is_file()))
        .map(str::to_string)
        .collect()
}

/// Re-observe declared output and log presence before assigning a current run verdict.
pub(crate) fn require_current_artifacts(
    root: &Path,
    evidence: MicroBenchmarkRunEvidence<'_>,
) -> Result<()> {
    for row in evidence.outputs {
        if !artifact_path(root, &row.path)?.exists() {
            bail!(
                "micro benchmark output `{}` is no longer present at `{}`",
                row.execution_id,
                row.path
            );
        }
    }
    for row in evidence.logs {
        if !artifact_path(root, &row.path)?.is_file() {
            bail!(
                "micro benchmark log `{}` is not a current file at `{}`",
                row.execution_id,
                row.path
            );
        }
    }
    Ok(())
}

fn artifact_path(root: &Path, value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('\\')
        || !path.components().all(|part| matches!(part, Component::Normal(_)))
    {
        bail!("micro benchmark evidence path must be repository-relative without traversal: `{value}`");
    }
    Ok(root.join(path))
}
