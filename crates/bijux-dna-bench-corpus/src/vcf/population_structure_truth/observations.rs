//! Native metrics and metadata-aware scientific truth retain their domain authority.
use super::admission::parse_supported_stage;
use super::model::{
    PopulationStructureTruthBundle, PopulationStructureTruthCaseTruth,
    PopulationStructureTruthManifest,
};
use super::{
    path_relative_to_repo, resolve_repo_relative_path,
    POPULATION_STRUCTURE_TRUTH_BUNDLE_SCHEMA_VERSION,
};
use anyhow::{anyhow, Result};
use bijux_dna_domain_vcf::{
    parse_eigensoft_stage_metrics, parse_plink2_stage_metrics,
    summarize_vcf_admixture_output_truth, summarize_vcf_pca_output_truth,
    summarize_vcf_population_structure_output_truth, VcfDomainStage,
};
use std::path::{Path, PathBuf};

pub(super) fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &PopulationStructureTruthManifest,
) -> Result<PopulationStructureTruthBundle> {
    let cases = manifest
        .cases
        .iter()
        .map(|case| {
            let stage = parse_supported_stage(&case.stage_id)?;
            let artifact_root = resolve_repo_relative_path(repo_root, &case.artifact_root_path);
            let metadata_path = resolve_repo_relative_path(repo_root, &case.sample_metadata_path);
            let metrics = match case.tool_id.as_str() {
                "plink2" => parse_plink2_stage_metrics(stage, &artifact_root)?,
                "eigensoft" => parse_eigensoft_stage_metrics(stage, &artifact_root)?,
                other => {
                    return Err(anyhow!(
                        "population-structure truth case `{}` does not support tool `{other}`",
                        case.case_id
                    ))
                }
            };
            let case_truth = match stage {
                VcfDomainStage::Pca => {
                    let mut output_truth =
                        summarize_vcf_pca_output_truth(&metrics, &metadata_path)?;
                    output_truth.sample_metadata_path =
                        PathBuf::from(path_relative_to_repo(repo_root, &metadata_path));
                    PopulationStructureTruthCaseTruth {
                        case_id: case.case_id.clone(),
                        pca_metrics: Some(metrics),
                        admixture_metrics: None,
                        population_structure_metrics: None,
                        pca_output_truth: Some(output_truth),
                        admixture_output_truth: None,
                        population_structure_output_truth: None,
                    }
                }
                VcfDomainStage::Admixture => {
                    let mut output_truth =
                        summarize_vcf_admixture_output_truth(&metrics, &metadata_path)?;
                    output_truth.sample_metadata_path =
                        PathBuf::from(path_relative_to_repo(repo_root, &metadata_path));
                    PopulationStructureTruthCaseTruth {
                        case_id: case.case_id.clone(),
                        pca_metrics: None,
                        admixture_metrics: Some(metrics),
                        population_structure_metrics: None,
                        pca_output_truth: None,
                        admixture_output_truth: Some(output_truth),
                        population_structure_output_truth: None,
                    }
                }
                VcfDomainStage::PopulationStructure => {
                    let mut output_truth =
                        summarize_vcf_population_structure_output_truth(&metrics, &metadata_path)?;
                    output_truth.sample_metadata_path =
                        PathBuf::from(path_relative_to_repo(repo_root, &metadata_path));
                    PopulationStructureTruthCaseTruth {
                        case_id: case.case_id.clone(),
                        pca_metrics: None,
                        admixture_metrics: None,
                        population_structure_metrics: Some(metrics),
                        pca_output_truth: None,
                        admixture_output_truth: None,
                        population_structure_output_truth: Some(output_truth),
                    }
                }
                _ => unreachable!("stage already constrained"),
            };
            Ok(case_truth)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(PopulationStructureTruthBundle {
        schema_version: POPULATION_STRUCTURE_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        cases,
    })
}
