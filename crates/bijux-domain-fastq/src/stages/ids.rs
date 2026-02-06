use bijux_core::ids::StageId;

pub const STAGE_VALIDATE_PRE: StageId = StageId::from_static("fastq.validate_pre");
pub const STAGE_DETECT_ADAPTERS: StageId = StageId::from_static("fastq.detect_adapters");
pub const STAGE_TRIM: StageId = StageId::from_static("fastq.trim");
pub const STAGE_FILTER: StageId = StageId::from_static("fastq.filter");
pub const STAGE_STATS_NEUTRAL: StageId = StageId::from_static("fastq.stats_neutral");
pub const STAGE_MERGE: StageId = StageId::from_static("fastq.merge");
pub const STAGE_CORRECT: StageId = StageId::from_static("fastq.correct");
pub const STAGE_QC_POST: StageId = StageId::from_static("fastq.qc_post");
pub const STAGE_UMI: StageId = StageId::from_static("fastq.umi");
pub const STAGE_SCREEN: StageId = StageId::from_static("fastq.screen");
pub const STAGE_PREPROCESS: StageId = StageId::from_static("fastq.preprocess");
pub const STAGE_RRNA: StageId = StageId::from_static("fastq.rrna");

pub const STAGE_PREFIX: &str = "fastq.";

pub const STAGES: [StageId; 11] = [
    STAGE_VALIDATE_PRE,
    STAGE_DETECT_ADAPTERS,
    STAGE_TRIM,
    STAGE_FILTER,
    STAGE_STATS_NEUTRAL,
    STAGE_MERGE,
    STAGE_CORRECT,
    STAGE_UMI,
    STAGE_SCREEN,
    STAGE_QC_POST,
    STAGE_PREPROCESS,
];

#[must_use]
pub fn bench_dir_name(stage: &StageId) -> Option<&'static str> {
    if stage == &STAGE_VALIDATE_PRE {
        Some("validate_pre")
    } else if stage == &STAGE_DETECT_ADAPTERS {
        Some("detect_adapters")
    } else if stage == &STAGE_TRIM {
        Some("trim")
    } else if stage == &STAGE_FILTER {
        Some("filter")
    } else if stage == &STAGE_STATS_NEUTRAL {
        Some("stats")
    } else if stage == &STAGE_MERGE {
        Some("merge")
    } else if stage == &STAGE_CORRECT {
        Some("correct")
    } else if stage == &STAGE_QC_POST {
        Some("qc_post")
    } else if stage == &STAGE_UMI {
        Some("umi")
    } else if stage == &STAGE_SCREEN {
        Some("screen")
    } else if stage == &STAGE_PREPROCESS {
        Some("preprocess")
    } else {
        None
    }
}
