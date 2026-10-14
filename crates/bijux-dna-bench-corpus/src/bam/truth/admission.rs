//! Admit declared identities and expected alignment record aggregates.
use super::{
    resolve_manifest_relative_path, validate_bam_corpus_fixture_manifest_path,
    BamAlignmentSampleTruth, BamAlignmentTruthBundle, BamAlignmentTruthManifest,
    BAM_ALIGNMENT_TRUTH_BUNDLE_SCHEMA_VERSION, BAM_ALIGNMENT_TRUTH_FIXTURE_ID,
    BAM_ALIGNMENT_TRUTH_MANIFEST_SCHEMA_VERSION,
};
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &BamAlignmentTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != BAM_ALIGNMENT_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM alignment truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            BAM_ALIGNMENT_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != BAM_ALIGNMENT_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "BAM alignment truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            BAM_ALIGNMENT_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "BAM alignment truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    let corpus_manifest_path =
        resolve_manifest_relative_path(repo_root, &manifest.corpus_manifest_path);
    if !corpus_manifest_path.is_file() {
        return Err(anyhow!(
            "BAM alignment truth corpus manifest is missing: {}",
            corpus_manifest_path.display()
        ));
    }
    validate_bam_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "BAM alignment truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_manifest_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "BAM alignment truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.sample_ids.is_empty() {
        return Err(anyhow!(
            "BAM alignment truth manifest `{}` must declare at least one sample_id",
            manifest_path.display()
        ));
    }
    let mut sample_ids = BTreeSet::new();
    for sample_id in &manifest.sample_ids {
        if sample_id.trim().is_empty() {
            return Err(anyhow!(
                "BAM alignment truth manifest `{}` contains an empty sample_id",
                manifest_path.display()
            ));
        }
        if !sample_ids.insert(sample_id.clone()) {
            return Err(anyhow!("BAM alignment truth manifest repeats sample_id `{sample_id}`"));
        }
    }
    Ok(())
}

pub(super) fn validate_bundle_contract(
    manifest: &BamAlignmentTruthManifest,
    bundle: &BamAlignmentTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != BAM_ALIGNMENT_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM alignment truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            BAM_ALIGNMENT_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM alignment truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let actual_sample_ids =
        bundle.sample_truths.iter().map(|sample| sample.sample_id.as_str()).collect::<Vec<_>>();
    let expected_sample_ids = manifest.sample_ids.iter().map(String::as_str).collect::<Vec<_>>();
    if actual_sample_ids != expected_sample_ids {
        return Err(anyhow!(
            "BAM alignment truth bundle `{}` must contain samples {:?}",
            expected_path.display(),
            expected_sample_ids
        ));
    }
    for sample in &bundle.sample_truths {
        validate_sample_truth_contract(sample, expected_path)?;
    }
    Ok(())
}

fn validate_sample_truth_contract(
    sample: &BamAlignmentSampleTruth,
    expected_path: &Path,
) -> Result<()> {
    if sample.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM alignment truth bundle `{}` contains a sample with an empty sample_id",
            expected_path.display()
        ));
    }
    if sample.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` must declare a non-empty cohort",
            sample.sample_id
        ));
    }
    if sample.alignment_path.trim().is_empty() {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` must declare a non-empty alignment_path",
            sample.sample_id
        ));
    }
    if sample.total_reads != sample.records.len() as u64 {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` total_reads {} does not equal record count {}",
            sample.sample_id,
            sample.total_reads,
            sample.records.len()
        ));
    }
    if sample.total_reads != sample.mapped_reads + sample.unmapped_reads {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` total_reads {} does not equal mapped+unmapped {}",
            sample.sample_id,
            sample.total_reads,
            sample.mapped_reads + sample.unmapped_reads
        ));
    }
    let expected_mapped_contigs = sample
        .records
        .iter()
        .filter(|record| record.mapped)
        .filter_map(|record| record.reference_name.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if expected_mapped_contigs != sample.mapped_contigs {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` mapped_contigs drifted",
            sample.sample_id
        ));
    }
    let mapped_position_total =
        sample.mapped_positions.iter().map(|row| row.read_count).sum::<u64>();
    if mapped_position_total != sample.mapped_reads {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` mapped_positions count {} does not equal mapped_reads {}",
            sample.sample_id,
            mapped_position_total,
            sample.mapped_reads
        ));
    }
    let mapped_mapq_total =
        sample.mapped_mapq_classes.iter().map(|row| row.read_count).sum::<u64>();
    if mapped_mapq_total != sample.mapped_reads {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` mapped MAPQ class count {} does not equal mapped_reads {}",
            sample.sample_id,
            mapped_mapq_total,
            sample.mapped_reads
        ));
    }
    let cigar_total = sample.cigar_classes.iter().map(|row| row.read_count).sum::<u64>();
    if cigar_total != sample.total_reads {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` cigar class count {} does not equal total_reads {}",
            sample.sample_id,
            cigar_total,
            sample.total_reads
        ));
    }
    Ok(())
}
