pub mod correct;
pub mod filter;
pub mod merge;
pub mod preprocess;
pub mod qc_post;
pub mod screen;
pub mod stats_neutral;
pub mod trim;
pub mod umi;
pub mod validate_pre;

use bijux_core::StageVersion;

#[derive(Debug, Clone)]
pub struct StageInfo {
    pub id: &'static str,
    pub version: StageVersion,
}

pub fn registry() -> Vec<StageInfo> {
    vec![
        StageInfo {
            id: correct::STAGE_ID,
            version: correct::STAGE_VERSION,
        },
        StageInfo {
            id: trim::STAGE_ID,
            version: trim::STAGE_VERSION,
        },
        StageInfo {
            id: validate_pre::STAGE_ID,
            version: validate_pre::STAGE_VERSION,
        },
        StageInfo {
            id: filter::STAGE_ID,
            version: filter::STAGE_VERSION,
        },
        StageInfo {
            id: merge::STAGE_ID,
            version: merge::STAGE_VERSION,
        },
        StageInfo {
            id: umi::STAGE_ID,
            version: umi::STAGE_VERSION,
        },
        StageInfo {
            id: screen::STAGE_ID,
            version: screen::STAGE_VERSION,
        },
        StageInfo {
            id: stats_neutral::STAGE_ID,
            version: stats_neutral::STAGE_VERSION,
        },
        StageInfo {
            id: preprocess::STAGE_ID,
            version: preprocess::STAGE_VERSION,
        },
        StageInfo {
            id: qc_post::STAGE_ID,
            version: qc_post::STAGE_VERSION,
        },
    ]
}
