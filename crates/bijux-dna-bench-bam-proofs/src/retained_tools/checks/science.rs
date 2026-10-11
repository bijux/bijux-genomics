use super::super::acquisition::EvidenceSet;
use super::super::declarations::{
    EXPECTED_BAM_COMPARABLE_STAGE_COUNT, EXPECTED_BAM_GOVERNED_METRIC_COUNT,
};
use anyhow::{bail, Result};
pub(in crate::retained_tools) fn check_science(evidence: &EvidenceSet) -> Result<String> {
    if evidence.science_thresholds.comparable_stage_count != EXPECTED_BAM_COMPARABLE_STAGE_COUNT
        || evidence.science_thresholds.stage_row_count != EXPECTED_BAM_COMPARABLE_STAGE_COUNT
        || evidence.science_thresholds.threshold_declared_stage_count
            != EXPECTED_BAM_COMPARABLE_STAGE_COUNT
        || evidence.science_thresholds.missing_threshold_stage_count != 0
        || evidence.science_thresholds.threshold_declared_metric_count
            != EXPECTED_BAM_GOVERNED_METRIC_COUNT
        || evidence.science_thresholds.missing_threshold_metric_count != 0
        || evidence.science_thresholds.governed_metric_count != EXPECTED_BAM_GOVERNED_METRIC_COUNT
    {
        bail!(
                    "BAM scientific thresholds drifted: comparable_stages={}, stage_rows={}, declared_stages={}, missing_stages={}, declared_metrics={}, missing_metrics={}, governed_metrics={}",
                    evidence.science_thresholds.comparable_stage_count,
                    evidence.science_thresholds.stage_row_count,
                    evidence.science_thresholds.threshold_declared_stage_count,
                    evidence.science_thresholds.missing_threshold_stage_count,
                    evidence.science_thresholds.threshold_declared_metric_count,
                    evidence.science_thresholds.missing_threshold_metric_count,
                    evidence.science_thresholds.governed_metric_count
                );
    }
    Ok("validated scientific pass/fail direction, tolerance, and insufficiency policy for every governed BAM comparable metric".to_string())
}
