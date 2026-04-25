use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

fn validate_typed_id(prefix: &str, value: &str) -> Result<(), String> {
    if !value.starts_with(prefix) {
        return Err(format!("{value} must start with {prefix}"));
    }
    if value.len() <= prefix.len() {
        return Err(format!("{value} must include a durable suffix"));
    }
    if value.contains("..") {
        return Err(format!("{value} must not contain empty path segments"));
    }
    if !value.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
    }) {
        return Err(format!(
            "{value} must contain only lowercase ascii letters, digits, '.' and '-'"
        ));
    }
    Ok(())
}

macro_rules! typed_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, String> {
                let value = value.into();
                validate_typed_id($prefix, &value)?;
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

typed_id!(SourceId, "source.");
typed_id!(EvidenceId, "evidence.");
typed_id!(ClaimId, "claim.");
typed_id!(AssumptionId, "assumption.");
typed_id!(ReasoningId, "reasoning.");
typed_id!(DecisionId, "decision.");
typed_id!(BindingId, "binding.");
typed_id!(ScienceReleaseId, "release.");

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Draft,
    Candidate,
    Accepted,
    Deprecated,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseStatus {
    Draft,
    Candidate,
    Released,
    Deprecated,
    Superseded,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStrengthTier {
    Observational,
    Governing,
    Benchmark,
    ReleaseCritical,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    RepoFile,
    RepoDirectory,
    Document,
    ExternalDocument,
    ExternalRepository,
    Paper,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceAccess {
    #[default]
    RepoPath,
    ManualDownload,
    ManualClone,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementLevel {
    Advisory,
    Required,
    Blocking,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceSpec {
    pub schema_version: String,
    pub source_id: SourceId,
    pub kind: SourceKind,
    #[serde(default)]
    pub access: SourceAccess,
    pub title: String,
    pub locator: String,
    pub authority: String,
    #[serde(default)]
    pub archive_path: Option<String>,
    #[serde(default)]
    pub citation: Option<String>,
    #[serde(default)]
    pub tool_ids: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceSpec {
    pub schema_version: String,
    pub evidence_id: EvidenceId,
    pub statement: String,
    pub status: ReviewStatus,
    pub strength: EvidenceStrengthTier,
    #[serde(default)]
    pub source_ids: Vec<SourceId>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClaimSpec {
    pub schema_version: String,
    pub claim_id: ClaimId,
    pub statement: String,
    pub scope: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub status: ReviewStatus,
    pub owner: String,
    pub review_due: String,
    #[serde(default)]
    pub supports: Vec<EvidenceId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssumptionSpec {
    pub schema_version: String,
    pub assumption_id: AssumptionId,
    pub statement: String,
    pub status: ReviewStatus,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReasoningSpec {
    pub schema_version: String,
    pub reasoning_id: ReasoningId,
    #[serde(default)]
    pub claim_ids: Vec<ClaimId>,
    #[serde(default)]
    pub evidence_ids: Vec<EvidenceId>,
    #[serde(default)]
    pub assumption_ids: Vec<AssumptionId>,
    pub residual_risk: String,
    pub conclusion: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionSpec {
    pub schema_version: String,
    pub decision_id: DecisionId,
    pub statement: String,
    pub decision_type: String,
    pub status: ReviewStatus,
    #[serde(default)]
    pub derived_from: Vec<ClaimId>,
    pub reasoning: ReasoningId,
    pub owner: String,
    pub effective_from: String,
    pub confidence: ConfidenceLevel,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BindingSpec {
    pub schema_version: String,
    pub binding_id: BindingId,
    pub decision_id: DecisionId,
    pub target_type: String,
    pub target_ref: String,
    pub enforcement_level: EnforcementLevel,
    pub target_domain: String,
    #[serde(default)]
    pub claim_ids: Vec<ClaimId>,
    #[serde(default)]
    pub source_ids: Vec<SourceId>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReleaseManifestSpec {
    pub schema_version: String,
    pub release_id: ScienceReleaseId,
    pub title: String,
    pub status: ReleaseStatus,
    #[serde(default)]
    pub binding_ids: Vec<BindingId>,
    #[serde(default)]
    pub claim_ids: Vec<ClaimId>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct LoadedSpecs {
    pub sources: BTreeMap<String, SourceSpec>,
    pub evidences: BTreeMap<String, EvidenceSpec>,
    pub claims: BTreeMap<String, ClaimSpec>,
    pub assumptions: BTreeMap<String, AssumptionSpec>,
    pub reasonings: BTreeMap<String, ReasoningSpec>,
    pub decisions: BTreeMap<String, DecisionSpec>,
    pub bindings: BTreeMap<String, BindingSpec>,
    pub releases: BTreeMap<String, ReleaseManifestSpec>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ClaimEvidenceRow {
    pub claim_id: String,
    pub evidence_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DecisionReasoningRow {
    pub decision_id: String,
    pub reasoning_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BindingResolutionRow {
    pub binding_id: String,
    pub decision_id: String,
    pub target_type: String,
    pub target_ref: String,
    pub enforcement_level: String,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceInventoryRow {
    pub source_id: String,
    pub kind: String,
    pub access: String,
    pub authority: String,
    pub locator: String,
    pub archive_path: String,
    pub archive_status: String,
    pub citation: String,
    pub tool_ids: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceArchiveGapRow {
    pub source_id: String,
    pub kind: String,
    pub access: String,
    pub locator: String,
    pub archive_path: String,
    pub citation: String,
    pub tool_ids: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FastqEnvironmentRow {
    pub stage_id: String,
    pub tool_id: String,
    pub stage_status: String,
    pub tool_status: String,
    pub is_default: bool,
    pub execution_status: String,
    pub runtime_support: String,
    pub normalization_support: String,
    pub benchmark_support: String,
    pub registry_status: String,
    pub runtimes: String,
    pub container_ref: String,
    pub dockerfile: String,
    pub apptainer_def: String,
    pub evidence_count: usize,
    pub claim_ids: String,
    pub decision_id: String,
    pub binding_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ScienceIndex {
    pub sources: usize,
    pub source_inventory_rows: usize,
    pub source_archive_gap_rows: usize,
    pub evidences: usize,
    pub claims: usize,
    pub assumptions: usize,
    pub reasonings: usize,
    pub decisions: usize,
    pub bindings: usize,
    pub releases: usize,
    pub fastq_environment_rows: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct CompiledScience {
    pub source_inventory: Vec<SourceInventoryRow>,
    pub source_archive_gaps: Vec<SourceArchiveGapRow>,
    pub claim_evidence_map: Vec<ClaimEvidenceRow>,
    pub decision_reasoning_map: Vec<DecisionReasoningRow>,
    pub binding_resolution: Vec<BindingResolutionRow>,
    pub unresolved_refs: Vec<String>,
    pub fastq_environment_rows: Vec<FastqEnvironmentRow>,
    pub index: ScienceIndex,
}
