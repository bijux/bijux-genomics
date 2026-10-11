//! Owner: bijux-dna-bench-corpus
//! EDNA corpus admission ownership.
use super::{
    anyhow, load_edna_corpus_fixture_manifest_path, load_validated_edna_expected_taxa_rows,
    path_relative_to_repo, resolve_edna_expected_taxa_path,
    validate_edna_corpus_fixture_manifest_contract, validate_edna_corpus_fixture_sample,
    EdnaCorpusFixtureValidationReport, EdnaExpectedPresence, Path, Result,
    EDNA_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
};

/// Validate the corpus manifest and its declared local payload observations.
///
/// # Errors
/// Rejects invalid declarations, unavailable inputs and observed corpus contract violations.
pub fn validate_edna_corpus_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<EdnaCorpusFixtureValidationReport> {
    let manifest = load_edna_corpus_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    validate_edna_corpus_fixture_manifest_contract(&manifest)?;

    let samples = manifest
        .samples
        .iter()
        .map(|sample| {
            validate_edna_corpus_fixture_sample(repo_root, manifest_dir, &manifest, sample)
        })
        .collect::<Result<Vec<_>>>()?;
    let expected_taxa_path = resolve_edna_expected_taxa_path(manifest_path, &manifest)?;
    let expected_taxa_rows =
        load_validated_edna_expected_taxa_rows(&manifest, &expected_taxa_path)?;
    let expected_present_row_count = expected_taxa_rows
        .iter()
        .filter(|row| row.expected_presence == EdnaExpectedPresence::Present)
        .count();
    let expected_absent_row_count =
        expected_taxa_rows.len().saturating_sub(expected_present_row_count);

    Ok(EdnaCorpusFixtureValidationReport {
        schema_version: EDNA_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        corpus_id: manifest.corpus_id,
        community_id: manifest.community_id,
        compression: manifest.compression.as_str().to_string(),
        sample_count: samples.len(),
        expected_taxa_count: manifest.expected_taxa.len(),
        expected_taxa_path: path_relative_to_repo(repo_root, &expected_taxa_path),
        expected_taxa_output_row_count: expected_taxa_rows.len(),
        expected_present_row_count,
        expected_absent_row_count,
        expected_taxa: manifest.expected_taxa,
        valid: true,
        samples,
    })
}
