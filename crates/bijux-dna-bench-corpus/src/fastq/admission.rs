//! Owner: bijux-dna-bench-corpus
//! FASTQ corpus admission ownership.
use super::{
    anyhow, load_fastq_corpus_fixture_manifest_path, path_relative_to_repo,
    validate_fastq_corpus_fixture_manifest_contract, validate_fastq_corpus_fixture_sample,
    FastqCorpusFixtureValidationReport, Path, Result,
    FASTQ_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
};

/// Validate the corpus manifest and its declared local payload observations.
///
/// # Errors
/// Rejects invalid declarations, unavailable inputs and observed corpus contract violations.
pub fn validate_fastq_corpus_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<FastqCorpusFixtureValidationReport> {
    let manifest = load_fastq_corpus_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    validate_fastq_corpus_fixture_manifest_contract(&manifest)?;

    let samples = manifest
        .samples
        .iter()
        .map(|sample| {
            validate_fastq_corpus_fixture_sample(repo_root, manifest_dir, &manifest, sample)
        })
        .collect::<Result<Vec<_>>>()?;
    let single_end_sample_count = samples.iter().filter(|sample| sample.layout == "se").count();
    let paired_end_sample_count = samples.iter().filter(|sample| sample.layout == "pe").count();

    Ok(FastqCorpusFixtureValidationReport {
        schema_version: FASTQ_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        corpus_id: manifest.corpus_id,
        species: manifest.species,
        compression: manifest.compression.as_str().to_string(),
        reference_id: manifest.reference_id,
        sample_count: samples.len(),
        single_end_sample_count,
        paired_end_sample_count,
        valid: true,
        samples,
    })
}
