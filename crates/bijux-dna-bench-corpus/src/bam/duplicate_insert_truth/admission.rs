//! Admit corpus membership, histogram consistency and expected sample order.
use super::model::{
    BamDuplicateInsertSampleTruth, BamDuplicateInsertTruthBundle, BamDuplicateInsertTruthManifest,
};
use super::{
    BAM_DUPLICATE_INSERT_TRUTH_BUNDLE_SCHEMA_VERSION, BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID,
    BAM_DUPLICATE_INSERT_TRUTH_MANIFEST_SCHEMA_VERSION,
};
use crate::bam::validate_bam_corpus_fixture_manifest_path;
use crate::paths::resolve_manifest_relative_path as resolve_repo_relative_path;
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &BamDuplicateInsertTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != BAM_DUPLICATE_INSERT_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM duplicate/insert truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            BAM_DUPLICATE_INSERT_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "BAM duplicate/insert truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "BAM duplicate/insert truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    if !corpus_manifest_path.is_file() {
        return Err(anyhow!(
            "BAM duplicate/insert truth corpus manifest is missing: {}",
            corpus_manifest_path.display()
        ));
    }
    validate_bam_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "BAM duplicate/insert truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "BAM duplicate/insert truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.sample_ids.is_empty() {
        return Err(anyhow!(
            "BAM duplicate/insert truth manifest `{}` must declare at least one sample_id",
            manifest_path.display()
        ));
    }
    let mut sample_ids = BTreeSet::new();
    for sample_id in &manifest.sample_ids {
        if sample_id.trim().is_empty() {
            return Err(anyhow!(
                "BAM duplicate/insert truth manifest `{}` contains an empty sample_id",
                manifest_path.display()
            ));
        }
        if !sample_ids.insert(sample_id.clone()) {
            return Err(anyhow!(
                "BAM duplicate/insert truth manifest repeats sample_id `{sample_id}`"
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_bundle_contract(
    manifest: &BamDuplicateInsertTruthManifest,
    bundle: &BamDuplicateInsertTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != BAM_DUPLICATE_INSERT_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM duplicate/insert truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            BAM_DUPLICATE_INSERT_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM duplicate/insert truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let actual_sample_ids =
        bundle.sample_truths.iter().map(|sample| sample.sample_id.as_str()).collect::<Vec<_>>();
    let expected_sample_ids = manifest.sample_ids.iter().map(String::as_str).collect::<Vec<_>>();
    if actual_sample_ids != expected_sample_ids {
        return Err(anyhow!(
            "BAM duplicate/insert truth bundle `{}` must contain samples {:?}",
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
    sample: &BamDuplicateInsertSampleTruth,
    expected_path: &Path,
) -> Result<()> {
    if sample.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM duplicate/insert truth bundle `{}` contains a sample with an empty sample_id",
            expected_path.display()
        ));
    }
    if sample.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM duplicate/insert truth sample `{}` must declare a non-empty cohort",
            sample.sample_id
        ));
    }
    if sample.alignment_path.trim().is_empty() {
        return Err(anyhow!(
            "BAM duplicate/insert truth sample `{}` must declare a non-empty alignment_path",
            sample.sample_id
        ));
    }
    if sample.total_reads != sample.mapped_reads + sample.unmapped_reads {
        return Err(anyhow!(
            "BAM duplicate/insert truth sample `{}` total_reads {} does not equal mapped+unmapped {}",
            sample.sample_id,
            sample.total_reads,
            sample.mapped_reads + sample.unmapped_reads
        ));
    }
    let examined_reads = sample
        .duplicate_family_histogram
        .iter()
        .map(|bin| bin.family_size.saturating_mul(bin.family_count))
        .sum::<u64>();
    if examined_reads != sample.examined_reads {
        return Err(anyhow!(
            "BAM duplicate/insert truth sample `{}` examined_reads {} does not match duplicate family histogram total {}",
            sample.sample_id,
            sample.examined_reads,
            examined_reads
        ));
    }
    let duplicate_reads = sample
        .duplicate_family_histogram
        .iter()
        .map(|bin| bin.family_size.saturating_sub(1).saturating_mul(bin.family_count))
        .sum::<u64>();
    if duplicate_reads != sample.duplicate_reads {
        return Err(anyhow!(
            "BAM duplicate/insert truth sample `{}` duplicate_reads {} does not match duplicate family histogram total {}",
            sample.sample_id,
            sample.duplicate_reads,
            duplicate_reads
        ));
    }
    let pair_count = sample.insert_size_histogram.iter().map(|bin| bin.pair_count).sum::<u64>();
    if pair_count != sample.pair_count {
        return Err(anyhow!(
            "BAM duplicate/insert truth sample `{}` pair_count {} does not match insert-size histogram total {}",
            sample.sample_id,
            sample.pair_count,
            pair_count
        ));
    }
    if sample.pair_count == 0 {
        if sample.median_insert_size.is_some()
            || sample.mean_insert_size.is_some()
            || sample.min_insert_size.is_some()
            || sample.max_insert_size.is_some()
        {
            return Err(anyhow!(
                "BAM duplicate/insert truth sample `{}` must not declare insert-size summary values when pair_count is zero",
                sample.sample_id
            ));
        }
    } else if sample.median_insert_size.is_none()
        || sample.mean_insert_size.is_none()
        || sample.min_insert_size.is_none()
        || sample.max_insert_size.is_none()
    {
        return Err(anyhow!(
            "BAM duplicate/insert truth sample `{}` must declare insert-size summary values when pair_count is non-zero",
            sample.sample_id
        ));
    }
    if sample.duplicate_pairs > sample.pair_count {
        return Err(anyhow!(
            "BAM duplicate/insert truth sample `{}` duplicate_pairs {} cannot exceed pair_count {}",
            sample.sample_id,
            sample.duplicate_pairs,
            sample.pair_count
        ));
    }
    Ok(())
}
