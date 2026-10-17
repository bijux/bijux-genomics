//! Read explicit local fixture evidence through canonical domain parsers and summaries.
use super::admission::parse_supported_stage;
use super::{
    VcfDamageFilterMetricsTruth, VcfFilterMetricsTruth, VcfFilterTruthBundle,
    VcfFilterTruthCaseTruth, VcfFilterTruthKind, VcfFilterTruthManifest,
    VCF_FILTER_TRUTH_BUNDLE_SCHEMA_VERSION,
};
use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::{
    parse_angsd_stage_metrics, parse_bcftools_stage_metrics, summarize_vcf_filter_output_truth,
    VcfDomainStage, VcfFilterOutputTruthSummaryV1,
};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn load_vcf_filter_truth_manifest_path(
    manifest_path: &Path,
) -> Result<VcfFilterTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn load_vcf_filter_truth_bundle(expected_path: &Path) -> Result<VcfFilterTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

pub(super) fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &VcfFilterTruthManifest,
) -> Result<VcfFilterTruthBundle> {
    let mut cases = Vec::with_capacity(manifest.cases.len());
    for case in &manifest.cases {
        let stage = parse_supported_stage(&case.stage_id)?;
        let artifact_root = resolve_repo_relative_path(repo_root, &case.artifact_root_path);
        let output_truth = case
            .output_vcf_path
            .as_ref()
            .map(|path| -> Result<VcfFilterOutputTruthSummaryV1> {
                let resolved = resolve_repo_relative_path(repo_root, path);
                let mut summary =
                    summarize_vcf_filter_output_truth(&resolved, &case.stage_id, &case.tool_id)?;
                summary.input_vcf = PathBuf::from(path_relative_to_repo(repo_root, &resolved));
                Ok(summary)
            })
            .transpose()?;
        let case_truth = match case.truth_kind {
            VcfFilterTruthKind::FilterLabels => VcfFilterTruthCaseTruth {
                case_id: case.case_id.clone(),
                filter_metrics: Some(parse_filter_metrics_truth(
                    &case.tool_id,
                    stage,
                    &artifact_root,
                )?),
                damage_filter_metrics: None,
                output_truth,
            },
            VcfFilterTruthKind::DamageRemoval => VcfFilterTruthCaseTruth {
                case_id: case.case_id.clone(),
                filter_metrics: None,
                damage_filter_metrics: Some(parse_damage_filter_metrics_truth(
                    &case.tool_id,
                    stage,
                    &artifact_root,
                )?),
                output_truth,
            },
        };
        cases.push(case_truth);
    }
    Ok(VcfFilterTruthBundle {
        schema_version: VCF_FILTER_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        cases,
    })
}

fn parse_filter_metrics_truth(
    tool_id: &str,
    stage: VcfDomainStage,
    artifact_root: &Path,
) -> Result<VcfFilterMetricsTruth> {
    let raw = match tool_id {
        "bcftools" => parse_bcftools_stage_metrics(stage, artifact_root)?,
        other => {
            return Err(anyhow!(
                "VCF filter truth does not support filter metrics for tool `{other}`"
            ))
        }
    };
    serde_json::from_value(raw).context("deserialize VCF filter metrics truth")
}

fn parse_damage_filter_metrics_truth(
    tool_id: &str,
    stage: VcfDomainStage,
    artifact_root: &Path,
) -> Result<VcfDamageFilterMetricsTruth> {
    let raw = match tool_id {
        "bcftools" => parse_bcftools_stage_metrics(stage, artifact_root)?,
        "angsd" => parse_angsd_stage_metrics(stage, artifact_root)?,
        other => {
            return Err(anyhow!(
                "VCF filter truth does not support damage-filter metrics for tool `{other}`"
            ))
        }
    };
    serde_json::from_value(raw).context("deserialize VCF damage-filter metrics truth")
}

pub(super) fn resolve_repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

pub(super) fn resolve_fixture_path(fixture_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        fixture_root.join(path)
    }
}

pub(super) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
