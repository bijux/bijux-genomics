#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::{
    evaluate_diploid_calling_boundary, evaluate_genotype_likelihood_workflow_boundary,
    evaluate_pseudohaploid_calling_boundary, summarize_vcf_genotype_truth, VcfCallingBoundaryV1,
    VcfGenotypeTruthSummaryV1, VcfLikelihoodWorkflowBoundaryV1,
};
use serde::{Deserialize, Serialize};

pub(crate) const VCF_GENOTYPE_TRUTH_FIXTURE_ID: &str = "vcf-genotype-truth";
pub(crate) const VCF_GENOTYPE_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.vcf_genotype_truth.v1";
const VCF_GENOTYPE_TRUTH_BUNDLE_SCHEMA_VERSION: &str = "bijux.bench.vcf_genotype_truth.expected.v1";
const VCF_GENOTYPE_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.vcf_genotype_truth.validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct VcfGenotypeTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    expected_path: PathBuf,
    source_paths: Vec<PathBuf>,
    cases: Vec<VcfGenotypeTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct VcfGenotypeTruthCase {
    case_id: String,
    stage_id: String,
    tool_id: String,
    input_vcf_path: PathBuf,
    #[serde(default)]
    command_path: Option<PathBuf>,
    evaluation_kind: VcfGenotypeEvaluationKind,
    #[serde(default)]
    declared_ploidy: Option<String>,
    #[serde(default, flatten)]
    coverage: VcfGenotypeTruthCaseCoverageSignals,
    #[serde(default, flatten)]
    likelihood: VcfGenotypeTruthCaseLikelihoodSignals,
    #[serde(default)]
    mean_coverage: Option<f64>,
    #[serde(default)]
    minimum_mean_coverage: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct VcfGenotypeTruthCaseCoverageSignals {
    #[serde(default)]
    has_input_bam: bool,
    #[serde(default)]
    has_reference_context: bool,
    #[serde(default)]
    low_coverage_expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct VcfGenotypeTruthCaseLikelihoodSignals {
    #[serde(default)]
    downstream_gl_compatible: bool,
    #[serde(default)]
    uncertainty_reported: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum VcfGenotypeEvaluationKind {
    None,
    Diploid,
    Pseudohaploid,
    Likelihood,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfGenotypeTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_path: String,
    pub(crate) validated_case_count: usize,
    pub(crate) validated_stage_ids: Vec<String>,
    pub(crate) validated_tool_ids: Vec<String>,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfGenotypeTruthBundle {
    schema_version: String,
    fixture_id: String,
    cases: Vec<VcfGenotypeTruthCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfGenotypeTruthCaseTruth {
    case_id: String,
    summary: VcfGenotypeTruthSummaryV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    command: Option<VcfGenotypeTruthCommandSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    diploid_boundary: Option<VcfCallingBoundaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pseudohaploid_boundary: Option<VcfCallingBoundaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    likelihood_boundary: Option<VcfLikelihoodWorkflowBoundaryV1>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfGenotypeTruthCommandSummary {
    command_path: String,
    #[serde(default)]
    stage_id: Option<String>,
    #[serde(default)]
    tool_id: Option<String>,
    #[serde(default)]
    sampling_policy: Option<String>,
    #[serde(default)]
    likelihood_model: Option<String>,
    #[serde(default)]
    random_seed: Option<u64>,
    #[serde(default)]
    seed_argument: Option<u64>,
    #[serde(default)]
    seed_argument_matches_random_seed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawVcfGenotypeTruthCommand {
    #[serde(default)]
    stage_id: Option<String>,
    #[serde(default)]
    tool_id: Option<String>,
    #[serde(default)]
    sampling_policy: Option<String>,
    #[serde(default)]
    likelihood_model: Option<String>,
    #[serde(default)]
    random_seed: Option<u64>,
    #[serde(default)]
    argv: Vec<String>,
}

pub(crate) fn validate_vcf_genotype_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<VcfGenotypeTruthValidationReport> {
    let manifest = load_vcf_genotype_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("VCF genotype truth manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("VCF genotype truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_vcf_genotype_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest)?;
    let expected_map =
        expected.cases.iter().map(|case| (case.case_id.as_str(), case)).collect::<BTreeMap<_, _>>();
    let actual_map =
        actual.cases.iter().map(|case| (case.case_id.as_str(), case)).collect::<BTreeMap<_, _>>();
    if expected_map.len() != actual_map.len() {
        return Err(anyhow!(
            "VCF genotype truth case count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for case in &manifest.cases {
        let expected_case = expected_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("expected VCF genotype truth is missing case `{}`", case.case_id)
        })?;
        let actual_case = actual_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("observed VCF genotype truth is missing case `{}`", case.case_id)
        })?;
        if expected_case != actual_case {
            return Err(anyhow!(
                "VCF genotype truth drifted for case `{}`\nexpected: {expected_case:#?}\nobserved: {actual_case:#?}",
                case.case_id
            ));
        }
    }

    let validated_stage_ids = collect_stage_ids(&actual.cases);
    let validated_tool_ids = collect_tool_ids(&actual.cases);
    Ok(VcfGenotypeTruthValidationReport {
        schema_version: VCF_GENOTYPE_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.cases.len(),
        validated_stage_ids,
        validated_tool_ids,
        valid: true,
    })
}

fn load_vcf_genotype_truth_manifest_path(manifest_path: &Path) -> Result<VcfGenotypeTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &VcfGenotypeTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != VCF_GENOTYPE_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "VCF genotype truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            VCF_GENOTYPE_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != VCF_GENOTYPE_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "VCF genotype truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            VCF_GENOTYPE_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "VCF genotype truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "VCF genotype truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF genotype truth source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "VCF genotype truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }

    let mut case_ids = BTreeSet::new();
    for case in &manifest.cases {
        validate_case_contract(repo_root, case, manifest_path)?;
        if !case_ids.insert(case.case_id.clone()) {
            return Err(anyhow!("VCF genotype truth manifest repeats case_id `{}`", case.case_id));
        }
    }
    Ok(())
}

fn validate_case_contract(
    repo_root: &Path,
    case: &VcfGenotypeTruthCase,
    manifest_path: &Path,
) -> Result<()> {
    if case.case_id.trim().is_empty() {
        return Err(anyhow!(
            "VCF genotype truth manifest `{}` contains an empty case_id",
            manifest_path.display()
        ));
    }
    if case.stage_id.trim().is_empty() || case.tool_id.trim().is_empty() {
        return Err(anyhow!(
            "VCF genotype truth case `{}` must declare non-empty stage_id and tool_id",
            case.case_id
        ));
    }
    let input_vcf_path = resolve_repo_relative_path(repo_root, &case.input_vcf_path);
    if !input_vcf_path.is_file() {
        return Err(anyhow!(
            "VCF genotype truth case `{}` input VCF is missing: {}",
            case.case_id,
            input_vcf_path.display()
        ));
    }
    if let Some(command_path) = &case.command_path {
        let resolved = resolve_repo_relative_path(repo_root, command_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF genotype truth case `{}` command file is missing: {}",
                case.case_id,
                resolved.display()
            ));
        }
    }
    match case.evaluation_kind {
        VcfGenotypeEvaluationKind::None | VcfGenotypeEvaluationKind::Likelihood => {}
        VcfGenotypeEvaluationKind::Diploid => {
            if case.declared_ploidy.as_deref() != Some("diploid") {
                return Err(anyhow!(
                    "VCF genotype truth diploid case `{}` must declare `diploid` ploidy",
                    case.case_id
                ));
            }
            if case.mean_coverage.is_none() || case.minimum_mean_coverage.is_none() {
                return Err(anyhow!(
                    "VCF genotype truth diploid case `{}` must declare mean coverage and minimum_mean_coverage",
                    case.case_id
                ));
            }
        }
        VcfGenotypeEvaluationKind::Pseudohaploid => {
            if !matches!(case.declared_ploidy.as_deref(), Some("haploid" | "pseudohaploid")) {
                return Err(anyhow!(
                    "VCF genotype truth pseudohaploid case `{}` must declare haploid-compatible ploidy",
                    case.case_id
                ));
            }
        }
    }
    Ok(())
}

fn load_vcf_genotype_truth_bundle(expected_path: &Path) -> Result<VcfGenotypeTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_bundle_contract(
    manifest: &VcfGenotypeTruthManifest,
    bundle: &VcfGenotypeTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != VCF_GENOTYPE_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "VCF genotype truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            VCF_GENOTYPE_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "VCF genotype truth bundle `{}` fixture_id `{}` must equal `{}`",
            expected_path.display(),
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let expected_case_ids =
        manifest.cases.iter().map(|case| case.case_id.as_str()).collect::<BTreeSet<_>>();
    let bundle_case_ids =
        bundle.cases.iter().map(|case| case.case_id.as_str()).collect::<BTreeSet<_>>();
    if expected_case_ids != bundle_case_ids {
        return Err(anyhow!(
            "VCF genotype truth bundle case ids do not match manifest `{}`",
            expected_path.display()
        ));
    }
    Ok(())
}

fn build_actual_truth_bundle(
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

fn collect_stage_ids(cases: &[VcfGenotypeTruthCaseTruth]) -> Vec<String> {
    let mut values = cases.iter().map(|case| case.summary.stage_id.clone()).collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn collect_tool_ids(cases: &[VcfGenotypeTruthCaseTruth]) -> Vec<String> {
    let mut values = cases.iter().map(|case| case.summary.tool_id.clone()).collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn resolve_repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn resolve_fixture_path(fixture_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        fixture_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
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
