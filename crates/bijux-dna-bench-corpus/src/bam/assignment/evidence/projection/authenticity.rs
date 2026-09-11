use crate::bam::assignment::configuration::models::LocalAuthenticityCase;
use crate::bam::assignment::declarations::NOT_APPLICABLE;
use crate::bam::assignment::evidence::models::BamStageEvidence;
use crate::bam::assignment::formatting::format_key_value_contract;
use crate::bam::assignment::formatting::format_numeric_list;
pub(in crate::bam::assignment) fn project_authenticity(
    authenticity_case: &LocalAuthenticityCase,
    damage_expectation: &str,
    damage_assets: &str,
) -> BamStageEvidence {
    BamStageEvidence {
        sample_id: authenticity_case.sample_id.clone(),
        input_contract: damage_expectation.to_string(),
        benchmark_limits: format_key_value_contract([
            ("complexity_min_reads", authenticity_case.complexity_min_reads.to_string()),
            (
                "coverage_depth_thresholds",
                format_numeric_list(&authenticity_case.coverage_depth_thresholds),
            ),
        ]),
        required_assets: damage_assets.to_string(),
        expected_outputs: "authenticity_report,summary,stage_metrics".to_string(),
        skip_behavior: NOT_APPLICABLE.to_string(),
    }
}
