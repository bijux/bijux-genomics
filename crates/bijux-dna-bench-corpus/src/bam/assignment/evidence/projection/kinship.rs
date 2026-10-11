use crate::bam::assignment::configuration::models::LocalKinshipConfig;
use crate::bam::assignment::evidence::models::BamStageEvidence;
use crate::bam::assignment::formatting::asset_id_from_path;
use crate::bam::assignment::formatting::format_key_value_contract;
use crate::bam::assignment::formatting::format_string_list;
use crate::bam::BamCorpusFixtureKinshipContractReport;
use crate::bam::BamCorpusFixtureValidationReport;
use anyhow::Result;
use std::path::Path;
pub(in crate::bam::assignment) fn project_kinship(
    kinship_fixture: &BamCorpusFixtureValidationReport,
    kinship: &LocalKinshipConfig,
    kinship_contract: &BamCorpusFixtureKinshipContractReport,
) -> Result<BamStageEvidence> {
    Ok(BamStageEvidence {
        sample_id: kinship
            .cases
            .iter()
            .map(|case| case.sample_id.clone())
            .collect::<Vec<_>>()
            .join(","),
        input_contract: format_key_value_contract([
            ("reference", asset_id_from_path(Path::new(&kinship_fixture.reference_fasta))?),
            ("reference_build", kinship_contract.reference_build.clone()),
            ("reference_panel", kinship_contract.reference_panel.clone()),
            ("population_scope", kinship_contract.population_scope.clone()),
        ]),
        benchmark_limits: format_key_value_contract(kinship.cases.iter().flat_map(|case| {
            [
                (format!("{}.min_overlap_snps", case.sample_id), case.min_overlap_snps.to_string()),
                (
                    format!("{}.observed_max_overlap_snps", case.sample_id),
                    case.expected_observed_max_overlap_snps.to_string(),
                ),
            ]
        })),
        required_assets: format_key_value_contract([
            ("reference_fasta", asset_id_from_path(Path::new(&kinship_fixture.reference_fasta))?),
            ("reference_panel", kinship_contract.reference_panel.clone()),
        ]),
        expected_outputs: format_string_list(&kinship_contract.expected_outputs),
        skip_behavior: format_key_value_contract(
            kinship_contract
                .cases
                .iter()
                .map(|case| (case.sample_id.clone(), case.skip_behavior.clone())),
        ),
    })
}
