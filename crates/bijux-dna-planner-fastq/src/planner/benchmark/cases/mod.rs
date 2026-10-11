//! Governed FASTQ case planning, grouped by the stage that owns each contract.
use bijux_dna_core::prelude::ToolExecutionSpecV1;

pub(in crate::planner::benchmark::cases) fn hydrate_smoke_threads(
    tool_spec: &mut ToolExecutionSpecV1,
    threads: Option<u32>,
) {
    if let Some(threads) = threads {
        tool_spec.resources.threads = threads.max(1);
    } else {
        tool_spec.resources.threads = tool_spec.resources.threads.max(1);
    }
}

mod admission;
mod amplicon;
pub use amplicon::cluster_otus::{local_cluster_otus_smoke_plans, LocalClusterOtusSmokeCasePlan};
pub use amplicon::infer_asvs::{local_infer_asvs_smoke_plans, LocalInferAsvsSmokeCasePlan};
pub use amplicon::normalize_abundance::{
    local_normalize_abundance_smoke_plans, LocalNormalizeAbundanceSmokeCasePlan,
};
pub use amplicon::normalize_primers::{
    local_normalize_primers_smoke_plans, LocalNormalizePrimersSmokeCasePlan,
};
pub use amplicon::remove_chimeras::{
    local_remove_chimeras_smoke_plans, LocalRemoveChimerasSmokeCasePlan,
};
mod pre;
pub use pre::detect_adapters::{
    local_detect_adapters_smoke_plans, LocalDetectAdaptersSmokeCasePlan,
};
pub use pre::detect_duplicates_premerge::{
    local_detect_duplicates_premerge_smoke_plans, LocalDetectDuplicatesPremergeSmokeCasePlan,
};
pub use pre::estimate_library_complexity_prealign::{
    local_estimate_library_complexity_prealign_smoke_plans,
    LocalEstimateLibraryComplexityPrealignSmokeCasePlan,
};
pub use pre::profile_overrepresented_sequences::{
    local_profile_overrepresented_sequences_smoke_plans,
    LocalProfileOverrepresentedSequencesSmokeCasePlan,
};
pub use pre::profile_read_lengths::{
    local_profile_read_lengths_smoke_plans, LocalProfileReadLengthsSmokeCasePlan,
};
pub use pre::validate_reads::{local_validate_reads_smoke_plans, LocalValidateReadsSmokeCasePlan};
mod qc;
pub use qc::profile_reads::{
    local_profile_reads_output_contract_plans, local_profile_reads_smoke_plans,
    LocalProfileReadsSmokeCasePlan,
};
pub use qc::report_qc::local_report_qc_smoke_plan;
mod transform;
pub use transform::correct_errors::{
    local_correct_errors_smoke_plans, LocalCorrectErrorsSmokeCasePlan,
};
pub use transform::extract_umis::{local_extract_umis_smoke_plans, LocalExtractUmisSmokeCasePlan};
pub use transform::filter_low_complexity::{
    local_filter_low_complexity_smoke_plans, LocalFilterLowComplexitySmokeCasePlan,
};
pub use transform::filter_reads::{local_filter_reads_smoke_plans, LocalFilterReadsSmokeCasePlan};
pub use transform::merge_pairs::{
    local_merge_pairs_output_contract_plans, local_merge_pairs_smoke_plans,
    LocalMergePairsSmokeCasePlan,
};
pub use transform::remove_duplicates::{
    local_remove_duplicates_smoke_plans, LocalRemoveDuplicatesSmokeCasePlan,
};
pub use transform::trim_polyg_tails::{
    local_trim_polyg_tails_smoke_plans, LocalTrimPolygTailsSmokeCasePlan,
};
pub use transform::trim_reads::{
    local_trim_reads_output_contract_plans, local_trim_reads_smoke_plans,
    LocalTrimReadsSmokeCasePlan,
};
pub use transform::trim_terminal_damage::{
    local_trim_terminal_damage_output_contract_plans, local_trim_terminal_damage_smoke_plans,
    LocalTrimTerminalDamageSmokeCasePlan,
};
