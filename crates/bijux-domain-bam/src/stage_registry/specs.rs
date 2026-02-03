#[path = "stage_specs_core.rs"]
mod stage_specs_core;
#[path = "stage_specs_downstream.rs"]
mod stage_specs_downstream;

use super::{BamStage, BamStageSpec};

pub use stage_specs_core::required_audit_artifacts;

/// # Panics
/// Panics if the stage registry is missing a spec for the requested stage.
#[must_use]
pub fn stage_spec(stage: BamStage) -> BamStageSpec {
    stage_specs_core::stage_spec_core(stage)
        .or_else(|| stage_specs_downstream::stage_spec_downstream(stage))
        .unwrap_or_else(|| panic!("missing stage spec for {}", stage.as_str()))
}

#[must_use]
pub fn stage_specs() -> Vec<BamStageSpec> {
    BamStage::all()
        .iter()
        .map(|stage| stage_spec(*stage))
        .collect()
}
