use bijux_core::prelude::{StageId, StageVersion};

pub mod preprocess {
    pub use crate::tool_adapters::stages::pre::preprocess::*;
}

pub mod validate_pre {
    pub use crate::tool_adapters::stages::pre::validate_pre::*;
}

pub mod detect_adapters {
    pub use crate::tool_adapters::stages::pre::detect_adapters::*;
}

pub mod trim {
    pub use crate::tool_adapters::stages::transform::trim::*;
}

pub mod filter {
    pub use crate::tool_adapters::stages::transform::filter::*;
}

pub mod merge {
    pub use crate::tool_adapters::stages::transform::merge::*;
}

pub mod correct {
    pub use crate::tool_adapters::stages::transform::correct::*;
}

pub mod umi {
    pub use crate::tool_adapters::stages::transform::umi::*;
}

pub mod stats_neutral {
    pub use crate::tool_adapters::stages::qc::stats_neutral::*;
}

pub mod qc_post {
    pub use crate::tool_adapters::stages::qc::qc_post::*;
}

pub mod screen {
    pub use crate::tool_adapters::stages::qc::screen::*;
}

#[derive(Debug, Clone)]
pub struct StageInfo {
    pub id: StageId,
    pub version: StageVersion,
    pub affects_read_counts: bool,
}

pub fn registry() -> Vec<StageInfo> {
    vec![
        StageInfo {
            id: crate::tool_adapters::stages::transform::correct::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::correct::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::trim::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::trim::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::pre::validate_pre::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::pre::validate_pre::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::pre::detect_adapters::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::pre::detect_adapters::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::filter::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::filter::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::merge::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::merge::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::umi::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::umi::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::qc::screen::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::qc::screen::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::qc::stats_neutral::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::qc::stats_neutral::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::pre::preprocess::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::pre::preprocess::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::qc::qc_post::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::qc::qc_post::STAGE_VERSION,
            affects_read_counts: false,
        },
    ]
}
