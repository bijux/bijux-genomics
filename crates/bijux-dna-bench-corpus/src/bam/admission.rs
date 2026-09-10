//! Owner: bijux-dna-bench-corpus
//! BAM corpus admission ownership.
use super::{
    anyhow, load_bam_corpus_fixture_manifest_path, parse_reference_contigs, path_relative_to_repo,
    resolve_manifest_relative_path, validate_bam_corpus_fixture_manifest_contract,
    validate_bam_corpus_fixture_sample, validate_bam_corpus_genotyping_contract,
    validate_bam_corpus_kinship_contract, BamCorpusFixtureValidationReport, Path, Result,
    BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
};

#[expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "Governed corpus filenames have case-sensitive spelling."
)]
/// Validate the corpus manifest and its declared local payload observations.
///
/// # Errors
/// Rejects invalid declarations, unavailable inputs and observed corpus contract violations.
pub fn validate_bam_corpus_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamCorpusFixtureValidationReport> {
    let manifest = load_bam_corpus_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    validate_bam_corpus_fixture_manifest_contract(&manifest)?;

    let reference_fasta = resolve_manifest_relative_path(manifest_dir, &manifest.reference_fasta);
    if !reference_fasta.is_file() {
        return Err(anyhow!(
            "BAM corpus fixture reference FASTA is missing: {}",
            reference_fasta.display()
        ));
    }
    if !reference_fasta
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".fasta") || name.ends_with(".fa"))
    {
        return Err(anyhow!("BAM corpus fixture reference FASTA must end with `.fasta` or `.fa`"));
    }
    let reference_contigs = parse_reference_contigs(&reference_fasta)?;
    if reference_contigs.is_empty() {
        return Err(anyhow!(
            "BAM corpus fixture reference FASTA has no contigs: {}",
            reference_fasta.display()
        ));
    }
    let genotyping_contract = manifest
        .genotyping_contract
        .as_ref()
        .map(|contract| {
            validate_bam_corpus_genotyping_contract(
                repo_root,
                manifest_dir,
                &manifest,
                &reference_fasta,
                contract,
            )
        })
        .transpose()?;
    let kinship_contract = manifest
        .kinship_contract
        .as_ref()
        .map(|contract| {
            validate_bam_corpus_kinship_contract(
                repo_root,
                manifest_dir,
                &manifest,
                &reference_fasta,
                contract,
            )
        })
        .transpose()?;

    let samples = manifest
        .samples
        .iter()
        .map(|sample| {
            validate_bam_corpus_fixture_sample(repo_root, manifest_dir, &reference_contigs, sample)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BamCorpusFixtureValidationReport {
        schema_version: BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        corpus_id: manifest.corpus_id,
        species: manifest.species,
        reference_id: manifest.reference_id,
        reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
        udg_model: manifest.udg_model,
        damage_signal: manifest.damage_signal,
        expected_terminal_pattern_class: manifest.expected_terminal_pattern_class,
        genotyping_contract,
        kinship_contract,
        reference_contigs: reference_contigs.into_iter().collect(),
        sample_count: samples.len(),
        valid: true,
        samples,
    })
}
