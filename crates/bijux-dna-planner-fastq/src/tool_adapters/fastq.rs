use bijux_dna_core::prelude::{StageId, StageVersion};
use bijux_dna_domain_fastq::stages::ids as fastq_ids;

pub mod validate_reads {
    pub use crate::tool_adapters::stages::pre::validate_reads::*;
}

pub mod detect_adapters {
    pub use crate::tool_adapters::stages::pre::detect_adapters::*;
}

pub mod profile_read_lengths {
    pub use crate::tool_adapters::stages::pre::profile_read_lengths::*;
}

pub mod profile_overrepresented_sequences {
    pub use crate::tool_adapters::stages::pre::profile_overrepresented_sequences::*;
}

pub mod trim_reads {
    pub use crate::tool_adapters::stages::transform::trim_reads::*;
}

pub mod filter_reads {
    pub use crate::tool_adapters::stages::transform::filter_reads::*;
}

pub mod remove_duplicates {
    pub use crate::tool_adapters::stages::transform::remove_duplicates::*;
}

pub mod deplete_host {
    pub use crate::tool_adapters::stages::transform::deplete_host::*;
}

pub mod deplete_reference_contaminants {
    pub use crate::tool_adapters::stages::transform::deplete_reference_contaminants::*;
}

pub mod filter_low_complexity {
    pub use crate::tool_adapters::stages::transform::filter_low_complexity::*;
}

pub mod merge_pairs {
    pub use crate::tool_adapters::stages::transform::merge_pairs::*;
}

pub mod trim_polyg_tails {
    pub use crate::tool_adapters::stages::transform::trim_polyg_tails::*;
}

pub mod correct_errors {
    pub use crate::tool_adapters::stages::transform::correct_errors::*;
}

pub mod extract_umis {
    pub use crate::tool_adapters::stages::transform::extract_umis::*;
}

pub mod profile_reads {
    pub use crate::tool_adapters::stages::qc::profile_reads::*;
}

pub mod report_qc {
    pub use crate::tool_adapters::stages::qc::report_qc::*;
}

pub mod deplete_rrna {
    pub use crate::tool_adapters::stages::qc::deplete_rrna::*;
}

pub mod screen_taxonomy {
    pub use crate::tool_adapters::stages::qc::screen_taxonomy::*;
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
            id: crate::tool_adapters::stages::transform::correct_errors::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::correct_errors::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::trim_reads::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::trim_reads::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::pre::validate_reads::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::pre::validate_reads::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::pre::detect_adapters::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::pre::detect_adapters::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::pre::profile_read_lengths::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::pre::profile_read_lengths::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::pre::profile_overrepresented_sequences::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::pre::profile_overrepresented_sequences::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::filter_reads::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::filter_reads::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::remove_duplicates::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::remove_duplicates::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::deplete_host::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::deplete_host::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::deplete_reference_contaminants::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::deplete_reference_contaminants::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::filter_low_complexity::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::filter_low_complexity::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::merge_pairs::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::merge_pairs::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::trim_polyg_tails::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::trim_polyg_tails::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::transform::extract_umis::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::extract_umis::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::qc::screen_taxonomy::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::qc::screen_taxonomy::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::qc::profile_reads::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::qc::profile_reads::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: fastq_ids::STAGE_NORMALIZE_PRIMERS,
            version: StageVersion(1),
            affects_read_counts: true,
        },
        StageInfo {
            id: fastq_ids::STAGE_TRIM_TERMINAL_DAMAGE,
            version: StageVersion(1),
            affects_read_counts: true,
        },
        StageInfo {
            id: fastq_ids::STAGE_REMOVE_CHIMERAS,
            version: StageVersion(1),
            affects_read_counts: true,
        },
        StageInfo {
            id: fastq_ids::STAGE_INFER_ASVS,
            version: StageVersion(1),
            affects_read_counts: false,
        },
        StageInfo {
            id: fastq_ids::STAGE_CLUSTER_OTUS,
            version: StageVersion(1),
            affects_read_counts: false,
        },
        StageInfo {
            id: fastq_ids::STAGE_NORMALIZE_ABUNDANCE,
            version: StageVersion(1),
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::qc::report_qc::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::qc::report_qc::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::qc::deplete_rrna::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::qc::deplete_rrna::STAGE_VERSION,
            affects_read_counts: true,
        },
    ]
}
