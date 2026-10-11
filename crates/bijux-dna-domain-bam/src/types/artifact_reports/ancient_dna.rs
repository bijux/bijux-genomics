//! Persisted BAM ancient dna reports and schema identities.

use super::workflow::BamAdvisoryBoundaryV1;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const BAM_ADNA_DAMAGE_TRUTH_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.adna_damage_truth.v1";

pub const BAM_ADNA_CONTAMINATION_TRUTH_SUMMARY_SCHEMA_VERSION: &str =
    "bijux.bam.adna_contamination_truth.v1";

pub const BAM_ADNA_WORKFLOW_SCHEMA_VERSION: &str = "bijux.bam.adna_workflow.v1";

pub const BAM_CONTAMINATION_WORKFLOW_SCHEMA_VERSION: &str = "bijux.bam.contamination_workflow.v1";

pub const BAM_DAMAGE_EVIDENCE_SCHEMA_VERSION: &str = "bijux.bam.damage_evidence.v1";

pub const BAM_AUTHENTICITY_ADVISORY_SCHEMA_VERSION: &str = "bijux.bam.authenticity_advisory.v1";

pub const BAM_CONTAMINATION_EVIDENCE_SCHEMA_VERSION: &str = "bijux.bam.contamination_evidence.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamAdnaDamageTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub udg_model: String,
    pub terminal_c_to_t_5p: f64,
    pub terminal_g_to_a_3p: f64,
    pub short_fragment_fraction: f64,
    pub damage_signal: String,
    pub strict_profile_upgraded: bool,
    pub terminal_5p_class: String,
    pub terminal_3p_class: String,
    pub terminal_pattern_class: String,
    #[serde(default)]
    pub insufficiency_reason: Option<String>,
    pub insufficiency_policy: String,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamAdnaContaminationTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool: String,
    pub scope: String,
    pub prerequisites_passed: bool,
    pub status: String,
    pub confidence_interval_status: String,
    #[serde(default)]
    pub estimate: Option<f64>,
    #[serde(default)]
    pub ci_low: Option<f64>,
    #[serde(default)]
    pub ci_high: Option<f64>,
    #[serde(default)]
    pub insufficiency_reason: Option<String>,
    pub insufficiency_policy: String,
    pub advisory_only: bool,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAdnaWorkflowV1 {
    pub schema_version: String,
    pub workflow_id: String,
    pub damage_tools: Vec<String>,
    pub authenticity_tools: Vec<String>,
    pub evidence_only: bool,
    pub authenticity_caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamDamageEvidenceV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub terminal_c_to_t_5p: f64,
    pub terminal_g_to_a_3p: f64,
    pub short_fragment_fraction: f64,
    pub damage_signal: String,
    pub strict_profile_upgraded: bool,
    pub advisory_boundary: BamAdvisoryBoundaryV1,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamAuthenticityAdvisoryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub score: f64,
    pub confidence: f64,
    pub status: String,
    pub pmd_like_signal_present: bool,
    pub advisory_boundary: BamAdvisoryBoundaryV1,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamContaminationEvidenceV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub scope: String,
    pub tool: String,
    pub prerequisites_passed: bool,
    #[serde(default)]
    pub estimate: Option<f64>,
    #[serde(default)]
    pub ci_low: Option<f64>,
    #[serde(default)]
    pub ci_high: Option<f64>,
    pub advisory_boundary: BamAdvisoryBoundaryV1,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamContaminationToolContractV1 {
    pub tool_id: String,
    pub supported_scope: String,
    pub required_inputs: Vec<String>,
    pub emits_confidence: bool,
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamContaminationWorkflowV1 {
    pub schema_version: String,
    pub workflow_id: String,
    pub tools: Vec<BamContaminationToolContractV1>,
}
