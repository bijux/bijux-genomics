//! Owner: bijux-dna-bench-corpus
//! Amplicon truth-bundle admission.
use super::{
    anyhow, build_actual_truth_bundle, load_amplicon_truth_bundle,
    load_amplicon_truth_manifest_path, path_relative_to_repo, resolve_fixture_path,
    resolve_repo_relative_path, AmpliconTruthBundle, AmpliconTruthManifest,
    AmpliconTruthValidationReport, Path, Result, AMPLICON_TRUTH_BUNDLE_SCHEMA_VERSION,
    AMPLICON_TRUTH_FIXTURE_ID, AMPLICON_TRUTH_MANIFEST_SCHEMA_VERSION,
    AMPLICON_TRUTH_VALIDATION_SCHEMA_VERSION,
};
/// Admit the complete truth bundle and compare its declared observations with corpus inputs.
///
/// # Errors
/// Rejects malformed manifests, path/identity drift, invalid truth rows and mismatched observations.
pub fn validate_amplicon_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<AmpliconTruthValidationReport> {
    let manifest = load_amplicon_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;
    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("amplicon truth manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!(
            "amplicon truth expected bundle is missing: {}",
            expected_path.display()
        ));
    }

    let expected = load_amplicon_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;
    let actual = build_actual_truth_bundle(repo_root, &manifest)?;

    if expected != actual {
        return Err(anyhow!("amplicon truth drifted for fixture `{}`", manifest.fixture_id));
    }

    let checked_sections = vec![
        "primer_truths".to_string(),
        "asv_truths".to_string(),
        "chimera_truths".to_string(),
        "asv_representatives".to_string(),
        "non_chimeric_representatives".to_string(),
        "otu_representatives".to_string(),
        "otu_abundances".to_string(),
        "normalized_abundances".to_string(),
    ];
    let validated_row_count = expected.primer_truths.len()
        + expected.asv_truths.len()
        + expected.chimera_truths.len()
        + expected.asv_representatives.len()
        + expected.non_chimeric_representatives.len()
        + expected.otu_representatives.len()
        + expected.otu_abundances.len()
        + expected.normalized_abundances.len();

    Ok(AmpliconTruthValidationReport {
        schema_version: AMPLICON_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_section_count: checked_sections.len(),
        validated_row_count,
        valid: true,
        checked_sections,
    })
}

pub(crate) fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &AmpliconTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != AMPLICON_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "amplicon truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            AMPLICON_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != AMPLICON_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "amplicon truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            AMPLICON_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "amplicon truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "amplicon truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "amplicon truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }
    for required_path in [
        &manifest.corpus_manifest_path,
        &manifest.asv_representatives_path,
        &manifest.non_chimeric_representatives_path,
        &manifest.otu_representatives_path,
        &manifest.normalized_abundance_path,
    ] {
        let resolved = resolve_repo_relative_path(repo_root, required_path);
        if !resolved.is_file() {
            return Err(anyhow!("amplicon truth manifest path is missing: {}", resolved.display()));
        }
    }
    Ok(())
}

pub(crate) fn validate_bundle_contract(
    manifest: &AmpliconTruthManifest,
    bundle: &AmpliconTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != AMPLICON_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "amplicon truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            AMPLICON_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "amplicon truth bundle fixture_id `{}` must equal `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    if bundle.primer_truths.is_empty()
        || bundle.asv_truths.is_empty()
        || bundle.chimera_truths.is_empty()
        || bundle.asv_representatives.is_empty()
        || bundle.non_chimeric_representatives.is_empty()
        || bundle.otu_representatives.is_empty()
        || bundle.otu_abundances.is_empty()
        || bundle.normalized_abundances.is_empty()
    {
        return Err(anyhow!(
            "amplicon truth bundle `{}` must declare every governed truth section",
            expected_path.display()
        ));
    }
    Ok(())
}
