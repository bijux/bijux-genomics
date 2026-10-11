//! Owner: bijux-dna-bench-corpus
//! BAM corpus genotyping ownership.
use super::{
    anyhow, fs, path_relative_to_repo, resolve_manifest_relative_path,
    validate_expected_output_contract, BamCorpusFixtureGenotypingContract,
    BamCorpusFixtureGenotypingContractReport, BamCorpusFixtureManifest, Path, Result,
};
use anyhow::Context;

#[expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "Governed corpus filenames have case-sensitive spelling."
)]
pub(super) fn validate_bam_corpus_genotyping_contract(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &BamCorpusFixtureManifest,
    reference_fasta: &Path,
    contract: &BamCorpusFixtureGenotypingContract,
) -> Result<BamCorpusFixtureGenotypingContractReport> {
    if contract.sample_id.trim().is_empty() {
        return Err(anyhow!("BAM corpus genotyping contract must declare a non-empty `sample_id`"));
    }
    if !manifest.samples.iter().any(|sample| sample.sample_id == contract.sample_id) {
        return Err(anyhow!(
            "BAM corpus genotyping contract sample `{}` must exist in the fixture sample set",
            contract.sample_id
        ));
    }
    if contract.min_posterior.is_nan() || !(0.0..=1.0).contains(&contract.min_posterior) {
        return Err(anyhow!(
            "BAM corpus genotyping contract `min_posterior` must stay within [0, 1]"
        ));
    }
    if contract.min_call_rate.is_nan() || !(0.0..=1.0).contains(&contract.min_call_rate) {
        return Err(anyhow!(
            "BAM corpus genotyping contract `min_call_rate` must stay within [0, 1]"
        ));
    }
    let sites_vcf = resolve_manifest_relative_path(manifest_dir, &contract.sites_vcf);
    if !sites_vcf.is_file() {
        return Err(anyhow!(
            "BAM corpus genotyping contract sites VCF is missing: {}",
            sites_vcf.display()
        ));
    }
    if !sites_vcf
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".vcf") || name.ends_with(".vcf.gz"))
    {
        return Err(anyhow!(
            "BAM corpus genotyping contract sites VCF must end with `.vcf` or `.vcf.gz`"
        ));
    }
    let sites_vcf = fs::canonicalize(&sites_vcf)
        .with_context(|| format!("canonicalize {}", sites_vcf.display()))?;
    let regions = resolve_manifest_relative_path(manifest_dir, &contract.regions);
    if !regions.is_file() {
        return Err(anyhow!(
            "BAM corpus genotyping contract regions file is missing: {}",
            regions.display()
        ));
    }
    if !regions
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".txt") || name.ends_with(".bed"))
    {
        return Err(anyhow!(
            "BAM corpus genotyping contract regions file must end with `.txt` or `.bed`"
        ));
    }
    let regions = fs::canonicalize(&regions)
        .with_context(|| format!("canonicalize {}", regions.display()))?;
    let expected_outputs = validate_expected_output_contract(
        &contract.expected_outputs,
        "BAM corpus genotyping contract",
    )?;

    if path_relative_to_repo(repo_root, reference_fasta).trim().is_empty() {
        return Err(anyhow!(
            "BAM corpus genotyping contract must resolve a non-empty reference FASTA path"
        ));
    }

    Ok(BamCorpusFixtureGenotypingContractReport {
        sample_id: contract.sample_id.clone(),
        sites_vcf: path_relative_to_repo(repo_root, &sites_vcf),
        regions: path_relative_to_repo(repo_root, &regions),
        min_posterior: contract.min_posterior,
        min_call_rate: contract.min_call_rate,
        expected_outputs,
    })
}
