//! Acquire declared local files and delegate damage interpretation to the BAM domain.
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::{
    inspect_tiny_alignment, metrics::DamageMetricsV1, summarize_tiny_bam_adna_damage_truth,
};

use super::model::{
    AdnaDamageSampleTruth, AdnaDamageTruthBundle, AdnaDamageTruthCase, AdnaDamageTruthManifest,
};
use super::ADNA_DAMAGE_TRUTH_BUNDLE_SCHEMA_VERSION;
use crate::paths::{
    path_relative_to_repo, resolve_manifest_relative_path as resolve_repo_relative_path,
};

pub(super) fn load_adna_damage_truth_manifest_path(
    manifest_path: &Path,
) -> Result<AdnaDamageTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn load_adna_damage_truth_bundle(expected_path: &Path) -> Result<AdnaDamageTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

pub(super) fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &AdnaDamageTruthManifest,
) -> Result<AdnaDamageTruthBundle> {
    let truths = manifest
        .cases
        .iter()
        .map(|case| build_actual_case_truth(repo_root, case))
        .collect::<Result<Vec<_>>>()?;
    Ok(AdnaDamageTruthBundle {
        schema_version: ADNA_DAMAGE_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        truths,
    })
}

fn build_actual_case_truth(
    repo_root: &Path,
    case: &AdnaDamageTruthCase,
) -> Result<AdnaDamageSampleTruth> {
    let alignment_path = resolve_repo_relative_path(repo_root, &case.alignment_path);
    let reference_path = resolve_repo_relative_path(repo_root, &case.reference_path);
    let inspection = inspect_tiny_alignment(&alignment_path)?;
    if !inspection.header_sample_ids.iter().any(|sample_id| sample_id == &case.sample_id) {
        return Err(anyhow!(
            "aDNA damage truth case `{}` sample_id `{}` is not present in alignment header samples",
            case.case_id,
            case.sample_id
        ));
    }
    let summary = summarize_tiny_bam_adna_damage_truth(
        &alignment_path,
        &DamageMetricsV1 {
            c_to_t_5p: case.terminal_c_to_t_5p,
            g_to_a_3p: case.terminal_g_to_a_3p,
            pmd_score_histogram: Vec::new(),
        },
        case.strict_profile,
        case.udg_model,
    )?;

    Ok(AdnaDamageSampleTruth {
        case_id: case.case_id.clone(),
        sample_id: case.sample_id.clone(),
        cohort: case.cohort.clone(),
        alignment_path: path_relative_to_repo(repo_root, &alignment_path),
        reference_path: path_relative_to_repo(repo_root, &reference_path),
        summary,
    })
}
