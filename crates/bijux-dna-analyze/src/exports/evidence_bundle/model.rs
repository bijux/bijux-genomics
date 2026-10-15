//! Owner: bijux-dna-analyze
//! Versioned evidence, profile, comparison, and reviewer journal records.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTimelineCategoryV1 {
    Planner,
    Scheduler,
    Execution,
    Artifact,
    Cache,
    Replay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSeverityV1 {
    Advisory,
    Blocking,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceBundleProfileV1 {
    Draft,
    Operational,
    Certification,
    Publication,
    PublicationStrict,
    CollaboratorRedacted,
    ArchiveRetention,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceCitationTypeV1 {
    Tool,
    Reference,
    Defaults,
    Method,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCitationV1 {
    pub citation_id: String,
    pub citation: String,
    pub citation_type: EvidenceCitationTypeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceMethodsToolV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_digest: Option<String>,
    pub params_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceMethodsSummaryV1 {
    pub schema_version: String,
    pub run_id: String,
    pub correlation_id: String,
    pub stage_count: usize,
    #[serde(default)]
    pub tools: Vec<EvidenceMethodsToolV1>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
    pub citation_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundleFileDigestV1 {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceArchiveMigrationV1 {
    pub manifest_schema_version: String,
    pub evidence_schema_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_inventory_schema_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash_ledger_schema_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceProfileBundleV1 {
    pub schema_version: String,
    pub profile: EvidenceBundleProfileV1,
    pub generated_at: String,
    pub run_id: String,
    pub correlation_id: String,
    pub evidence_bundle: EvidenceBundleV1,
    pub evidence_verification: EvidenceVerificationV1,
    pub profile_validation: EvidenceProfileValidationV1,
    #[serde(default)]
    pub required_files: Vec<EvidenceBundleFileDigestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_migration: Option<EvidenceArchiveMigrationV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceProfileBundleVerificationV1 {
    pub schema_version: String,
    pub verified: bool,
    #[serde(default)]
    pub missing_paths: Vec<String>,
    #[serde(default)]
    pub hash_mismatches: Vec<String>,
    pub evidence_verified: bool,
    pub profile_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerChallengeRequestV1 {
    pub artifact_id: String,
    pub evidence_path: String,
    pub report_field: String,
    pub caveat: String,
    pub question: String,
    pub requested_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerChallengeRecordV1 {
    pub schema_version: String,
    pub challenge_id: String,
    pub created_at: String,
    pub artifact_id: String,
    pub evidence_path: String,
    pub report_field: String,
    pub caveat: String,
    pub question: String,
    pub requested_by: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceTimelineEventV1 {
    pub category: EvidenceTimelineCategoryV1,
    pub event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
    pub correlation_id: String,
    pub status: String,
    #[serde(default)]
    pub attrs: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceArtifactV1 {
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGapV1 {
    pub code: String,
    pub severity: EvidenceSeverityV1,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub blocks_audit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCheckV1 {
    pub check_id: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceHealthV1 {
    pub status: String,
    pub auditable: bool,
    pub checks: Vec<EvidenceCheckV1>,
    pub gaps: Vec<EvidenceGapV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceMetricsV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_time_ms: Option<u64>,
    pub run_time_s: f64,
    pub retry_count: u64,
    pub cache_hit_count: u64,
    pub cache_miss_count: u64,
    pub total_timeline_events: u64,
    pub scientific_failure_classes: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCompactSummaryV1 {
    pub stage_count: usize,
    pub artifact_count: usize,
    pub failed_stage_count: usize,
    pub advisory_gap_count: usize,
    pub final_outputs: Vec<String>,
    pub stage_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceNodeV1 {
    pub node_id: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEdgeV1 {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceProvenanceGraphV1 {
    pub nodes: Vec<EvidenceNodeV1>,
    pub edges: Vec<EvidenceEdgeV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSourcesV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_manifest_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_summary_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facts_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_policy_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_state_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executor_descriptor_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_inventory_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_manifest_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash_ledger_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_verification_path: Option<String>,
    #[serde(default)]
    pub telemetry_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundleV1 {
    pub schema_version: String,
    pub run_id: String,
    pub correlation_id: String,
    pub sources: EvidenceSourcesV1,
    pub compact_summary: EvidenceCompactSummaryV1,
    pub health: EvidenceHealthV1,
    pub metrics: EvidenceMetricsV1,
    pub timeline: Vec<EvidenceTimelineEventV1>,
    pub artifacts: Vec<EvidenceArtifactV1>,
    pub provenance_graph: EvidenceProvenanceGraphV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<EvidenceCitationV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub methods_summary: Option<EvidenceMethodsSummaryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceVerificationV1 {
    pub schema_version: String,
    pub verified: bool,
    pub checks: Vec<EvidenceCheckV1>,
    pub missing_paths: Vec<String>,
    pub gap_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceComparisonV1 {
    pub schema_version: String,
    pub left_run_id: String,
    pub right_run_id: String,
    pub left_correlation_id: String,
    pub right_correlation_id: String,
    pub changed_stage_ids: Vec<String>,
    pub changed_artifacts: Vec<String>,
    pub runtime_delta_s: f64,
    pub evidence_gap_delta: i64,
    pub policy_change_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceProfileCheckV1 {
    pub check_id: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceProfileValidationV1 {
    pub schema_version: String,
    pub profile: EvidenceBundleProfileV1,
    pub ok: bool,
    pub required_paths_present: bool,
    pub tolerated_gap_codes: Vec<String>,
    pub blocking_gap_codes: Vec<String>,
    pub checks: Vec<EvidenceProfileCheckV1>,
}

impl EvidenceHealthV1 {
    pub(super) fn health_gap_count(&self) -> usize {
        self.gaps.len()
    }
}
