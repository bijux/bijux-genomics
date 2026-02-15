use bijux_dna_core::prelude::{StageId, StageVersion};
use bijux_dna_domain_fastq::stages::ids as fastq_ids;

pub mod preprocess {
    pub use crate::tool_adapters::stages::pre::preprocess::*;
}

pub mod validate_pre {
    pub use crate::tool_adapters::stages::pre::validate_pre::*;
}

pub mod detect_adapters {
    pub use crate::tool_adapters::stages::pre::detect_adapters::*;
}

pub mod length_distribution_pre {
    pub use crate::tool_adapters::stages::pre::length_distribution_pre::*;
}

pub mod overrepresented_sequences {
    pub use crate::tool_adapters::stages::pre::overrepresented_sequences::*;
}

pub mod trim {
    pub use crate::tool_adapters::stages::transform::trim::*;
}

pub mod filter {
    pub use crate::tool_adapters::stages::transform::filter::*;
}

pub mod deduplicate {
    pub use crate::tool_adapters::stages::transform::deduplicate::*;
}

pub mod host_depletion {
    pub use crate::tool_adapters::stages::transform::host_depletion::*;
}

pub mod contaminant_screen {
    pub use crate::tool_adapters::stages::transform::contaminant_screen::*;
}

pub mod low_complexity {
    pub use crate::tool_adapters::stages::transform::low_complexity::*;
}

pub mod merge {
    pub use crate::tool_adapters::stages::transform::merge::*;
}

pub mod polyg_tailing {
    pub use crate::tool_adapters::stages::transform::polyg_tailing::*;
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

pub mod rrna {
    pub use crate::tool_adapters::stages::qc::rrna::*;
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
            id: crate::tool_adapters::stages::pre::length_distribution_pre::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::pre::length_distribution_pre::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::pre::overrepresented_sequences::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::pre::overrepresented_sequences::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::filter::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::filter::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::deduplicate::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::deduplicate::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::host_depletion::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::host_depletion::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::contaminant_screen::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::contaminant_screen::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::low_complexity::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::low_complexity::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::merge::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::merge::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::polyg_tailing::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::polyg_tailing::STAGE_VERSION,
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
            id: fastq_ids::STAGE_PRIMER_NORMALIZATION,
            version: StageVersion(1),
            affects_read_counts: true,
        },
        StageInfo {
            id: fastq_ids::STAGE_CHIMERA_DETECTION,
            version: StageVersion(1),
            affects_read_counts: true,
        },
        StageInfo {
            id: fastq_ids::STAGE_ASV_INFERENCE,
            version: StageVersion(1),
            affects_read_counts: false,
        },
        StageInfo {
            id: fastq_ids::STAGE_OTU_CLUSTERING,
            version: StageVersion(1),
            affects_read_counts: false,
        },
        StageInfo {
            id: fastq_ids::STAGE_ABUNDANCE_NORMALIZATION,
            version: StageVersion(1),
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
        StageInfo {
            id: crate::tool_adapters::stages::qc::rrna::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::qc::rrna::STAGE_VERSION,
            affects_read_counts: false,
        },
    ]
}
