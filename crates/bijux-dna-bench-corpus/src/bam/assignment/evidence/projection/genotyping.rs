use crate::bam::assignment::configuration::models::LocalGenotypingConfig;
use crate::bam::assignment::declarations::NOT_APPLICABLE;
use crate::bam::assignment::evidence::models::BamStageEvidence;
use crate::bam::assignment::formatting::asset_id_from_path;
use crate::bam::assignment::formatting::format_decimal;
use crate::bam::assignment::formatting::format_key_value_contract;
use crate::bam::assignment::formatting::format_string_list;
use crate::bam::BamCorpusFixtureGenotypingContractReport;
use anyhow::Result;
pub(in crate::bam::assignment) fn project_genotyping(
    genotyping: &LocalGenotypingConfig,
    genotyping_contract: &BamCorpusFixtureGenotypingContractReport,
) -> Result<BamStageEvidence> {
    Ok(BamStageEvidence {
        sample_id: genotyping.sample_id.clone(),
        input_contract: format_key_value_contract([
            ("reference", asset_id_from_path(&genotyping.reference_fasta)?),
            ("regions", asset_id_from_path(&genotyping.regions)?),
            ("sites", asset_id_from_path(&genotyping.sites_vcf)?),
        ]),
        benchmark_limits: format_key_value_contract([
            ("min_call_rate", format_decimal(genotyping.min_call_rate)),
            ("min_posterior", format_decimal(genotyping.min_posterior)),
        ]),
        required_assets: format_key_value_contract([
            ("reference_fasta", asset_id_from_path(&genotyping.reference_fasta)?),
            ("regions", asset_id_from_path(&genotyping.regions)?),
            ("sites_vcf", asset_id_from_path(&genotyping.sites_vcf)?),
        ]),
        expected_outputs: format_string_list(&genotyping_contract.expected_outputs),
        skip_behavior: NOT_APPLICABLE.to_string(),
    })
}
