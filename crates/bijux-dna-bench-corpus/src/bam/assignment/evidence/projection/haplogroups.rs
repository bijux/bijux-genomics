use crate::bam::assignment::configuration::models::LocalHaplogroupsConfig;
use crate::bam::assignment::declarations::NOT_APPLICABLE;
use crate::bam::assignment::evidence::models::BamStageEvidence;
use crate::bam::assignment::formatting::asset_id_from_path;
use crate::bam::assignment::formatting::format_decimal;
use crate::bam::assignment::formatting::format_key_value_contract;
use anyhow::Result;
pub(in crate::bam::assignment) fn project_haplogroups(
    haplogroups: &LocalHaplogroupsConfig,
    inherited_adna_profile: &str,
) -> Result<BamStageEvidence> {
    Ok(BamStageEvidence {
        sample_id: haplogroups.sample_id.clone(),
        input_contract: inherited_adna_profile.to_string(),
        benchmark_limits: format_key_value_contract([(
            "min_coverage",
            format_decimal(haplogroups.min_coverage),
        )]),
        required_assets: format_key_value_contract([
            ("reference_fasta", asset_id_from_path(&haplogroups.reference_fasta)?),
            ("reference_panel", haplogroups.reference_panel_id.clone()),
        ]),
        expected_outputs: "haplogroups,summary,stage_metrics".to_string(),
        skip_behavior: NOT_APPLICABLE.to_string(),
    })
}
