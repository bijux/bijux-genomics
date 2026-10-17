//! Admit declared contamination source membership and expected case identity.
use bijux_dna_core::id_catalog::BAM_CONTAMINATION;
use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_domain_bam::BAM_ADNA_CONTAMINATION_TRUTH_SUMMARY_SCHEMA_VERSION;

use super::model::{
    AdnaContaminationTruthBundle, AdnaContaminationTruthCase, AdnaContaminationTruthManifest,
};
use super::{
    ADNA_CONTAMINATION_TRUTH_BUNDLE_SCHEMA_VERSION, ADNA_CONTAMINATION_TRUTH_FIXTURE_ID,
    ADNA_CONTAMINATION_TRUTH_MANIFEST_SCHEMA_VERSION,
};
use crate::paths::resolve_manifest_relative_path as resolve_repo_relative_path;

pub(super) fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &AdnaContaminationTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != ADNA_CONTAMINATION_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "aDNA contamination truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            ADNA_CONTAMINATION_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != ADNA_CONTAMINATION_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "aDNA contamination truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            ADNA_CONTAMINATION_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "aDNA contamination truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "aDNA contamination truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for path in &manifest.source_paths {
        let absolute = resolve_repo_relative_path(repo_root, path);
        if !absolute.exists() {
            return Err(anyhow!(
                "aDNA contamination truth manifest source path is missing: {}",
                absolute.display()
            ));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "aDNA contamination truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }

    let mut case_ids = BTreeSet::new();
    for case in &manifest.cases {
        validate_adna_contamination_case_contract(repo_root, manifest_path, &mut case_ids, case)?;
    }

    Ok(())
}

fn validate_adna_contamination_case_contract(
    repo_root: &Path,
    manifest_path: &Path,
    case_ids: &mut BTreeSet<String>,
    case: &AdnaContaminationTruthCase,
) -> Result<()> {
    if case.case_id.trim().is_empty() {
        return Err(anyhow!(
            "aDNA contamination truth manifest `{}` contains an empty case_id",
            manifest_path.display()
        ));
    }
    if !case_ids.insert(case.case_id.clone()) {
        return Err(anyhow!(
            "aDNA contamination truth manifest repeats case_id `{}`",
            case.case_id
        ));
    }
    if case.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` must declare a non-empty sample_id",
            case.case_id
        ));
    }
    if case.cohort.trim().is_empty() {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` must declare a non-empty cohort",
            case.case_id
        ));
    }
    if case.tool != "schmutzi" && case.tool != "verifybamid2" && case.tool != "contammix" {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` must use schmutzi, verifybamid2, or contammix",
            case.case_id
        ));
    }
    if case.minimum_mean_coverage < 0.0 || case.observed_mean_coverage < 0.0 {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` must keep coverage non-negative",
            case.case_id
        ));
    }
    if !(0.0..=1.0).contains(&case.raw_estimate)
        || !(0.0..=1.0).contains(&case.raw_ci_low)
        || !(0.0..=1.0).contains(&case.raw_ci_high)
    {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` must keep contamination values within [0, 1]",
            case.case_id
        ));
    }
    if case.raw_ci_low > case.raw_estimate || case.raw_estimate > case.raw_ci_high {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` must keep ci_low <= estimate <= ci_high",
            case.case_id
        ));
    }

    let alignment_path = resolve_repo_relative_path(repo_root, &case.alignment_path);
    if !alignment_path.is_file() {
        return Err(anyhow!(
            "aDNA contamination truth alignment path is missing for case `{}`: {}",
            case.case_id,
            alignment_path.display()
        ));
    }
    let reference_path = resolve_repo_relative_path(repo_root, &case.reference_path);
    if !reference_path.is_file() {
        return Err(anyhow!(
            "aDNA contamination truth reference path is missing for case `{}`: {}",
            case.case_id,
            reference_path.display()
        ));
    }

    Ok(())
}

pub(super) fn validate_bundle_contract(
    manifest: &AdnaContaminationTruthManifest,
    bundle: &AdnaContaminationTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != ADNA_CONTAMINATION_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "aDNA contamination truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            ADNA_CONTAMINATION_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "aDNA contamination truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }

    let expected_case_ids =
        manifest.cases.iter().map(|case| case.case_id.as_str()).collect::<BTreeSet<_>>();
    let observed_case_ids =
        bundle.truths.iter().map(|truth| truth.case_id.as_str()).collect::<BTreeSet<_>>();
    if expected_case_ids != observed_case_ids {
        return Err(anyhow!(
            "aDNA contamination truth bundle `{}` must contain cases {:?}",
            expected_path.display(),
            expected_case_ids
        ));
    }
    for truth in &bundle.truths {
        if truth.case_id.trim().is_empty()
            || truth.sample_id.trim().is_empty()
            || truth.cohort.trim().is_empty()
            || truth.tool.trim().is_empty()
        {
            return Err(anyhow!(
                "aDNA contamination truth bundle `{}` contains an incomplete case header",
                expected_path.display()
            ));
        }
        if truth.alignment_path.trim().is_empty() || truth.reference_path.trim().is_empty() {
            return Err(anyhow!(
                "aDNA contamination truth case `{}` must declare non-empty input paths",
                truth.case_id
            ));
        }
        if truth.summary.schema_version != BAM_ADNA_CONTAMINATION_TRUTH_SUMMARY_SCHEMA_VERSION {
            return Err(anyhow!(
                "aDNA contamination truth case `{}` uses schema `{}` instead of `{}`",
                truth.case_id,
                truth.summary.schema_version,
                BAM_ADNA_CONTAMINATION_TRUTH_SUMMARY_SCHEMA_VERSION
            ));
        }
        if truth.summary.stage_id != BAM_CONTAMINATION {
            return Err(anyhow!(
                "aDNA contamination truth case `{}` must resolve to stage_id `bam.contamination`",
                truth.case_id
            ));
        }
    }

    Ok(())
}
