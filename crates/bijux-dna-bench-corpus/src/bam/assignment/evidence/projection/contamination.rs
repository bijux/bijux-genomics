use crate::bam::assignment::configuration::models::LocalContaminationConfig;
use crate::bam::assignment::declarations::NOT_APPLICABLE;
use crate::bam::assignment::evidence::models::BamStageEvidence;
use crate::bam::assignment::formatting::asset_id_from_path;
use crate::bam::assignment::formatting::format_decimal;
use crate::bam::assignment::formatting::format_key_value_contract;
use anyhow::anyhow;
use anyhow::Result;
pub(in crate::bam::assignment) fn project_contamination(
    contamination: &LocalContaminationConfig,
    inherited_adna_profile: &str,
) -> Result<BamStageEvidence> {
    Ok(BamStageEvidence {
        sample_id: contamination.sample_id.clone(),
        input_contract: inherited_adna_profile.to_string(),
        benchmark_limits: format_key_value_contract([(
            "minimum_mean_coverage",
            format_decimal(contamination.minimum_mean_coverage),
        )]),
        required_assets: format_key_value_contract([
            ("reference_fasta", asset_id_from_path(&contamination.reference_fasta)?),
            (
                "reference_panel",
                asset_id_from_path(contamination.reference_panels.first().ok_or_else(|| {
                    anyhow!("BAM contamination config must declare at least one reference panel")
                })?)?,
            ),
        ]),
        expected_outputs: "contamination_report,summary,stage_metrics".to_string(),
        skip_behavior: NOT_APPLICABLE.to_string(),
    })
}
