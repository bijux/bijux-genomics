//! Owner: bijux-dna-bench-corpus
//! DAMAGE corpus admission ownership.
use super::{
    anyhow, ensure_fixture_file, ensure_reference_fasta, load_bam_damage_fixture_expectation_path,
    load_bam_damage_fixture_manifest_path, parse_reference_contigs, parse_tiny_sam_header,
    path_relative_to_repo, resolve_manifest_relative_path,
    validate_bam_damage_fixture_expectation_contract,
    validate_bam_damage_fixture_manifest_contract, BamDamageFixtureValidationReport, Path, Result,
    BAM_DAMAGE_FIXTURE_VALIDATION_SCHEMA_VERSION,
};

/// Validate the corpus manifest and its declared local payload observations.
///
/// # Errors
/// Rejects invalid declarations, unavailable inputs and observed corpus contract violations.
pub fn validate_bam_damage_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamDamageFixtureValidationReport> {
    let manifest = load_bam_damage_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    validate_bam_damage_fixture_manifest_contract(&manifest)?;

    let bam_path = resolve_manifest_relative_path(manifest_dir, &manifest.bam_path);
    ensure_fixture_file(&bam_path, ".sam", "BAM damage fixture alignment")?;
    let index_path = resolve_manifest_relative_path(manifest_dir, &manifest.index_path);
    ensure_fixture_file(&index_path, ".bai", "BAM damage fixture index")?;
    let reference_fasta = resolve_manifest_relative_path(manifest_dir, &manifest.reference_fasta);
    ensure_reference_fasta(&reference_fasta)?;
    let expected_damage_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.expected_damage_path);
    ensure_fixture_file(&expected_damage_path, ".json", "BAM damage expectation")?;

    let reference_contigs = parse_reference_contigs(&reference_fasta)?;
    if reference_contigs.is_empty() {
        return Err(anyhow!(
            "BAM damage fixture reference FASTA has no contigs: {}",
            reference_fasta.display()
        ));
    }
    let sam_header = parse_tiny_sam_header(&bam_path)?;
    if sam_header.contigs.is_empty() {
        return Err(anyhow!(
            "BAM damage fixture must declare at least one SAM contig in {}",
            bam_path.display()
        ));
    }
    if !sam_header.contigs.iter().all(|contig| reference_contigs.contains(contig)) {
        return Err(anyhow!(
            "BAM damage fixture SAM header contigs are not all present in the reference FASTA"
        ));
    }
    if !sam_header.sample_ids.iter().any(|sample_id| sample_id == &manifest.sample_id) {
        return Err(anyhow!(
            "BAM damage fixture sample_id `{}` is not present in the SAM header SM tags",
            manifest.sample_id
        ));
    }

    let expected_damage = load_bam_damage_fixture_expectation_path(&expected_damage_path)?;
    validate_bam_damage_fixture_expectation_contract(&expected_damage)?;
    if expected_damage.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM damage expectation fixture_id `{}` does not match manifest fixture_id `{}`",
            expected_damage.fixture_id,
            manifest.fixture_id
        ));
    }
    if expected_damage.sample_id != manifest.sample_id {
        return Err(anyhow!(
            "BAM damage expectation sample_id `{}` does not match manifest sample_id `{}`",
            expected_damage.sample_id,
            manifest.sample_id
        ));
    }

    let source_paths = manifest
        .source_paths
        .iter()
        .map(|path| {
            let absolute = if path.is_absolute() { path.clone() } else { repo_root.join(path) };
            if !absolute.is_file() {
                return Err(anyhow!(
                    "BAM damage fixture source path is missing: {}",
                    absolute.display()
                ));
            }
            Ok(path_relative_to_repo(repo_root, &absolute))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BamDamageFixtureValidationReport {
        schema_version: BAM_DAMAGE_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        fixture_id: manifest.fixture_id,
        sample_id: manifest.sample_id,
        species: manifest.species,
        bam_path: path_relative_to_repo(repo_root, &bam_path),
        index_path: path_relative_to_repo(repo_root, &index_path),
        reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
        expected_damage_path: path_relative_to_repo(repo_root, &expected_damage_path),
        udg_model: manifest.udg_model,
        expected_terminal_pattern_class: manifest.expected_terminal_pattern_class,
        limitations: manifest.limitations,
        observed_contigs: sam_header.contigs,
        observed_header_sample_ids: sam_header.sample_ids,
        expected_damage,
        source_paths,
        valid: true,
    })
}
