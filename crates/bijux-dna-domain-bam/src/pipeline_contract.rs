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
    if matches!(stage, BamStage::BiasMitigation | BamStage::Genotyping) {
        Some(StageCriticality::Experimental)
    } else if bam_stage_is_stable(stage) {
        Some(StageCriticality::Essential)
    } else {
        Some(StageCriticality::Optional)
    }
}

#[must_use]
pub const fn optional_branches() -> &'static [(&'static str, &'static [&'static str])] {
    &[
        ("ancient_dna_authenticity", &["bam.damage", "bam.authenticity", "bam.contamination"]),
        ("sample_identity", &["bam.sex", "bam.haplogroups", "bam.kinship"]),
        ("variant_readiness", &["bam.recalibration", "bam.genotyping"]),
        ("bias_controls", &["bam.gc_bias", "bam.bias_mitigation"]),
    ]
}

#[must_use]
pub const fn forbidden_transitions() -> &'static [(&'static str, &'static str)] {
    &[
        ("bam.align", "bam.damage"),
        ("bam.align", "bam.authenticity"),
        ("bam.align", "bam.contamination"),
        ("bam.align", "bam.sex"),
        ("bam.align", "bam.genotyping"),
        ("bam.align", "bam.kinship"),
    ]
}
