use crate::bam::assignment::configuration::models::LocalDamageCase;
use crate::bam::assignment::declarations::NOT_APPLICABLE;
use crate::bam::assignment::evidence::models::BamStageEvidence;
pub(in crate::bam::assignment) fn project_damage(
    damage_case: &LocalDamageCase,
    damage_expectation: &str,
    damage_assets: &str,
) -> BamStageEvidence {
    BamStageEvidence {
        sample_id: damage_case.sample_id.clone(),
        input_contract: damage_expectation.to_string(),
        benchmark_limits: NOT_APPLICABLE.to_string(),
        required_assets: damage_assets.to_string(),
        expected_outputs: "damage_report,terminal_position_metrics,stage_metrics".to_string(),
        skip_behavior: NOT_APPLICABLE.to_string(),
    }
}
