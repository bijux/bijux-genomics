//! Native segment metrics and scientific summaries retain their VCF domain authority.
use super::admission::parse_supported_stage;
use super::model::{
    SegmentsDemographyTruthBundle, SegmentsDemographyTruthCaseTruth,
    SegmentsDemographyTruthManifest,
};
use super::{resolve_repo_relative_path, SEGMENTS_DEMOGRAPHY_TRUTH_BUNDLE_SCHEMA_VERSION};
use anyhow::Result;
use bijux_dna_domain_vcf::{
    parse_segment_stage_metrics, summarize_vcf_demography_output_truth,
    summarize_vcf_ibd_output_truth, summarize_vcf_roh_output_truth, VcfDomainStage,
};
use std::path::Path;

pub(super) fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &SegmentsDemographyTruthManifest,
) -> Result<SegmentsDemographyTruthBundle> {
    let cases = manifest
        .cases
        .iter()
        .map(|case| {
            let stage = parse_supported_stage(&case.stage_id)?;
            let artifact_root = resolve_repo_relative_path(repo_root, &case.artifact_root_path);
            let metrics = parse_segment_stage_metrics(&case.tool_id, stage, &artifact_root)?;
            let case_truth = match stage {
                VcfDomainStage::Roh => SegmentsDemographyTruthCaseTruth {
                    case_id: case.case_id.clone(),
                    roh_metrics: Some(metrics.clone()),
                    ibd_metrics: None,
                    demography_metrics: None,
                    roh_output_truth: Some(summarize_vcf_roh_output_truth(&metrics)?),
                    ibd_output_truth: None,
                    demography_output_truth: None,
                },
                VcfDomainStage::Ibd => SegmentsDemographyTruthCaseTruth {
                    case_id: case.case_id.clone(),
                    roh_metrics: None,
                    ibd_metrics: Some(metrics.clone()),
                    demography_metrics: None,
                    roh_output_truth: None,
                    ibd_output_truth: Some(summarize_vcf_ibd_output_truth(&metrics)?),
                    demography_output_truth: None,
                },
                VcfDomainStage::Demography => SegmentsDemographyTruthCaseTruth {
                    case_id: case.case_id.clone(),
                    roh_metrics: None,
                    ibd_metrics: None,
                    demography_metrics: Some(metrics.clone()),
                    roh_output_truth: None,
                    ibd_output_truth: None,
                    demography_output_truth: Some(summarize_vcf_demography_output_truth(&metrics)?),
                },
                _ => unreachable!("stage already constrained"),
            };
            Ok(case_truth)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(SegmentsDemographyTruthBundle {
        schema_version: SEGMENTS_DEMOGRAPHY_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        cases,
    })
}
