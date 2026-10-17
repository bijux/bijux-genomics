//! Native parser and output-truth acquisition retain the VCF domain authority.
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::{
    parse_imputation_stage_metrics, parse_phasing_stage_metrics,
    summarize_vcf_imputation_output_truth, summarize_vcf_phasing_output_truth, VcfDomainStage,
};

use super::admission::parse_supported_stage;
use super::model::{
    PhasingImputationTruthBundle, PhasingImputationTruthCaseTruth, PhasingImputationTruthManifest,
};
use super::{
    path_relative_to_repo, resolve_repo_relative_path,
    PHASING_IMPUTATION_TRUTH_BUNDLE_SCHEMA_VERSION,
};

pub(super) fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &PhasingImputationTruthManifest,
) -> Result<PhasingImputationTruthBundle> {
    let mut cases = Vec::with_capacity(manifest.cases.len());
    for case in &manifest.cases {
        let stage = parse_supported_stage(&case.stage_id)?;
        let artifact_root = resolve_repo_relative_path(repo_root, &case.artifact_root_path);
        let output_vcf = resolve_repo_relative_path(repo_root, &case.output_vcf_path);
        let truth_vcf =
            case.truth_vcf_path.as_ref().map(|path| resolve_repo_relative_path(repo_root, path));
        let case_truth = match stage {
            VcfDomainStage::Phasing => {
                let raw = parse_phasing_stage_metrics(&case.tool_id, &artifact_root)?;
                let mut output_truth =
                    summarize_vcf_phasing_output_truth(&output_vcf, &case.stage_id, &case.tool_id)?;
                output_truth.input_vcf =
                    PathBuf::from(path_relative_to_repo(repo_root, &output_vcf));
                PhasingImputationTruthCaseTruth {
                    case_id: case.case_id.clone(),
                    phasing_metrics: Some(
                        serde_json::from_value(raw).context("deserialize phasing truth metrics")?,
                    ),
                    impute_metrics: None,
                    imputation_metrics: None,
                    phasing_output_truth: Some(output_truth),
                    imputation_output_truth: None,
                }
            }
            VcfDomainStage::Impute => {
                let raw = parse_imputation_stage_metrics(&case.tool_id, stage, &artifact_root)?;
                let mut output_truth = summarize_vcf_imputation_output_truth(
                    &output_vcf,
                    truth_vcf.as_deref(),
                    &case.stage_id,
                    &case.tool_id,
                )?;
                output_truth.input_vcf =
                    PathBuf::from(path_relative_to_repo(repo_root, &output_vcf));
                output_truth.truth_vcf = truth_vcf
                    .as_ref()
                    .map(|path| PathBuf::from(path_relative_to_repo(repo_root, path)));
                PhasingImputationTruthCaseTruth {
                    case_id: case.case_id.clone(),
                    phasing_metrics: None,
                    impute_metrics: Some(
                        serde_json::from_value(raw).context("deserialize impute truth metrics")?,
                    ),
                    imputation_metrics: None,
                    phasing_output_truth: None,
                    imputation_output_truth: Some(output_truth),
                }
            }
            VcfDomainStage::ImputationMetrics => {
                let raw = parse_imputation_stage_metrics(&case.tool_id, stage, &artifact_root)?;
                let mut output_truth = summarize_vcf_imputation_output_truth(
                    &output_vcf,
                    truth_vcf.as_deref(),
                    &case.stage_id,
                    &case.tool_id,
                )?;
                output_truth.input_vcf =
                    PathBuf::from(path_relative_to_repo(repo_root, &output_vcf));
                output_truth.truth_vcf = truth_vcf
                    .as_ref()
                    .map(|path| PathBuf::from(path_relative_to_repo(repo_root, path)));
                PhasingImputationTruthCaseTruth {
                    case_id: case.case_id.clone(),
                    phasing_metrics: None,
                    impute_metrics: None,
                    imputation_metrics: Some(
                        serde_json::from_value(raw)
                            .context("deserialize imputation-metrics truth")?,
                    ),
                    phasing_output_truth: None,
                    imputation_output_truth: Some(output_truth),
                }
            }
            other => {
                return Err(anyhow!(
                    "unsupported phasing/imputation truth stage `{}`",
                    other.as_str()
                ))
            }
        };
        cases.push(case_truth);
    }

    Ok(PhasingImputationTruthBundle {
        schema_version: PHASING_IMPUTATION_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        cases,
    })
}
