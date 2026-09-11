use super::contracts::float_matches;
use super::declarations::EXPECTED_EFFECTIVE_MODE;
use super::declarations::EXPECTED_KNOWN_SITES_ASSET_ID;
use super::declarations::EXPECTED_KNOWN_SITES_PATH;
use super::declarations::EXPECTED_MIN_BREADTH_1X;
use super::declarations::EXPECTED_MIN_MEAN_COVERAGE;
use super::declarations::EXPECTED_OBSERVED_BREADTH_1X;
use super::declarations::EXPECTED_OBSERVED_MEAN_COVERAGE;
use super::declarations::EXPECTED_REASON;
use super::declarations::EXPECTED_REFERENCE_FASTA;
use super::declarations::EXPECTED_REQUESTED_MODE;
use super::declarations::EXPECTED_SAMPLE_ID;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_STAGE_METRICS_SCHEMA_VERSION;
use super::declarations::EXPECTED_STATUS;
use super::declarations::EXPECTED_SUMMARY_SCHEMA_VERSION;
use super::observations::LocalRecalibrationSmokeReport;
use super::paths::path_relative_to_repo;
use std::path::Path;
pub(super) fn check_local_smoke_ready(
    smoke_report: &LocalRecalibrationSmokeReport,
    summary_path: &Path,
    stage_metrics_path: &Path,
    recalibration_report_path: &Path,
    recalibrated_bam_path: &Path,
) -> bool {
    smoke_report.expectation_matched
        && smoke_report.sample_id == EXPECTED_SAMPLE_ID
        && smoke_report.input_bam
            == "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_recalibration_low_coverage.sam"
        && smoke_report.reference_fasta == EXPECTED_REFERENCE_FASTA
        && smoke_report.known_sites == vec![EXPECTED_KNOWN_SITES_PATH.to_string()]
        && smoke_report.known_sites_asset_ids
            == vec![EXPECTED_KNOWN_SITES_ASSET_ID.to_string()]
        && smoke_report.requested_mode == EXPECTED_REQUESTED_MODE
        && smoke_report.effective_mode == EXPECTED_EFFECTIVE_MODE
        && smoke_report.status == EXPECTED_STATUS
        && smoke_report.reason == EXPECTED_REASON
        && float_matches(smoke_report.coverage_gate.min_mean_coverage, EXPECTED_MIN_MEAN_COVERAGE)
        && float_matches(smoke_report.coverage_gate.min_breadth_1x, EXPECTED_MIN_BREADTH_1X)
        && float_matches(smoke_report.observed_mean_coverage, EXPECTED_OBSERVED_MEAN_COVERAGE)
        && float_matches(smoke_report.observed_breadth_1x, EXPECTED_OBSERVED_BREADTH_1X)
        && smoke_report.output_bam_present
        && smoke_report.recalibration_report_present
        && recalibrated_bam_path.is_file()
        && recalibration_report_path.is_file()
        && summary_path.is_file()
        && stage_metrics_path.is_file()
}

pub(super) fn check_summary_ready(
    repo_root: &Path,
    smoke_report: &LocalRecalibrationSmokeReport,
    summary: &bijux_dna_domain_bam::BamRecalibrationSummaryV1,
) -> bool {
    summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && summary.stage_id == EXPECTED_STAGE_ID
        && path_relative_to_repo(repo_root, &summary.input_bam) == smoke_report.input_bam
        && summary
            .reference_fasta
            .as_ref()
            .is_some_and(|path| path_relative_to_repo(repo_root, path) == EXPECTED_REFERENCE_FASTA)
        && summary.known_sites.len() == 1
        && path_relative_to_repo(repo_root, &summary.known_sites[0]) == EXPECTED_KNOWN_SITES_PATH
        && summary.requested_mode == bijux_dna_domain_bam::params::BqsrMode::Standard
        && summary.effective_mode == bijux_dna_domain_bam::params::BqsrMode::Skip
        && summary.status == EXPECTED_STATUS
        && summary.reason == EXPECTED_REASON
        && float_matches(summary.coverage_gate.min_mean_coverage, EXPECTED_MIN_MEAN_COVERAGE)
        && float_matches(summary.coverage_gate.min_breadth_1x, EXPECTED_MIN_BREADTH_1X)
        && float_matches(summary.observed_mean_coverage, EXPECTED_OBSERVED_MEAN_COVERAGE)
        && float_matches(summary.observed_breadth_1x, EXPECTED_OBSERVED_BREADTH_1X)
        && summary.output_bam_present
        && summary.recalibration_report_present
}

pub(super) fn check_stage_metrics_ready(stage_metrics: &serde_json::Value) -> bool {
    stage_metrics.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_STAGE_METRICS_SCHEMA_VERSION)
        && stage_metrics.get("expected_requested_mode").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_REQUESTED_MODE)
        && stage_metrics.get("requested_mode").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_REQUESTED_MODE)
        && stage_metrics.get("expected_effective_mode").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_EFFECTIVE_MODE)
        && stage_metrics.get("effective_mode").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_EFFECTIVE_MODE)
        && stage_metrics.get("expected_status").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_STATUS)
        && stage_metrics.get("status").and_then(serde_json::Value::as_str) == Some(EXPECTED_STATUS)
        && stage_metrics.get("expected_reason").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_REASON)
        && stage_metrics.get("reason").and_then(serde_json::Value::as_str) == Some(EXPECTED_REASON)
        && stage_metrics.get("expected_known_sites").and_then(serde_json::Value::as_array)
            == Some(&vec![serde_json::json!(EXPECTED_KNOWN_SITES_PATH)])
        && stage_metrics.get("known_sites").and_then(serde_json::Value::as_array)
            == Some(&vec![serde_json::json!(EXPECTED_KNOWN_SITES_PATH)])
        && stage_metrics.get("expected_known_sites_asset_ids").and_then(serde_json::Value::as_array)
            == Some(&vec![serde_json::json!(EXPECTED_KNOWN_SITES_ASSET_ID)])
        && stage_metrics.get("known_sites_asset_ids").and_then(serde_json::Value::as_array)
            == Some(&vec![serde_json::json!(EXPECTED_KNOWN_SITES_ASSET_ID)])
        && stage_metrics.get("expectation_matched").and_then(serde_json::Value::as_bool)
            == Some(true)
        && stage_metrics.get("output_bam_present").and_then(serde_json::Value::as_bool)
            == Some(true)
        && stage_metrics.get("recalibration_report_present").and_then(serde_json::Value::as_bool)
            == Some(true)
}

pub(super) fn check_skip_behavior_ready(smoke_report: &LocalRecalibrationSmokeReport) -> bool {
    smoke_report.requested_mode == EXPECTED_REQUESTED_MODE
        && smoke_report.effective_mode == EXPECTED_EFFECTIVE_MODE
        && smoke_report.status == EXPECTED_STATUS
        && smoke_report.reason == EXPECTED_REASON
        && smoke_report.observed_mean_coverage < smoke_report.coverage_gate.min_mean_coverage
        && smoke_report.observed_breadth_1x < smoke_report.coverage_gate.min_breadth_1x
}

pub(super) fn check_known_sites_identity_ready(
    repo_root: &Path,
    smoke_report: &LocalRecalibrationSmokeReport,
    summary: &bijux_dna_domain_bam::BamRecalibrationSummaryV1,
) -> bool {
    smoke_report.known_sites == vec![EXPECTED_KNOWN_SITES_PATH.to_string()]
        && smoke_report.known_sites_asset_ids == vec![EXPECTED_KNOWN_SITES_ASSET_ID.to_string()]
        && summary.known_sites.len() == 1
        && path_relative_to_repo(repo_root, &summary.known_sites[0]) == EXPECTED_KNOWN_SITES_PATH
}

pub(super) fn check_recalibration_report_ready(recalibration_report: &str) -> bool {
    recalibration_report.contains("status=skipped")
        && recalibration_report.contains("reason=coverage_below_gate")
        && recalibration_report.contains("requested_mode=standard")
        && recalibration_report.contains("effective_mode=skip")
        && recalibration_report.contains(
            "known_sites=benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf",
        )
        && recalibration_report.contains("coverage_gate.min_mean_coverage=0.2")
        && recalibration_report.contains("coverage_gate.min_breadth_1x=0.2")
}
