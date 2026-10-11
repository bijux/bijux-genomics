use super::declarations::EXPECTED_CORRECTED_OVERLAP_BASES;
use super::declarations::EXPECTED_CORRECTED_PAIR_COUNT;
use super::declarations::EXPECTED_PAIR_COUNT;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_STAGE_METRICS_SCHEMA_VERSION;
use super::declarations::EXPECTED_SUMMARY_SCHEMA_VERSION;
use super::declarations::EXPECTED_TOOL_ID;
use super::models::LocalOverlapCorrectionSmokeMetrics;
use super::observations::LocalOverlapCorrectionSmokeReport;
pub(super) fn check_summary_ready(
    summary: &bijux_dna_domain_bam::BamOverlapCorrectionSummaryV1,
) -> bool {
    summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && summary.stage_id == EXPECTED_STAGE_ID
        && summary.method == EXPECTED_TOOL_ID
        && summary.pair_count == Some(EXPECTED_PAIR_COUNT)
        && summary.corrected_pairs == Some(EXPECTED_CORRECTED_PAIR_COUNT)
        && summary.corrected_overlap_bases == Some(EXPECTED_CORRECTED_OVERLAP_BASES)
        && summary.insufficiency_reason.is_none()
}

pub(super) fn check_normalized_metrics_ready(
    local_smoke_report: &LocalOverlapCorrectionSmokeReport,
    normalized_metrics: &LocalOverlapCorrectionSmokeMetrics,
) -> bool {
    normalized_metrics.schema_version == EXPECTED_STAGE_METRICS_SCHEMA_VERSION
        && normalized_metrics.stage_id == EXPECTED_STAGE_ID
        && normalized_metrics.sample_id == local_smoke_report.sample_id
        && normalized_metrics.method == EXPECTED_TOOL_ID
        && normalized_metrics.expected_pair_count == EXPECTED_PAIR_COUNT
        && normalized_metrics.pair_count == EXPECTED_PAIR_COUNT
        && normalized_metrics.pair_count_delta == 0
        && normalized_metrics.expected_corrected_pairs == EXPECTED_CORRECTED_PAIR_COUNT
        && normalized_metrics.corrected_pairs == EXPECTED_CORRECTED_PAIR_COUNT
        && normalized_metrics.corrected_pair_delta == 0
        && normalized_metrics.expected_corrected_overlap_bases == EXPECTED_CORRECTED_OVERLAP_BASES
        && normalized_metrics.corrected_overlap_bases == EXPECTED_CORRECTED_OVERLAP_BASES
        && normalized_metrics.corrected_overlap_base_delta == 0
        && normalized_metrics.expectation_matched
        && normalized_metrics.insufficiency_reason.is_none()
}

pub(super) fn check_corrected_overlap_metrics_ready(
    local_smoke_report: &LocalOverlapCorrectionSmokeReport,
    summary: &bijux_dna_domain_bam::BamOverlapCorrectionSummaryV1,
    normalized_metrics: &LocalOverlapCorrectionSmokeMetrics,
) -> bool {
    local_smoke_report.expectation_matched
        && local_smoke_report.method == EXPECTED_TOOL_ID
        && local_smoke_report.pair_count == EXPECTED_PAIR_COUNT
        && local_smoke_report.corrected_pairs == EXPECTED_CORRECTED_PAIR_COUNT
        && local_smoke_report.corrected_overlap_bases == EXPECTED_CORRECTED_OVERLAP_BASES
        && local_smoke_report.insufficiency_reason.is_none()
        && summary.pair_count == Some(normalized_metrics.pair_count)
        && summary.corrected_pairs == Some(normalized_metrics.corrected_pairs)
        && summary.corrected_overlap_bases == Some(normalized_metrics.corrected_overlap_bases)
}
