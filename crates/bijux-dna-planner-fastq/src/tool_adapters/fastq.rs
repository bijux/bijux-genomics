use bijux_dna_core::prelude::{StageId, StageVersion};
use bijux_dna_domain_fastq::stages::ids as fastq_ids;
use bijux_dna_domain_fastq::ExecutionStatus;

pub use crate::tool_adapters::stages::amplicon::cluster_otus;
pub use crate::tool_adapters::stages::amplicon::infer_asvs;
pub use crate::tool_adapters::stages::amplicon::normalize_abundance;
pub use crate::tool_adapters::stages::amplicon::normalize_primers;
pub use crate::tool_adapters::stages::amplicon::remove_chimeras;
pub use crate::tool_adapters::stages::pre::detect_adapters;
pub use crate::tool_adapters::stages::pre::detect_duplicates_premerge;
pub use crate::tool_adapters::stages::pre::estimate_library_complexity_prealign;
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
    let mut stages = pre_registry();
    stages.extend(transform_registry());
    stages.extend(qc_registry());
    stages.extend(amplicon_registry());
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

fn stage_info(id: StageId, version: StageVersion, affects_read_counts: bool) -> StageInfo {
    StageInfo { id, version, affects_read_counts }
}

fn pre_registry() -> Vec<StageInfo> {
    vec![
        stage_info(index_reference::STAGE_ID.clone(), index_reference::STAGE_VERSION, false),
        stage_info(validate_reads::STAGE_ID.clone(), validate_reads::STAGE_VERSION, false),
        stage_info(detect_adapters::STAGE_ID.clone(), detect_adapters::STAGE_VERSION, false),
        stage_info(
            profile_read_lengths::STAGE_ID.clone(),
            profile_read_lengths::STAGE_VERSION,
            false,
        ),
        stage_info(
            profile_overrepresented_sequences::STAGE_ID.clone(),
            profile_overrepresented_sequences::STAGE_VERSION,
            false,
        ),
    ]
}

fn transform_registry() -> Vec<StageInfo> {
    vec![
        stage_info(correct_errors::STAGE_ID.clone(), correct_errors::STAGE_VERSION, true),
        stage_info(trim_reads::STAGE_ID.clone(), trim_reads::STAGE_VERSION, true),
        stage_info(filter_reads::STAGE_ID.clone(), filter_reads::STAGE_VERSION, true),
        stage_info(remove_duplicates::STAGE_ID.clone(), remove_duplicates::STAGE_VERSION, true),
        stage_info(deplete_host::STAGE_ID.clone(), deplete_host::STAGE_VERSION, true),
        stage_info(
            deplete_reference_contaminants::STAGE_ID.clone(),
            deplete_reference_contaminants::STAGE_VERSION,
            true,
        ),
        stage_info(
            filter_low_complexity::STAGE_ID.clone(),
            filter_low_complexity::STAGE_VERSION,
            true,
        ),
        stage_info(merge_pairs::STAGE_ID.clone(), merge_pairs::STAGE_VERSION, true),
        stage_info(trim_polyg_tails::STAGE_ID.clone(), trim_polyg_tails::STAGE_VERSION, true),
        stage_info(
            trim_terminal_damage::STAGE_ID.clone(),
            trim_terminal_damage::STAGE_VERSION,
            true,
        ),
        stage_info(extract_umis::STAGE_ID.clone(), extract_umis::STAGE_VERSION, true),
    ]
}

fn qc_registry() -> Vec<StageInfo> {
    vec![
        stage_info(screen_taxonomy::STAGE_ID.clone(), screen_taxonomy::STAGE_VERSION, false),
        stage_info(profile_reads::STAGE_ID.clone(), profile_reads::STAGE_VERSION, false),
        stage_info(report_qc::STAGE_ID.clone(), report_qc::STAGE_VERSION, false),
        stage_info(deplete_rrna::STAGE_ID.clone(), deplete_rrna::STAGE_VERSION, true),
    ]
}

fn amplicon_registry() -> Vec<StageInfo> {
    vec![
        stage_info(normalize_primers::STAGE_ID.clone(), normalize_primers::STAGE_VERSION, true),
        stage_info(infer_asvs::STAGE_ID.clone(), infer_asvs::STAGE_VERSION, false),
        stage_info(fastq_ids::STAGE_REMOVE_CHIMERAS, StageVersion(1), true),
        stage_info(cluster_otus::STAGE_ID.clone(), cluster_otus::STAGE_VERSION, false),
        stage_info(
            normalize_abundance::STAGE_ID.clone(),
            normalize_abundance::STAGE_VERSION,
            false,
        ),
    ]
}
