#![allow(dead_code)]

use bijux_macros::fastq_v1_invariant;

use crate::domain::FastqStageKind;

#[fastq_v1_invariant]
pub const CORE_STAGES: [&str; 6] = [
    "fastq.validate_pre",
    "fastq.trim",
    "fastq.merge",
    "fastq.correct",
    "fastq.filter",
    "fastq.stats_neutral",
];

#[fastq_v1_invariant]
pub const OPTIONAL_STAGES: [&str; 3] = ["fastq.qc_post", "fastq.umi", "fastq.screen"];

#[fastq_v1_invariant]
pub const META_STAGES: [&str; 1] = ["fastq.preprocess"];

#[fastq_v1_invariant]
pub const MUTATING_STAGES: [&str; 5] = [
    "fastq.trim",
    "fastq.merge",
    "fastq.correct",
    "fastq.filter",
    "fastq.umi",
];

#[fastq_v1_invariant]
pub const LOSSLESS_STAGES: [&str; 2] = ["fastq.validate_pre", "fastq.stats_neutral"];

#[fastq_v1_invariant]
pub const OBSERVATIONAL_STAGES: [&str; 4] = [
    "fastq.validate_pre",
    "fastq.stats_neutral",
    "fastq.qc_post",
    "fastq.screen",
];

#[fastq_v1_invariant]
#[must_use]
pub fn stage_kind(stage_id: &str) -> Option<FastqStageKind> {
    if CORE_STAGES.contains(&stage_id) {
        return Some(FastqStageKind::Core);
    }
    if OPTIONAL_STAGES.contains(&stage_id) {
        return Some(FastqStageKind::Optional);
    }
    if META_STAGES.contains(&stage_id) {
        return Some(FastqStageKind::Meta);
    }
    None
}
