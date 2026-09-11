use crate::bam::assignment::configuration::models::LocalSexCase;
use crate::bam::assignment::declarations::NOT_APPLICABLE;
use crate::bam::assignment::evidence::models::BamStageEvidence;
use crate::bam::assignment::formatting::asset_id_from_path;
use crate::bam::assignment::formatting::format_decimal;
use crate::bam::assignment::formatting::format_key_value_contract;
use anyhow::Result;
pub(in crate::bam::assignment) fn project_sex(
    sex_case: &LocalSexCase,
    inherited_adna_profile: &str,
) -> Result<BamStageEvidence> {
    Ok(BamStageEvidence {
        sample_id: sex_case.sample_id.clone(),
        input_contract: inherited_adna_profile.to_string(),
        benchmark_limits: format_key_value_contract([
            ("expected_autosomal_coverage", format_decimal(sex_case.expected_autosomal_coverage)),
            ("expected_x_coverage", format_decimal(sex_case.expected_x_coverage)),
            ("expected_y_coverage", format_decimal(sex_case.expected_y_coverage)),
            ("minimum_y_sites", sex_case.minimum_y_sites.to_string()),
        ]),
        required_assets: format_key_value_contract([(
            "reference_fasta",
            asset_id_from_path(&sex_case.reference)?,
        )]),
        expected_outputs: "sex_report,summary,stage_metrics".to_string(),
        skip_behavior: NOT_APPLICABLE.to_string(),
    })
}
