use super::contracts::float_matches;
use super::declarations::EXPECTED_AUTHENTICITY_REPORT_SCHEMA_VERSION;
use super::declarations::EXPECTED_COMPOSITION_SCHEMA_VERSION;
use super::declarations::EXPECTED_CONFIDENCE;
use super::declarations::EXPECTED_CONTAMINATION_ESTIMATE;
use super::declarations::EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION;
use super::declarations::EXPECTED_METHOD;
use super::declarations::EXPECTED_PMD_LIKE_SIGNAL_PRESENT;
use super::declarations::EXPECTED_SAMPLE_ID;
use super::declarations::EXPECTED_SCORE;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_STAGE_METRICS_SCHEMA_VERSION;
use super::declarations::EXPECTED_STATUS;
use super::declarations::EXPECTED_SUMMARY_SCHEMA_VERSION;
use super::declarations::REQUIRED_EVIDENCE_METRIC_IDS;
use super::observations::AuthenticityStageProof;
use super::paths::normalize_path_string;
use bijux_dna_bench_bam_evidence::readiness::damage_authenticity::BamDamageAuthenticityReadyRow;
use std::collections::BTreeSet;
use std::path::Path;
pub(super) fn check_authenticity_report_contract_ready(
    repo_root: &Path,
    authenticity_row: &BamDamageAuthenticityReadyRow,
    proof: &AuthenticityStageProof,
) -> bool {
    proof.local_smoke_report_path == repo_root.join(&authenticity_row.local_smoke_proof_path)
        && proof.authenticity_report.get("schema_version").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_AUTHENTICITY_REPORT_SCHEMA_VERSION)
        && proof.authenticity_report.get("summary").is_some()
        && proof.authenticity_report.get("composition").is_some()
}

pub(super) fn check_summary_ready(proof: &AuthenticityStageProof) -> bool {
    proof.summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && proof.summary.stage_id == EXPECTED_STAGE_ID
        && float_matches(proof.summary.score, EXPECTED_SCORE)
        && float_matches(proof.summary.confidence, EXPECTED_CONFIDENCE)
        && proof.summary.status == EXPECTED_STATUS
        && proof.summary.pmd_like_signal_present == EXPECTED_PMD_LIKE_SIGNAL_PRESENT
        && proof.summary.advisory_boundary.stage_id == EXPECTED_STAGE_ID
        && proof.summary.advisory_boundary.advisory_only
}

pub(super) fn check_composition_ready(proof: &AuthenticityStageProof) -> bool {
    proof.composition.schema_version == EXPECTED_COMPOSITION_SCHEMA_VERSION
        && proof.composition.stage_id == EXPECTED_STAGE_ID
        && float_matches(proof.composition.score, EXPECTED_SCORE)
        && float_matches(proof.composition.confidence, EXPECTED_CONFIDENCE)
        && proof.composition.pmd_like_signal_present == EXPECTED_PMD_LIKE_SIGNAL_PRESENT
        && REQUIRED_EVIDENCE_METRIC_IDS.iter().all(|metric_id| {
            proof
                .composition
                .consumed_metrics
                .get(*metric_id)
                .is_some_and(|entry| entry.available && entry.source == "stage_artifact")
        })
}

pub(super) fn check_normalized_metrics_ready(
    proof: &AuthenticityStageProof,
    expected_consumed_metric_ids: &[String],
) -> bool {
    proof.normalized_metrics.schema_version == EXPECTED_STAGE_METRICS_SCHEMA_VERSION
        && proof.normalized_metrics.stage_id == EXPECTED_STAGE_ID
        && proof.normalized_metrics.sample_id == EXPECTED_SAMPLE_ID
        && proof.normalized_metrics.method == EXPECTED_METHOD
        && float_matches(proof.normalized_metrics.expected_score, EXPECTED_SCORE)
        && float_matches(proof.normalized_metrics.score, EXPECTED_SCORE)
        && float_matches(proof.normalized_metrics.score_delta, 0.0)
        && float_matches(proof.normalized_metrics.expected_confidence, EXPECTED_CONFIDENCE)
        && float_matches(proof.normalized_metrics.confidence, EXPECTED_CONFIDENCE)
        && float_matches(proof.normalized_metrics.confidence_delta, 0.0)
        && proof.normalized_metrics.expected_status == EXPECTED_STATUS
        && proof.normalized_metrics.status == EXPECTED_STATUS
        && proof.normalized_metrics.expected_pmd_like_signal_present
            == EXPECTED_PMD_LIKE_SIGNAL_PRESENT
        && proof.normalized_metrics.pmd_like_signal_present == EXPECTED_PMD_LIKE_SIGNAL_PRESENT
        && proof
            .normalized_metrics
            .contamination_estimate
            .is_some_and(|value| float_matches(value, EXPECTED_CONTAMINATION_ESTIMATE))
        && proof.normalized_metrics.expected_consumed_metric_ids == expected_consumed_metric_ids
        && proof.normalized_metrics.consumed_metric_ids == expected_consumed_metric_ids
        && proof.normalized_metrics.missing_metric_ids.is_empty()
        && proof.normalized_metrics.expectation_matched
}

pub(super) fn check_evidence_consumption_ready(
    repo_root: &Path,
    proof: &AuthenticityStageProof,
    inherited_artifact_set: &BTreeSet<String>,
    required_artifact_paths: &[String],
    expected_consumed_metric_ids: &[String],
) -> bool {
    proof.local_smoke_report.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && proof.local_smoke_report.stage_id == EXPECTED_STAGE_ID
        && proof.local_smoke_report.sample_id == EXPECTED_SAMPLE_ID
        && proof.local_smoke_report.method == EXPECTED_METHOD
        && proof.local_smoke_report.expectation_matched
        && float_matches(proof.local_smoke_report.score, EXPECTED_SCORE)
        && float_matches(proof.local_smoke_report.confidence, EXPECTED_CONFIDENCE)
        && proof.local_smoke_report.status == EXPECTED_STATUS
        && proof.local_smoke_report.pmd_like_signal_present == EXPECTED_PMD_LIKE_SIGNAL_PRESENT
        && proof
            .local_smoke_report
            .contamination_estimate
            .is_some_and(|value| float_matches(value, EXPECTED_CONTAMINATION_ESTIMATE))
        && proof.local_smoke_report.consumed_metrics == expected_consumed_metric_ids
        && proof.local_smoke_report.missing_metrics.is_empty()
        && required_artifact_paths.iter().all(|path| inherited_artifact_set.contains(path))
        && proof
            .composition
            .consumed_metrics
            .get("damage")
            .and_then(|entry| entry.path.as_deref())
            .map(|path| normalize_path_string(repo_root, path))
            .as_deref()
            == Some(proof.local_smoke_report.damage_unified_metrics.as_str())
        && proof
            .composition
            .consumed_metrics
            .get("contamination")
            .and_then(|entry| entry.path.as_deref())
            .map(|path| normalize_path_string(repo_root, path))
            .as_deref()
            == Some(proof.local_smoke_report.contamination_summary.as_str())
        && proof
            .composition
            .consumed_metrics
            .get("complexity")
            .and_then(|entry| entry.path.as_deref())
            .map(|path| normalize_path_string(repo_root, path))
            .as_deref()
            == Some(proof.local_smoke_report.complexity_summary.as_str())
        && proof
            .composition
            .consumed_metrics
            .get("coverage")
            .and_then(|entry| entry.path.as_deref())
            .map(|path| normalize_path_string(repo_root, path))
            .as_deref()
            == Some(proof.local_smoke_report.coverage_regime.as_str())
        && proof
            .composition
            .consumed_metrics
            .get("mapping")
            .and_then(|entry| entry.path.as_deref())
            .map(|path| normalize_path_string(repo_root, path))
            .as_deref()
            == Some(proof.local_smoke_report.mapping_summary.as_str())
}
