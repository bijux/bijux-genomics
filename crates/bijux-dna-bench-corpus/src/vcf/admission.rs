//! Owner: bijux-dna-bench-corpus
//! VCF corpus admission ownership.
use super::{
    anyhow, load_population_metadata, load_sample_metadata, load_validated_reference_contigs,
    load_vcf_corpus_fixture_manifest_path, path_relative_to_repo, resolve_manifest_relative_path,
    validate_manifest_sample_coverage, validate_sample_metadata_population_links,
    validate_target_sites_bed, validate_variant_set, validate_vcf_corpus_fixture_manifest_contract,
    Path, Result, VcfCorpusFixtureValidationReport, VcfVariantRole,
    VCF_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
};

/// Validate the corpus manifest and its declared local payload observations.
///
/// # Errors
/// Rejects invalid declarations, unavailable inputs and observed corpus contract violations.
pub fn validate_vcf_corpus_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<VcfCorpusFixtureValidationReport> {
    let manifest = load_vcf_corpus_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    validate_vcf_corpus_fixture_manifest_contract(&manifest)?;

    let reference_fasta =
        resolve_manifest_relative_path(manifest_dir, &manifest.reference_fasta_path);
    let reference_fasta_index =
        resolve_manifest_relative_path(manifest_dir, &manifest.reference_fasta_index_path);
    let reference_dict =
        resolve_manifest_relative_path(manifest_dir, &manifest.reference_dict_path);
    let target_sites_bed =
        resolve_manifest_relative_path(manifest_dir, &manifest.target_sites_bed_path);
    let sample_metadata_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.sample_metadata_path);
    let population_metadata_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.population_metadata_path);

    let reference_contigs = load_validated_reference_contigs(
        &reference_fasta,
        &reference_fasta_index,
        &reference_dict,
    )?;
    let reference_contig_names =
        reference_contigs.iter().map(|contig| contig.name.clone()).collect::<Vec<_>>();
    let target_interval_count = validate_target_sites_bed(&target_sites_bed, &reference_contigs)?;
    let sample_metadata = load_sample_metadata(&sample_metadata_path)?;
    let population_ids = load_population_metadata(&population_metadata_path)?;
    validate_sample_metadata_population_links(&sample_metadata, &population_ids)?;

    let variant_sets = [
        (VcfVariantRole::Raw, &manifest.raw_vcf_path, &manifest.expected_raw_sample_ids),
        (
            VcfVariantRole::Filtered,
            &manifest.filtered_vcf_path,
            &manifest.expected_filtered_sample_ids,
        ),
        (
            VcfVariantRole::Multisample,
            &manifest.multisample_vcf_path,
            &manifest.expected_multisample_sample_ids,
        ),
        (VcfVariantRole::Phased, &manifest.phased_vcf_path, &manifest.expected_phased_sample_ids),
        (VcfVariantRole::Panel, &manifest.panel_vcf_path, &manifest.expected_panel_sample_ids),
    ]
    .into_iter()
    .map(|(role, path, expected_sample_ids)| {
        validate_variant_set(
            repo_root,
            manifest_dir,
            role,
            path,
            expected_sample_ids,
            &reference_contigs,
        )
    })
    .collect::<Result<Vec<_>>>()?;

    validate_manifest_sample_coverage(&sample_metadata, &variant_sets)?;

    Ok(VcfCorpusFixtureValidationReport {
        schema_version: VCF_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        corpus_id: manifest.corpus_id,
        reference_id: manifest.reference_id,
        reference_fasta_path: path_relative_to_repo(repo_root, &reference_fasta),
        reference_fasta_index_path: path_relative_to_repo(repo_root, &reference_fasta_index),
        reference_dict_path: path_relative_to_repo(repo_root, &reference_dict),
        reference_contigs: reference_contig_names,
        target_sites_bed_path: path_relative_to_repo(repo_root, &target_sites_bed),
        target_interval_count,
        sample_metadata_path: path_relative_to_repo(repo_root, &sample_metadata_path),
        population_metadata_path: path_relative_to_repo(repo_root, &population_metadata_path),
        sample_count: sample_metadata.len(),
        population_count: population_ids.len(),
        valid: true,
        variant_sets: variant_sets.into_iter().map(|(report, _)| report).collect(),
    })
}
