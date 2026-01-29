pub mod filter;
pub mod merge;
pub mod trim;
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
    ]
}
