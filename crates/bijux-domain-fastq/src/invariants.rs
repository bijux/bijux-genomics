use crate::domain::FastqStageKind;

pub const CORE_STAGES: [&str; 6] = [
    "fastq.validate_pre",
    "fastq.trim",
    "fastq.merge",
    "fastq.correct",
    "fastq.filter",
    "fastq.stats_neutral",
];

pub const OPTIONAL_STAGES: [&str; 3] = ["fastq.qc_post", "fastq.umi", "fastq.screen"];

pub const META_STAGES: [&str; 1] = ["fastq.preprocess"];

pub const MUTATING_STAGES: [&str; 5] = [
    "fastq.trim",
    "fastq.merge",
    "fastq.correct",
    "fastq.filter",
    "fastq.umi",
];

pub const LOSSLESS_STAGES: [&str; 2] = ["fastq.validate_pre", "fastq.stats_neutral"];

pub const OBSERVATIONAL_STAGES: [&str; 4] = [
    "fastq.validate_pre",
    "fastq.stats_neutral",
    "fastq.qc_post",
    "fastq.screen",
];

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
