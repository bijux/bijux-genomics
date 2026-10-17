//! Wire records for declared genotype expectations and their validation report.
use std::path::PathBuf;

use bijux_dna_domain_vcf::{
    VcfCallingBoundaryV1, VcfGenotypeTruthSummaryV1, VcfLikelihoodWorkflowBoundaryV1,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfGenotypeTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) cases: Vec<VcfGenotypeTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfGenotypeTruthCase {
    pub(super) case_id: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) input_vcf_path: PathBuf,
    #[serde(default)]
    pub(super) command_path: Option<PathBuf>,
    pub(super) evaluation_kind: VcfGenotypeEvaluationKind,
    #[serde(default)]
    pub(super) declared_ploidy: Option<String>,
    #[serde(default, flatten)]
    pub(super) coverage: VcfGenotypeTruthCaseCoverageSignals,
    #[serde(default, flatten)]
    pub(super) likelihood: VcfGenotypeTruthCaseLikelihoodSignals,
    #[serde(default)]
    pub(super) mean_coverage: Option<f64>,
    #[serde(default)]
    pub(super) minimum_mean_coverage: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfGenotypeTruthCaseCoverageSignals {
    #[serde(default)]
    pub(super) has_input_bam: bool,
    #[serde(default)]
    pub(super) has_reference_context: bool,
    #[serde(default)]
    pub(super) low_coverage_expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfGenotypeTruthCaseLikelihoodSignals {
    #[serde(default)]
    pub(super) downstream_gl_compatible: bool,
    #[serde(default)]
    pub(super) uncertainty_reported: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum VcfGenotypeEvaluationKind {
    None,
    Diploid,
    Pseudohaploid,
    Likelihood,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfGenotypeTruthValidationReport {
    pub schema_version: &'static str,
    pub fixture_id: String,
    pub manifest_path: String,
    pub expected_path: String,
    pub validated_case_count: usize,
    pub validated_stage_ids: Vec<String>,
    pub validated_tool_ids: Vec<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfGenotypeTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) cases: Vec<VcfGenotypeTruthCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfGenotypeTruthCaseTruth {
    pub(super) case_id: String,
    pub(super) summary: VcfGenotypeTruthSummaryV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) command: Option<VcfGenotypeTruthCommandSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) diploid_boundary: Option<VcfCallingBoundaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) pseudohaploid_boundary: Option<VcfCallingBoundaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) likelihood_boundary: Option<VcfLikelihoodWorkflowBoundaryV1>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfGenotypeTruthCommandSummary {
    pub(super) command_path: String,
    #[serde(default)]
    pub(super) stage_id: Option<String>,
    #[serde(default)]
    pub(super) tool_id: Option<String>,
    #[serde(default)]
    pub(super) sampling_policy: Option<String>,
    #[serde(default)]
    pub(super) likelihood_model: Option<String>,
    #[serde(default)]
    pub(super) random_seed: Option<u64>,
    #[serde(default)]
    pub(super) seed_argument: Option<u64>,
    #[serde(default)]
    pub(super) seed_argument_matches_random_seed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct RawVcfGenotypeTruthCommand {
    #[serde(default)]
    pub(super) stage_id: Option<String>,
    #[serde(default)]
    pub(super) tool_id: Option<String>,
    #[serde(default)]
    pub(super) sampling_policy: Option<String>,
    #[serde(default)]
    pub(super) likelihood_model: Option<String>,
    #[serde(default)]
    pub(super) random_seed: Option<u64>,
    #[serde(default)]
    pub(super) argv: Vec<String>,
}
