//! Admit declared coverage regions, GC windows and corpus membership.
use super::model::{BamCoverageTruthCase, BamGcBiasTruthCase, BamGcCoverageTruthManifest};
use super::{BAM_GC_COVERAGE_TRUTH_FIXTURE_ID, BAM_GC_COVERAGE_TRUTH_MANIFEST_SCHEMA_VERSION};
use crate::bam::validate_bam_corpus_fixture_manifest_path;
use crate::paths::resolve_manifest_relative_path as resolve_repo_relative_path;
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &BamGcCoverageTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != BAM_GC_COVERAGE_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            BAM_GC_COVERAGE_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != BAM_GC_COVERAGE_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            BAM_GC_COVERAGE_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }

    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    if !corpus_manifest_path.is_file() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth corpus manifest is missing: {}",
            corpus_manifest_path.display()
        ));
    }
    validate_bam_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;

    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }

    if manifest.coverage_cases.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest `{}` must declare at least one coverage case",
            manifest_path.display()
        ));
    }
    if manifest.gc_bias_cases.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest `{}` must declare at least one GC-bias case",
            manifest_path.display()
        ));
    }

    let mut coverage_sample_ids = BTreeSet::new();
    for case in &manifest.coverage_cases {
        validate_coverage_case_contract(repo_root, manifest_path, &mut coverage_sample_ids, case)?;
    }

    let mut gc_bias_sample_ids = BTreeSet::new();
    for case in &manifest.gc_bias_cases {
        validate_gc_bias_case_contract(repo_root, manifest_path, &mut gc_bias_sample_ids, case)?;
    }

    Ok(())
}

fn validate_coverage_case_contract(
    repo_root: &Path,
    manifest_path: &Path,
    coverage_sample_ids: &mut BTreeSet<String>,
    case: &BamCoverageTruthCase,
) -> Result<()> {
    if case.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest `{}` contains an empty coverage sample_id",
            manifest_path.display()
        ));
    }
    if !coverage_sample_ids.insert(case.sample_id.clone()) {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest repeats coverage sample_id `{}`",
            case.sample_id
        ));
    }
    if case.depth_thresholds.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` must declare depth thresholds",
            case.sample_id
        ));
    }
    let regions_path = resolve_repo_relative_path(repo_root, &case.regions_path);
    if !regions_path.is_file() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage regions path is missing: {}",
            regions_path.display()
        ));
    }
    Ok(())
}

fn validate_gc_bias_case_contract(
    repo_root: &Path,
    manifest_path: &Path,
    gc_bias_sample_ids: &mut BTreeSet<String>,
    case: &BamGcBiasTruthCase,
) -> Result<()> {
    if case.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest `{}` contains an empty GC-bias sample_id",
            manifest_path.display()
        ));
    }
    if !gc_bias_sample_ids.insert(case.sample_id.clone()) {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest repeats GC-bias sample_id `{}`",
            case.sample_id
        ));
    }
    if case.window_size == 0 {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` must declare window_size greater than zero",
            case.sample_id
        ));
    }
    let reference_path = resolve_repo_relative_path(repo_root, &case.reference_path);
    if !reference_path.is_file() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth reference path is missing: {}",
            reference_path.display()
        ));
    }
    Ok(())
}
