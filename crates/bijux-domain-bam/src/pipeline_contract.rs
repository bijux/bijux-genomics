//! BAM pipeline contract helpers.

use crate::{bam_stage_is_stable, BamStage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageCriticality {
    Essential,
    Optional,
    Experimental,
}

#[must_use]
pub fn canonical_stage_order() -> Vec<&'static str> {
    BamStage::all().iter().map(|stage| stage.as_str()).collect()
}

#[must_use]
pub fn stage_criticality(stage_id: &str) -> Option<StageCriticality> {
    let stage = BamStage::try_from(stage_id).ok()?;
    if bam_stage_is_stable(stage) {
        Some(StageCriticality::Essential)
    } else {
        Some(StageCriticality::Optional)
    }
}

#[must_use]
pub fn optional_branches() -> Vec<(&'static str, &'static [&'static str])> {
    Vec::new()
}

#[must_use]
pub fn forbidden_transitions() -> Vec<(&'static str, &'static str)> {
    Vec::new()
}
