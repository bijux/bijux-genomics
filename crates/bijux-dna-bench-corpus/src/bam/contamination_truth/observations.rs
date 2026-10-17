//! Read local contamination declarations and consume existing domain summaries.
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::{
    inspect_tiny_alignment,
    metrics::{BamMetricsV1, ContaminationMetricsV1},
    summarize_bam_adna_contamination_truth,
};

use super::model::{
    AdnaContaminationSampleTruth, AdnaContaminationTruthBundle, AdnaContaminationTruthCase,
    AdnaContaminationTruthManifest,
};
use super::ADNA_CONTAMINATION_TRUTH_BUNDLE_SCHEMA_VERSION;
use crate::paths::{
    path_relative_to_repo, resolve_manifest_relative_path as resolve_repo_relative_path,
};

pub(super) fn load_adna_contamination_truth_manifest_path(
    manifest_path: &Path,
) -> Result<AdnaContaminationTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn load_adna_contamination_truth_bundle(
    expected_path: &Path,
) -> Result<AdnaContaminationTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

pub(super) fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &AdnaContaminationTruthManifest,
) -> Result<AdnaContaminationTruthBundle> {
    let truths = manifest
        .cases
        .iter()
        .map(|case| build_actual_case_truth(repo_root, case))
        .collect::<Result<Vec<_>>>()?;
    Ok(AdnaContaminationTruthBundle {
        schema_version: ADNA_CONTAMINATION_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        truths,
    })
}

fn build_actual_case_truth(
    repo_root: &Path,
    case: &AdnaContaminationTruthCase,
) -> Result<AdnaContaminationSampleTruth> {
    let alignment_path = resolve_repo_relative_path(repo_root, &case.alignment_path);
    let reference_path = resolve_repo_relative_path(repo_root, &case.reference_path);
    let inspection = inspect_tiny_alignment(&alignment_path)?;
    if !inspection.header_sample_ids.iter().any(|sample_id| sample_id == &case.sample_id) {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` sample_id `{}` is not present in alignment header samples",
            case.case_id,
            case.sample_id
        ));
    }

    let mut metrics = BamMetricsV1::empty();
    metrics.coverage.mean = case.observed_mean_coverage;
    metrics.contamination = ContaminationMetricsV1 {
        method: case.tool.clone(),
        estimate: case.raw_estimate,
        ci_low: case.raw_ci_low,
        ci_high: case.raw_ci_high,
        assumptions: vec![
            format!("tool:{}", case.tool),
            format!("case:{}", case.case_id),
            format!("sample:{}", case.sample_id),
        ],
    };

    let summary = summarize_bam_adna_contamination_truth(
        &case.tool,
        &metrics,
        case.minimum_mean_coverage,
        case.context.has_mito_reference,
        case.context.has_damage_context,
        case.context.has_reference_panel,
        case.compatibility.panel_build_compatible,
        case.compatibility.sex_context_available,
    )?;

    Ok(AdnaContaminationSampleTruth {
        case_id: case.case_id.clone(),
        sample_id: case.sample_id.clone(),
        cohort: case.cohort.clone(),
        tool: case.tool.clone(),
        alignment_path: path_relative_to_repo(repo_root, &alignment_path),
        reference_path: path_relative_to_repo(repo_root, &reference_path),
        summary,
    })
}
