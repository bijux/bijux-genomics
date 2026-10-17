//! Read-only local evidence acquisition delegates interpretation to the VCF domain.
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_domain_vcf::{
    evaluate_diploid_calling_boundary, evaluate_genotype_likelihood_workflow_boundary,
    evaluate_pseudohaploid_calling_boundary, summarize_vcf_genotype_truth,
};

use super::model::{
    RawVcfGenotypeTruthCommand, VcfGenotypeEvaluationKind, VcfGenotypeTruthBundle,
    VcfGenotypeTruthCaseTruth, VcfGenotypeTruthCommandSummary, VcfGenotypeTruthManifest,
};
use super::{
    path_relative_to_repo, resolve_repo_relative_path, VCF_GENOTYPE_TRUTH_BUNDLE_SCHEMA_VERSION,
};

pub(super) fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &VcfGenotypeTruthManifest,
) -> Result<VcfGenotypeTruthBundle> {
    let mut cases = Vec::with_capacity(manifest.cases.len());
    for case in &manifest.cases {
        let input_vcf = resolve_repo_relative_path(repo_root, &case.input_vcf_path);
        let mut summary = summarize_vcf_genotype_truth(&input_vcf, &case.stage_id, &case.tool_id)?;
        summary.input_vcf = PathBuf::from(path_relative_to_repo(repo_root, &input_vcf));
        let command = case
            .command_path
            .as_ref()
            .map(|command_path| summarize_command(repo_root, command_path))
            .transpose()?;
        let diploid_boundary = if case.evaluation_kind == VcfGenotypeEvaluationKind::Diploid {
            Some(evaluate_diploid_calling_boundary(
                case.coverage.has_input_bam,
                case.coverage.has_reference_context,
                case.declared_ploidy.as_deref(),
                case.mean_coverage.unwrap_or(0.0),
                case.minimum_mean_coverage.unwrap_or(0.0),
            ))
        } else {
            None
        };
        let pseudohaploid_boundary =
            if case.evaluation_kind == VcfGenotypeEvaluationKind::Pseudohaploid {
                Some(evaluate_pseudohaploid_calling_boundary(
                    case.coverage.has_input_bam,
                    case.coverage.low_coverage_expected,
                    command.as_ref().and_then(|value| value.sampling_policy.as_deref()),
                    case.declared_ploidy.as_deref(),
                    case.likelihood.uncertainty_reported,
                ))
            } else {
                None
            };
        let likelihood_boundary = if case.evaluation_kind == VcfGenotypeEvaluationKind::Likelihood {
            Some(evaluate_genotype_likelihood_workflow_boundary(
                summary.likelihood_fields_present.iter().any(|field| field == "GL"),
                summary
                    .likelihood_fields_present
                    .iter()
                    .any(|field| matches!(field.as_str(), "GP" | "PL")),
                command.as_ref().and_then(|value| value.likelihood_model.as_deref()).is_some(),
                case.likelihood.downstream_gl_compatible,
                case.likelihood.uncertainty_reported,
            ))
        } else {
            None
        };
        cases.push(VcfGenotypeTruthCaseTruth {
            case_id: case.case_id.clone(),
            summary,
            command,
            diploid_boundary,
            pseudohaploid_boundary,
            likelihood_boundary,
        });
    }
    Ok(VcfGenotypeTruthBundle {
        schema_version: VCF_GENOTYPE_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        cases,
    })
}

fn summarize_command(
    repo_root: &Path,
    command_path: &Path,
) -> Result<VcfGenotypeTruthCommandSummary> {
    let absolute_path = resolve_repo_relative_path(repo_root, command_path);
    let raw = fs::read_to_string(&absolute_path)
        .with_context(|| format!("read {}", absolute_path.display()))?;
    let command: RawVcfGenotypeTruthCommand =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", absolute_path.display()))?;
    let seed_argument = extract_seed_argument(&command.argv);
    let seed_argument_matches_random_seed = match (command.random_seed, seed_argument) {
        (Some(random_seed), Some(seed_argument)) => Some(random_seed == seed_argument),
        _ => None,
    };
    Ok(VcfGenotypeTruthCommandSummary {
        command_path: path_relative_to_repo(repo_root, &absolute_path),
        stage_id: command.stage_id,
        tool_id: command.tool_id,
        sampling_policy: command.sampling_policy,
        likelihood_model: command.likelihood_model,
        random_seed: command.random_seed,
        seed_argument,
        seed_argument_matches_random_seed,
    })
}

fn extract_seed_argument(argv: &[String]) -> Option<u64> {
    argv.windows(2).find_map(|window| match window {
        [flag, value] if matches!(flag.as_str(), "-seed" | "--seed") => value.parse().ok(),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::extract_seed_argument;

    #[test]
    fn extract_seed_argument_reads_short_and_long_flags() {
        assert_eq!(
            extract_seed_argument(&["angsd".to_string(), "-seed".to_string(), "42".to_string()]),
            Some(42)
        );
        assert_eq!(
            extract_seed_argument(&["tool".to_string(), "--seed".to_string(), "99".to_string()]),
            Some(99)
        );
        assert_eq!(extract_seed_argument(&["tool".to_string()]), None);
    }
}
