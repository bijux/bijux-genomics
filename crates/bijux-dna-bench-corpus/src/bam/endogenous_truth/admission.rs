//! Admit endogenous declarations and expected count/fraction invariants.
use super::model::{
    BamEndogenousTruthBundle, BamEndogenousTruthManifest, BamEndogenousTruthSample,
};
use super::{
    BAM_ENDOGENOUS_TRUTH_BUNDLE_SCHEMA_VERSION, BAM_ENDOGENOUS_TRUTH_FIXTURE_ID,
    BAM_ENDOGENOUS_TRUTH_MANIFEST_SCHEMA_VERSION,
};
use crate::bam::validate_bam_corpus_fixture_manifest_path;
use crate::paths::resolve_manifest_relative_path as resolve_repo_relative_path;
use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::numeric::checked_f64_from_u64;
use bijux_dna_domain_bam::BAM_ENDOGENOUS_TRUTH_SUMMARY_SCHEMA_VERSION;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &BamEndogenousTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != BAM_ENDOGENOUS_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM endogenous truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            BAM_ENDOGENOUS_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != BAM_ENDOGENOUS_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "BAM endogenous truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            BAM_ENDOGENOUS_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    if !corpus_manifest_path.is_file() {
        return Err(anyhow!(
            "BAM endogenous truth corpus manifest is missing: {}",
            corpus_manifest_path.display()
        ));
    }
    validate_bam_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "BAM endogenous truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }
    let mut sample_ids = BTreeSet::new();
    for case in &manifest.cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "BAM endogenous truth manifest `{}` contains an empty sample_id",
                manifest_path.display()
            ));
        }
        if !sample_ids.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "BAM endogenous truth manifest repeats sample_id `{}`",
                case.sample_id
            ));
        }
        if case.method.trim().is_empty() {
            return Err(anyhow!(
                "BAM endogenous truth case `{}` must declare a non-empty method",
                case.sample_id
            ));
        }
        if case.host_reference_scope.trim().is_empty() {
            return Err(anyhow!(
                "BAM endogenous truth case `{}` must declare a non-empty host_reference_scope",
                case.sample_id
            ));
        }
        if !(0.0..=1.0).contains(&case.prealignment_fraction) {
            return Err(anyhow!(
                "BAM endogenous truth case `{}` prealignment_fraction {} must be within [0, 1]",
                case.sample_id,
                case.prealignment_fraction
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_bundle_contract(
    manifest: &BamEndogenousTruthManifest,
    bundle: &BamEndogenousTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != BAM_ENDOGENOUS_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM endogenous truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            BAM_ENDOGENOUS_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM endogenous truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let actual_sample_ids =
        bundle.sample_truths.iter().map(|sample| sample.sample_id.as_str()).collect::<Vec<_>>();
    let expected_sample_ids =
        manifest.cases.iter().map(|case| case.sample_id.as_str()).collect::<Vec<_>>();
    if actual_sample_ids != expected_sample_ids {
        return Err(anyhow!(
            "BAM endogenous truth bundle `{}` must contain samples {:?}",
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
    sample: &BamEndogenousTruthSample,
    expected_path: &Path,
) -> Result<()> {
    if sample.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth bundle `{}` contains a sample with an empty sample_id",
            expected_path.display()
        ));
    }
    if sample.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` must declare a non-empty cohort",
            sample.sample_id
        ));
    }
    if sample.alignment_path.trim().is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` must declare a non-empty alignment_path",
            sample.sample_id
        ));
    }
    if sample.summary.schema_version != BAM_ENDOGENOUS_TRUTH_SUMMARY_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` uses summary schema `{}` instead of `{}`",
            sample.sample_id,
            sample.summary.schema_version,
            BAM_ENDOGENOUS_TRUTH_SUMMARY_SCHEMA_VERSION
        ));
    }
    if sample.summary.total_reads
        != sample.summary.mapped_reads.saturating_add(sample.summary.contaminant_reads)
    {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` total_reads {} does not equal mapped+contaminant {}",
            sample.sample_id,
            sample.summary.total_reads,
            sample.summary.mapped_reads + sample.summary.contaminant_reads
        ));
    }
    if sample.summary.retained_reads != sample.summary.mapped_reads {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` retained_reads {} must equal mapped_reads {}",
            sample.sample_id,
            sample.summary.retained_reads,
            sample.summary.mapped_reads
        ));
    }
    if sample.summary.endogenous_reads != sample.summary.retained_reads {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` endogenous_reads {} must equal retained_reads {}",
            sample.sample_id,
            sample.summary.endogenous_reads,
            sample.summary.retained_reads
        ));
    }
    let total_reads = checked_f64_from_u64(sample.summary.total_reads, "total reads")?;
    let expected_contaminant_fraction = if sample.summary.total_reads > 0 {
        checked_f64_from_u64(sample.summary.contaminant_reads, "contaminant reads")? / total_reads
    } else {
        0.0
    };
    let expected_retained_fraction = if sample.summary.total_reads > 0 {
        checked_f64_from_u64(sample.summary.retained_reads, "retained reads")? / total_reads
    } else {
        0.0
    };
    if (sample.summary.contaminant_fraction - expected_contaminant_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` contaminant_fraction drifted",
            sample.sample_id
        ));
    }
    if (sample.summary.retained_fraction - expected_retained_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` retained_fraction drifted",
            sample.sample_id
        ));
    }
    if (sample.summary.endogenous_fraction - sample.summary.retained_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` endogenous_fraction must equal retained_fraction",
            sample.sample_id
        ));
    }
    if sample.summary.count_provenance.trim().is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` must declare non-empty count provenance",
            sample.sample_id
        ));
    }
    Ok(())
}
