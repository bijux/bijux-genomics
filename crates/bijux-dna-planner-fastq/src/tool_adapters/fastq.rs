use bijux_dna_core::prelude::{StageId, StageVersion};
use bijux_dna_domain_fastq::stages::ids as fastq_ids;
use bijux_dna_domain_fastq::ExecutionStatus;

pub use crate::tool_adapters::stages::amplicon::cluster_otus;
pub use crate::tool_adapters::stages::amplicon::infer_asvs;
pub use crate::tool_adapters::stages::amplicon::normalize_abundance;
pub use crate::tool_adapters::stages::amplicon::normalize_primers;
pub use crate::tool_adapters::stages::amplicon::remove_chimeras;
pub use crate::tool_adapters::stages::pre::detect_adapters;
pub use crate::tool_adapters::stages::pre::index_reference;
pub use crate::tool_adapters::stages::pre::profile_overrepresented_sequences;
pub use crate::tool_adapters::stages::pre::profile_read_lengths;
pub use crate::tool_adapters::stages::pre::validate_reads;
pub use crate::tool_adapters::stages::qc::deplete_rrna;
pub use crate::tool_adapters::stages::qc::profile_reads;
pub use crate::tool_adapters::stages::qc::report_qc;
pub use crate::tool_adapters::stages::qc::screen_taxonomy;
pub use crate::tool_adapters::stages::transform::correct_errors;
pub use crate::tool_adapters::stages::transform::deplete_host;
pub use crate::tool_adapters::stages::transform::deplete_reference_contaminants;
pub use crate::tool_adapters::stages::transform::extract_umis;
pub use crate::tool_adapters::stages::transform::filter_low_complexity;
pub use crate::tool_adapters::stages::transform::filter_reads;
pub use crate::tool_adapters::stages::transform::merge_pairs;
pub use crate::tool_adapters::stages::transform::remove_duplicates;
pub use crate::tool_adapters::stages::transform::trim_polyg_tails;
pub use crate::tool_adapters::stages::transform::trim_reads;
pub use crate::tool_adapters::stages::transform::trim_terminal_damage;

#[derive(Debug, Clone)]
pub struct StageInfo {
    id: StageId,
    version: StageVersion,
    affects_read_counts: bool,
}

impl StageInfo {
    #[must_use]
    pub fn id(&self) -> &StageId {
        &self.id
    }

    #[must_use]
    pub fn version(&self) -> StageVersion {
        self.version
    }

    #[must_use]
    pub fn affects_read_counts(&self) -> bool {
        self.affects_read_counts
    }
}

#[must_use]
pub fn registry() -> Vec<StageInfo> {
    let stages = vec![
        StageInfo {
            id: crate::tool_adapters::stages::pre::index_reference::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::pre::index_reference::STAGE_VERSION,
            affects_read_counts: false,
        },
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
            id: crate::tool_adapters::stages::transform::trim_terminal_damage::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::transform::trim_terminal_damage::STAGE_VERSION,
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
            id: crate::tool_adapters::stages::amplicon::normalize_primers::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::amplicon::normalize_primers::STAGE_VERSION,
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::amplicon::infer_asvs::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::amplicon::infer_asvs::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: fastq_ids::STAGE_REMOVE_CHIMERAS,
            version: StageVersion(1),
            affects_read_counts: true,
        },
        StageInfo {
            id: crate::tool_adapters::stages::amplicon::cluster_otus::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::amplicon::cluster_otus::STAGE_VERSION,
            affects_read_counts: false,
        },
        StageInfo {
            id: crate::tool_adapters::stages::amplicon::normalize_abundance::STAGE_ID.clone(),
            version: crate::tool_adapters::stages::amplicon::normalize_abundance::STAGE_VERSION,
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
    ];
    stages
        .into_iter()
        .filter(|stage| {
            matches!(
                bijux_dna_domain_fastq::execution_support_for_stage(stage.id()),
                Some(support) if support.execution_status == ExecutionStatus::Closed
            )
        })
        .collect()
}
