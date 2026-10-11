//! Governed BAM case planning, grouped by the stage that owns each contract.
use bijux_dna_core::prelude::ToolExecutionSpecV1;

pub(in crate::benchmark::cases) fn hydrate_smoke_threads(
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
mod pre;
pub use pre::filter::{local_filter_smoke_plans, LocalFilterSmokeCasePlan};
pub use pre::length_filter::{local_length_filter_smoke_plans, LocalLengthFilterSmokeCasePlan};
pub use pre::mapping_summary::{
    local_mapping_summary_smoke_plans, LocalMappingSummarySmokeCasePlan,
};
pub use pre::mapq_filter::{local_mapq_filter_smoke_plans, LocalMapqFilterSmokeCasePlan};
pub use pre::overlap_correction::{
    local_overlap_correction_smoke_plans, LocalOverlapCorrectionSmokeCasePlan,
};
pub use pre::qc_pre::{local_qc_pre_smoke_plans, LocalQcPreSmokeCasePlan};
pub use pre::validate::{
    local_validate_smoke_plans, LocalValidateAlignmentFixtureEncoding, LocalValidateSmokeCasePlan,
};
mod qc;
pub use qc::complexity::{local_complexity_smoke_plans, LocalComplexitySmokeCasePlan};
pub use qc::coverage::{
    local_coverage_smoke_plans, LocalCoverageSmokeCasePlan, LocalCoverageSmokeExpectedRow,
};
pub use qc::duplication_metrics::{
    local_duplication_metrics_smoke_plans, LocalDuplicationMetricsSmokeCasePlan,
};
pub use qc::endogenous_content::{
    local_endogenous_content_smoke_plans, LocalEndogenousContentSmokeCasePlan,
};
pub use qc::gc_bias::{
    local_gc_bias_smoke_plans, LocalGcBiasSmokeCasePlan, LocalGcBiasSmokeExpectedRow,
};
pub use qc::insert_size::{local_insert_size_smoke_plans, LocalInsertSizeSmokeCasePlan};
pub use qc::markdup::{local_markdup_smoke_plans, LocalMarkdupSmokeCasePlan};
pub use qc::recalibration::{local_recalibration_smoke_plans, LocalRecalibrationSmokeCasePlan};
mod ancient;
pub use ancient::authenticity::{
    local_authenticity_output_contract_plans, local_authenticity_smoke_plans,
    LocalAuthenticitySmokeCasePlan,
};
pub use ancient::damage::{
    local_damage_output_contract_plans, local_damage_smoke_plans, LocalDamageSmokeCasePlan,
};
pub use ancient::sex::{
    local_sex_output_contract_plans, local_sex_smoke_plans, LocalSexSmokeCasePlan,
};
mod downstream;
#[cfg(feature = "bam_downstream")]
pub use downstream::bias_mitigation::{
    local_bias_mitigation_smoke_plans, LocalBiasMitigationSmokeCasePlan,
};
#[cfg(feature = "bam_downstream")]
pub use downstream::kinship::{
    local_kinship_output_contract_plans, local_kinship_smoke_plans, LocalKinshipSmokeCasePlan,
    LocalKinshipSmokeExpectedPair,
};
