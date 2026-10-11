use crate::bam::assignment::paths::repo_relative_path;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
pub(in crate::bam::assignment) fn ensure_paths_match(
    repo_root: &Path,
    stage_id: &str,
    field_name: &str,
    observed_path: &Path,
    expected_path: &Path,
) -> Result<()> {
    let observed = repo_relative_path(repo_root, observed_path)
        .canonicalize()
        .with_context(|| format!("canonicalize {}", observed_path.display()))?;
    let expected = expected_path
        .canonicalize()
        .with_context(|| format!("canonicalize {}", expected_path.display()))?;
    if observed != expected {
        return Err(anyhow!(
            "BAM stage `{stage_id}` field `{field_name}` path `{}` must match `{}`",
            observed_path.display(),
            expected_path.display()
        ));
    }
    Ok(())
}
pub(in crate::bam::assignment) fn ensure_float_matches(
    stage_id: &str,
    field_name: &str,
    observed: f64,
    expected: f64,
) -> Result<()> {
    if !observed.is_finite() || !expected.is_finite() || (observed - expected).abs() > f64::EPSILON
    {
        return Err(anyhow!(
            "BAM stage `{stage_id}` field `{field_name}` value `{observed}` must match `{expected}`"
        ));
    }
    Ok(())
}
