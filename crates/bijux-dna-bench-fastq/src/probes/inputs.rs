use anyhow::{anyhow, ensure, Result};
use std::path::PathBuf;
pub(super) fn required_input_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    artifact_name: &str,
) -> Result<PathBuf> {
    plan.io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!("plan `{}` is missing input `{artifact_name}`", plan.stage_id.as_str())
        })
}
pub(super) fn optional_input_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    artifact_name: &str,
) -> Option<PathBuf> {
    plan.io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_name)
        .map(|artifact| artifact.path.clone())
}
pub(super) fn admit_stage(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    expected: &str,
) -> Result<()> {
    ensure!(
        plan.stage_id.as_str() == expected,
        "FASTQ parser probe requires `{expected}`, found `{}`",
        plan.stage_id
    );
    Ok(())
}
