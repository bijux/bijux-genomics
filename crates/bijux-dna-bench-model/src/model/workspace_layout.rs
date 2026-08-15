//! Owner: bijux-dna-bench-model
//! Persisted benchmark workspace layout, convergence, and normalization reports.

use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct WorkspaceEntrySummary {
    pub exists: bool,
    pub kind: String,
    pub file_count: u64,
    pub total_size_bytes: u64,
    pub mtime: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_sha256: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct WorkspaceConvergenceAction {
    pub action: String,
    pub entry_name: String,
    pub source: String,
    pub target: String,
    pub legacy_summary: WorkspaceEntrySummary,
    pub canonical_summary: WorkspaceEntrySummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct WorkspaceConvergencePlan {
    pub canonical_root: String,
    pub legacy_root: String,
    pub actions: Vec<WorkspaceConvergenceAction>,
    pub removable_legacy_root: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_root_removed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct WorkspaceStageNormalizationReport {
    pub stage_id: String,
    pub action: String,
    pub source: String,
    pub target: String,
    pub legacy_summary: WorkspaceEntrySummary,
    pub canonical_summary: WorkspaceEntrySummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct WorkspaceNormalizationReport {
    pub corpus_id: String,
    pub corpus_dir_name: String,
    pub canonical_corpus_root: String,
    pub legacy_corpus_root: String,
    pub archive_stage_ids: Vec<String>,
    pub cache_stage_ids: Vec<String>,
    pub shared_stage_ids: Vec<String>,
    pub archive_only_stage_ids: Vec<String>,
    pub cache_only_stage_ids: Vec<String>,
    pub mode: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub convergence_plan: Option<WorkspaceConvergencePlan>,
    pub stage_reports: Vec<WorkspaceStageNormalizationReport>,
    pub moved_stage_ids: Vec<String>,
    #[serde(alias = "removed_duplicate_stage_ids")]
    pub quarantined_duplicate_stage_ids: Vec<String>,
    pub manual_review_stage_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct WorkspaceAuthoritativeRoots {
    pub remote_results_root: String,
    pub remote_reference_root: String,
    pub local_stage_root: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct WorkspaceRootPairSummary {
    pub scope: String,
    pub canonical_root: String,
    pub legacy_root: String,
    pub canonical_exists: bool,
    pub legacy_exists: bool,
    pub canonical_entries: Vec<String>,
    pub legacy_entries: Vec<String>,
    pub shared_entries: Vec<String>,
    pub canonical_only_entries: Vec<String>,
    pub legacy_only_entries: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct WorkspaceLocalStageLayout {
    pub archive_corpus_root: String,
    pub cache_corpus_root: String,
    pub archive_stage_ids: Vec<String>,
    pub cache_stage_ids: Vec<String>,
    pub shared_stage_ids: Vec<String>,
    pub archive_only_stage_ids: Vec<String>,
    pub cache_only_stage_ids: Vec<String>,
    pub authoritative_stage_root: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct WorkspaceLayoutIssue {
    pub issue_id: String,
    pub detail: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct WorkspaceLayoutReport {
    pub local_results_root: String,
    pub local_cache_mirror_root: String,
    pub remote_workspace_root: String,
    pub authoritative_roots: WorkspaceAuthoritativeRoots,
    pub root_pairs: Vec<WorkspaceRootPairSummary>,
    pub unexpected_remote_siblings: Vec<String>,
    pub local_stage_layout: WorkspaceLocalStageLayout,
    pub status: String,
    pub issue_count: usize,
    pub issues: Vec<WorkspaceLayoutIssue>,
}

#[must_use]
pub fn workspace_convergence_plan_schema() -> Schema {
    schemars::schema_for!(WorkspaceConvergencePlan)
}

#[must_use]
pub fn workspace_normalization_report_schema() -> Schema {
    schemars::schema_for!(WorkspaceNormalizationReport)
}

#[must_use]
pub fn workspace_layout_report_schema() -> Schema {
    schemars::schema_for!(WorkspaceLayoutReport)
}
