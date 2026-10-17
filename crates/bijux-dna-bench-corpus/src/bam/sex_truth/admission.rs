//! Admit sex-inference declarations and corpus membership before expected acquisition.
use super::model::BamSexTruthManifest;
use super::{BAM_SEX_TRUTH_FIXTURE_ID, BAM_SEX_TRUTH_MANIFEST_SCHEMA_VERSION};
use crate::bam::validate_bam_corpus_fixture_manifest_path;
use crate::paths::resolve_manifest_relative_path as resolve_repo_relative_path;
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &BamSexTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != BAM_SEX_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM sex truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            BAM_SEX_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != BAM_SEX_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "BAM sex truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            BAM_SEX_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "BAM sex truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    if !corpus_manifest_path.is_file() {
        return Err(anyhow!(
            "BAM sex truth corpus manifest is missing: {}",
            corpus_manifest_path.display()
        ));
    }
    validate_bam_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "BAM sex truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "BAM sex truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "BAM sex truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }

    let mut sample_ids = BTreeSet::new();
    for case in &manifest.cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "BAM sex truth manifest `{}` contains an empty sample_id",
                manifest_path.display()
            ));
        }
        if !sample_ids.insert(case.sample_id.clone()) {
            return Err(anyhow!("BAM sex truth manifest repeats sample_id `{}`", case.sample_id));
        }
        if case.method.trim().is_empty() {
            return Err(anyhow!(
                "BAM sex truth case `{}` must declare a non-empty method",
                case.sample_id
            ));
        }
        if case.chromosome_system.trim().is_empty() {
            return Err(anyhow!(
                "BAM sex truth case `{}` must declare a non-empty chromosome_system",
                case.sample_id
            ));
        }
    }

    Ok(())
}
