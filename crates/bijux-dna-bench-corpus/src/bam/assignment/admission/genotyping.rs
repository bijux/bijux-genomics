use crate::bam::assignment::admission::identity::ensure_float_matches;
use crate::bam::assignment::admission::identity::ensure_paths_match;
use crate::bam::assignment::admission::samples::ensure_stage_sample_matches_bam_fixture;
use crate::bam::assignment::configuration::models::LocalGenotypingConfig;
use crate::bam::BamCorpusFixtureGenotypingContractReport;
use crate::bam::BamCorpusFixtureValidationReport;
use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
pub(in crate::bam::assignment) fn ensure_stage_matches_genotyping_fixture(
    repo_root: &Path,
    stage_id: &str,
    config: &LocalGenotypingConfig,
    fixture: &BamCorpusFixtureValidationReport,
) -> Result<BamCorpusFixtureGenotypingContractReport> {
    let contract = fixture.genotyping_contract.clone().ok_or_else(|| {
        anyhow!("BAM stage `{stage_id}` requires a governed genotyping contract in the fixture")
    })?;
    ensure_stage_sample_matches_bam_fixture(stage_id, &config.sample_id, fixture)?;
    if config.sample_id != contract.sample_id {
        return Err(anyhow!(
            "BAM stage `{stage_id}` sample `{}` must match genotyping contract sample `{}`",
            config.sample_id,
            contract.sample_id
        ));
    }
    ensure_paths_match(
        repo_root,
        stage_id,
        "reference_fasta",
        &config.reference_fasta,
        &repo_root.join(&fixture.reference_fasta),
    )?;
    ensure_paths_match(
        repo_root,
        stage_id,
        "sites_vcf",
        &config.sites_vcf,
        &repo_root.join(&contract.sites_vcf),
    )?;
    ensure_paths_match(
        repo_root,
        stage_id,
        "regions",
        &config.regions,
        &repo_root.join(&contract.regions),
    )?;
    ensure_float_matches(stage_id, "min_posterior", config.min_posterior, contract.min_posterior)?;
    ensure_float_matches(stage_id, "min_call_rate", config.min_call_rate, contract.min_call_rate)?;
    Ok(contract)
}
