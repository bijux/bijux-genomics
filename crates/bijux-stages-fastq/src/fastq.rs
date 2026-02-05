use bijux_core::contract::StageVersion;

pub mod preprocess {
    pub use crate::stages_pre::preprocess::*;
}

pub mod validate_pre {
    pub use crate::stages_pre::validate_pre::*;
}

pub mod detect_adapters {
    pub use crate::stages_pre::detect_adapters::*;
}

pub mod trim {
    pub use crate::stages_transform::trim::*;
}

pub mod filter {
    pub use crate::stages_transform::filter::*;
}

pub mod merge {
    pub use crate::stages_transform::merge::*;
}

pub mod correct {
    pub use crate::stages_transform::correct::*;
}

pub mod umi {
    pub use crate::stages_transform::umi::*;
}

pub mod stats_neutral {
    pub use crate::stages_qc::stats_neutral::*;
}

pub mod qc_post {
    pub use crate::stages_qc::qc_post::*;
}

pub mod screen {
    pub use crate::stages_qc::screen::*;
}

#[derive(Debug, Clone)]
pub struct StageInfo {
    pub id: &'static str,
    pub version: StageVersion,
    pub affects_read_counts: bool,
}

pub fn registry() -> Vec<StageInfo> {
    vec![
        StageInfo {
            id: crate::stages_transform::correct::STAGE_ID,
            version: crate::stages_transform::correct::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::stages_transform::trim::STAGE_ID,
            version: crate::stages_transform::trim::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::stages_pre::validate_pre::STAGE_ID,
            version: crate::stages_pre::validate_pre::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::stages_pre::detect_adapters::STAGE_ID,
            version: crate::stages_pre::detect_adapters::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::stages_transform::filter::STAGE_ID,
            version: crate::stages_transform::filter::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::stages_transform::merge::STAGE_ID,
            version: crate::stages_transform::merge::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::stages_transform::umi::STAGE_ID,
            version: crate::stages_transform::umi::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::stages_qc::screen::STAGE_ID,
            version: crate::stages_qc::screen::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::stages_qc::stats_neutral::STAGE_ID,
            version: crate::stages_qc::stats_neutral::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::stages_pre::preprocess::STAGE_ID,
            version: crate::stages_pre::preprocess::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::stages_qc::qc_post::STAGE_ID,
            version: crate::stages_qc::qc_post::STAGE_VERSION,
            affects_read_counts: false,
        },
    ]
}
