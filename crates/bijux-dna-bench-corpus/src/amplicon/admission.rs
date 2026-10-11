//! Owner: bijux-dna-bench-corpus
//! AMPLICON corpus admission ownership.
use super::{
    anyhow, load_amplicon_corpus_fixture_manifest_path, load_fasta_records, path_relative_to_repo,
    resolve_manifest_relative_path, validate_amplicon_abundance_table,
    validate_amplicon_corpus_fixture_manifest_contract, validate_amplicon_corpus_fixture_sample,
    validate_amplicon_governance_contract, validate_chimera_expectations_contract,
    validate_expected_asvs_contract, validate_primer_fasta_headers, validate_primer_table_contract,
    AmpliconCorpusControlValidationReport, AmpliconCorpusFixtureValidationReport,
    AmpliconExpectedPresence, Path, Result, AMPLICON_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
};
use anyhow::Context;

/// Validate the corpus manifest and its declared local payload observations.
///
/// # Errors
/// Rejects invalid declarations, unavailable inputs and observed corpus contract violations.
pub fn validate_amplicon_corpus_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<AmpliconCorpusFixtureValidationReport> {
    let manifest = load_amplicon_corpus_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    validate_amplicon_corpus_fixture_manifest_contract(&manifest)?;

    let primer_fasta_path = resolve_manifest_relative_path(manifest_dir, &manifest.primer_fasta);
    let primers_tsv_path = resolve_manifest_relative_path(manifest_dir, &manifest.primers_tsv_path);
    let expected_asvs_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.expected_asvs_path);
    let chimera_controls_fasta_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.chimera_controls_fasta_path);
    let chimera_expectations_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.chimera_expectations_path);
    let amplicon_governance_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.amplicon_governance_path);
    validate_amplicon_governance_contract(
        repo_root,
        &manifest,
        &primer_fasta_path,
        &amplicon_governance_path,
    )?;
    let primer_fasta_records = validate_primer_fasta_headers(&manifest, &primer_fasta_path)?;
    let primer_table_rows = validate_primer_table_contract(
        repo_root,
        &manifest,
        &primers_tsv_path,
        &primer_fasta_records,
    )?;
    let expected_asv_rows =
        validate_expected_asvs_contract(repo_root, &manifest, &expected_asvs_path)?;
    let chimera_control_records = load_fasta_records(&chimera_controls_fasta_path)?;
    let chimera_expectation_rows = validate_chimera_expectations_contract(
        repo_root,
        &manifest,
        &chimera_controls_fasta_path,
        &chimera_expectations_path,
        &chimera_control_records,
    )?;
    let abundance_tables = manifest
        .abundance_tables
        .iter()
        .map(|table| validate_amplicon_abundance_table(repo_root, manifest_dir, &manifest, table))
        .collect::<Result<Vec<_>>>()?;
    let report_primer_fasta_path = canonical_report_path(&primer_fasta_path)?;
    let report_primers_tsv_path = canonical_report_path(&primers_tsv_path)?;
    let report_expected_asvs_path = canonical_report_path(&expected_asvs_path)?;
    let report_chimera_controls_fasta_path = canonical_report_path(&chimera_controls_fasta_path)?;
    let report_chimera_expectations_path = canonical_report_path(&chimera_expectations_path)?;
    let report_amplicon_governance_path = canonical_report_path(&amplicon_governance_path)?;
    let (expected_asv_present_row_count, expected_asv_absent_row_count) =
        presence_counts(expected_asv_rows.iter().map(|row| row.expected_presence));
    let (chimera_expected_present_row_count, chimera_expected_absent_row_count) =
        presence_counts(chimera_expectation_rows.iter().map(|row| row.expected_presence));

    let samples = sample_reports(repo_root, manifest_dir, &manifest)?;
    let controls = control_reports(&manifest);

    Ok(AmpliconCorpusFixtureValidationReport {
        schema_version: AMPLICON_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        corpus_id: manifest.corpus_id,
        assay_id: manifest.assay_id,
        marker_id: manifest.marker_id,
        target_region: manifest.target_region,
        compression: manifest.compression.as_str().to_string(),
        primer_set_id: manifest.primer_set_id,
        forward_primer_id: manifest.forward_primer_id,
        reverse_primer_id: manifest.reverse_primer_id,
        primer_fasta: path_relative_to_repo(repo_root, &report_primer_fasta_path),
        primers_tsv_path: path_relative_to_repo(repo_root, &report_primers_tsv_path),
        primer_table_row_count: primer_table_rows.len(),
        expected_asvs_path: path_relative_to_repo(repo_root, &report_expected_asvs_path),
        expected_asv_row_count: expected_asv_rows.len(),
        expected_asv_present_row_count,
        expected_asv_absent_row_count,
        chimera_controls_fasta_path: path_relative_to_repo(
            repo_root,
            &report_chimera_controls_fasta_path,
        ),
        chimera_expectations_path: path_relative_to_repo(
            repo_root,
            &report_chimera_expectations_path,
        ),
        chimera_expectation_row_count: chimera_expectation_rows.len(),
        chimera_expected_present_row_count,
        chimera_expected_absent_row_count,
        amplicon_governance_path: path_relative_to_repo(
            repo_root,
            &report_amplicon_governance_path,
        ),
        abundance_table_count: abundance_tables.len(),
        sample_count: samples.len(),
        control_count: controls.len(),
        valid: true,
        controls,
        abundance_tables,
        samples,
    })
}

fn control_reports(
    manifest: &super::AmpliconCorpusFixtureManifest,
) -> Vec<AmpliconCorpusControlValidationReport> {
    manifest
        .controls
        .iter()
        .map(|control| AmpliconCorpusControlValidationReport {
            sample_id: control.sample_id.clone(),
            control_kind: control.control_kind.clone(),
            purpose: control.purpose.clone(),
            valid: true,
        })
        .collect::<Vec<_>>()
}

fn canonical_report_path(path: &Path) -> Result<std::path::PathBuf> {
    path.canonicalize().with_context(|| format!("canonicalize {}", path.display()))
}
fn presence_counts(rows: impl Iterator<Item = AmpliconExpectedPresence>) -> (usize, usize) {
    rows.fold((0, 0), |(present, absent), value| match value {
        AmpliconExpectedPresence::Present => (present + 1, absent),
        AmpliconExpectedPresence::Absent => (present, absent + 1),
    })
}

fn sample_reports(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &super::AmpliconCorpusFixtureManifest,
) -> Result<Vec<super::AmpliconCorpusFixtureSampleValidationReport>> {
    manifest
        .samples
        .iter()
        .map(|sample| {
            validate_amplicon_corpus_fixture_sample(repo_root, manifest_dir, manifest, sample)
        })
        .collect::<Result<Vec<_>>>()
}
