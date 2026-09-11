//! Project the public benchmark index while preserving native evidence and execution records.
use super::PublishedArtifactPaths;
use crate::contract::{validate_stage_result_manifest, BenchStageResultManifestV1};
use anyhow::{ensure, Result};
use std::path::Path;

pub(super) fn project_stable_stage_result(
    source: &Path,
    paths: &PublishedArtifactPaths<'_>,
) -> Result<BenchStageResultManifestV1> {
    let mut report: BenchStageResultManifestV1 =
        serde_json::from_slice(&super::paths::read_bounded(source)?)?;
    validate_stage_result_manifest(&report)?;
    for output in &mut report.outputs {
        let execution_path = paths.repository.join(&output.realized_path);
        ensure!(
            execution_path.starts_with(paths.execution),
            "benchmark stage output escapes its admitted execution"
        );
        super::paths::admit_relative(execution_path.strip_prefix(paths.execution)?)?;
        output.realized_path = paths.path(&execution_path)?;
    }
    validate_stage_result_manifest(&report)?;
    Ok(report)
}
