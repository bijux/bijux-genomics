use super::contracts::float_matches;
use super::declarations::EXPECTED_ENDOGENOUS_FRACTION;
use super::declarations::EXPECTED_ENDOGENOUS_READS;
use super::declarations::EXPECTED_HOST_REFERENCE_SCOPE;
use super::declarations::EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION;
use super::declarations::EXPECTED_MAPPED_READS;
use super::declarations::EXPECTED_METHOD;
use super::declarations::EXPECTED_SAMPLE_ID;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_STAGE_METRICS_SCHEMA_VERSION;
use super::declarations::EXPECTED_SUMMARY_SCHEMA_VERSION;
use super::declarations::EXPECTED_TOTAL_READS;
use super::models::LocalEndogenousContentSmokeMetrics;
use super::observations::LocalEndogenousContentSmokeReport;
pub(super) fn check_summary_ready(
    summary: &bijux_dna_domain_bam::BamEndogenousContentEstimateV1,
) -> bool {
    summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && summary.stage_id == EXPECTED_STAGE_ID
        && summary.method == EXPECTED_METHOD
        && summary.mapped_reads == EXPECTED_MAPPED_READS
        && summary.endogenous_reads == EXPECTED_ENDOGENOUS_READS
        && summary.total_reads == EXPECTED_TOTAL_READS
        && float_matches(summary.endogenous_fraction, EXPECTED_ENDOGENOUS_FRACTION)
        && summary.prealignment_fraction.is_none()
        && float_matches(summary.postalignment_fraction, EXPECTED_ENDOGENOUS_FRACTION)
        && summary.host_reference_scope.as_deref() == Some(EXPECTED_HOST_REFERENCE_SCOPE)
        && !summary.caveats.is_empty()
}

pub(super) fn check_normalized_metrics_ready(
    normalized_metrics: &LocalEndogenousContentSmokeMetrics,
) -> bool {
    normalized_metrics.schema_version == EXPECTED_STAGE_METRICS_SCHEMA_VERSION
        && normalized_metrics.stage_id == EXPECTED_STAGE_ID
        && normalized_metrics.sample_id == EXPECTED_SAMPLE_ID
        && normalized_metrics.expected_method == EXPECTED_METHOD
        && normalized_metrics.method == EXPECTED_METHOD
        && normalized_metrics.expected_host_reference_scope == EXPECTED_HOST_REFERENCE_SCOPE
        && normalized_metrics.host_reference_scope.as_deref() == Some(EXPECTED_HOST_REFERENCE_SCOPE)
        && normalized_metrics.expected_mapped_reads == EXPECTED_MAPPED_READS
        && normalized_metrics.mapped_reads == EXPECTED_MAPPED_READS
        && normalized_metrics.mapped_read_delta == 0
        && normalized_metrics.expected_endogenous_reads == EXPECTED_ENDOGENOUS_READS
        && normalized_metrics.endogenous_reads == EXPECTED_ENDOGENOUS_READS
        && normalized_metrics.expected_total_reads == EXPECTED_TOTAL_READS
        && normalized_metrics.total_reads == EXPECTED_TOTAL_READS
        && normalized_metrics.total_read_delta == 0
        && float_matches(
            normalized_metrics.expected_endogenous_fraction,
            EXPECTED_ENDOGENOUS_FRACTION,
        )
        && float_matches(normalized_metrics.endogenous_fraction, EXPECTED_ENDOGENOUS_FRACTION)
        && float_matches(normalized_metrics.endogenous_fraction_delta, 0.0)
        && normalized_metrics.prealignment_fraction.is_none()
        && normalized_metrics.expectation_matched
}

pub(super) fn check_endogenous_metric_consistency_ready(
    local_smoke_report: &LocalEndogenousContentSmokeReport,
    summary: &bijux_dna_domain_bam::BamEndogenousContentEstimateV1,
    normalized_metrics: &LocalEndogenousContentSmokeMetrics,
) -> bool {
    local_smoke_report.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && local_smoke_report.stage_id == EXPECTED_STAGE_ID
        && local_smoke_report.sample_id == EXPECTED_SAMPLE_ID
        && local_smoke_report.expectation_matched
        && local_smoke_report.method == EXPECTED_METHOD
        && local_smoke_report.host_reference_scope.as_deref() == Some(EXPECTED_HOST_REFERENCE_SCOPE)
        && local_smoke_report.mapped_reads == EXPECTED_MAPPED_READS
        && local_smoke_report.endogenous_reads == EXPECTED_ENDOGENOUS_READS
        && local_smoke_report.total_reads == EXPECTED_TOTAL_READS
        && float_matches(local_smoke_report.endogenous_fraction, EXPECTED_ENDOGENOUS_FRACTION)
        && local_smoke_report.prealignment_fraction.is_none()
        && summary.method == local_smoke_report.method
        && summary.host_reference_scope == local_smoke_report.host_reference_scope
        && summary.mapped_reads == local_smoke_report.mapped_reads
        && summary.endogenous_reads == local_smoke_report.endogenous_reads
        && summary.total_reads == local_smoke_report.total_reads
        && float_matches(summary.endogenous_fraction, local_smoke_report.endogenous_fraction)
        && normalized_metrics.method == local_smoke_report.method
        && normalized_metrics.host_reference_scope == local_smoke_report.host_reference_scope
        && normalized_metrics.mapped_reads == local_smoke_report.mapped_reads
        && normalized_metrics.endogenous_reads == local_smoke_report.endogenous_reads
        && normalized_metrics.total_reads == local_smoke_report.total_reads
        && float_matches(
            normalized_metrics.endogenous_fraction,
            local_smoke_report.endogenous_fraction,
        )
        && local_smoke_report.endogenous_report.as_str()
            == local_smoke_report
                .endogenous_summary
                .replace("endogenous.summary.json", "endogenous.content.json")
}
