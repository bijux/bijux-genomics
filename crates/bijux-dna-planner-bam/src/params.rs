use anyhow::Result;
use bijux_dna_domain_bam::params::BamEffectiveParams;
use bijux_dna_domain_bam::{stage_spec, BamStage};
use serde_json::Value;

/// # Errors
/// Returns an error if the provided params do not match the stage contract.
pub fn effective_params_for_stage(
    stage: BamStage,
    params: Option<&Value>,
) -> Result<BamEffectiveParams> {
    if let Some(value) = params {
        return stage.parse_effective_params(value);
    }
    Ok(stage_spec(stage).default_params.clone())
}
