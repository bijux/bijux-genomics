pub mod correct;
pub mod detect_adapters;
pub mod filter;
pub mod merge;
pub mod preprocess;
pub mod qc_post;
#[path = "qc/screen.rs"]
pub mod screen;
#[path = "qc/stats_neutral.rs"]
pub mod stats_neutral;
pub mod trim;
pub mod umi;
pub mod validate_pre;

use bijux_core::StageVersion;

#[derive(Debug, Clone)]
pub struct StageInfo {
    pub id: &'static str,
    pub version: StageVersion,
    pub affects_read_counts: bool,
}

pub fn registry() -> Vec<StageInfo> {
    vec![
        StageInfo {
            id: correct::STAGE_ID,
            version: correct::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: trim::STAGE_ID,
            version: trim::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: validate_pre::STAGE_ID,
            version: validate_pre::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: detect_adapters::STAGE_ID,
            version: detect_adapters::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: filter::STAGE_ID,
            version: filter::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: merge::STAGE_ID,
            version: merge::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: umi::STAGE_ID,
            version: umi::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: screen::STAGE_ID,
            version: screen::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: stats_neutral::STAGE_ID,
            version: stats_neutral::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: preprocess::STAGE_ID,
            version: preprocess::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: qc_post::STAGE_ID,
            version: qc_post::STAGE_VERSION,
            affects_read_counts: false,
        },
    ]
}
