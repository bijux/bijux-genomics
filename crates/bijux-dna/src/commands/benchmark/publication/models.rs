use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct CorpusSummary {
    pub(super) schema_version: String,
    pub(super) corpus_id: String,
    pub(super) stage_id: String,
    pub(super) scenario_id: String,
    pub(super) generated_at_utc: String,
    pub(super) platform: String,
    pub(super) corpus_root: String,
    pub(super) run_root: String,
    pub(super) sample_scope: String,
    pub(super) samples_total: usize,
    pub(super) samples_failed: usize,
    pub(super) tools: Vec<String>,
    pub(super) cohort_counts: BTreeMap<String, usize>,
    pub(super) era_counts: BTreeMap<String, usize>,
    pub(super) layout_counts: BTreeMap<String, usize>,
    pub(super) tool_summary: Vec<CorpusToolSummary>,
    pub(super) headline: CorpusHeadline,
}

#[derive(Debug, Serialize)]
pub(super) struct CorpusToolSummary {
    pub(super) tool: String,
    pub(super) records: usize,
    pub(super) pass_rate: Option<f64>,
    pub(super) median_runtime_s: Option<f64>,
    pub(super) mean_runtime_s: Option<f64>,
    pub(super) max_runtime_s: Option<f64>,
}

#[derive(Debug, Serialize)]
pub(super) struct CorpusHeadline {
    pub(super) fastest_tool: Option<String>,
    pub(super) fastest_runtime_s: Option<f64>,
    pub(super) highest_pass_rate_tool: Option<String>,
    pub(super) highest_pass_rate: Option<f64>,
}

#[derive(Debug, Clone)]
pub(super) struct CorpusSampleResultRow {
    pub(super) sample_id: String,
    pub(super) accession: String,
    pub(super) era: String,
    pub(super) layout: String,
    pub(super) study_accession: String,
    pub(super) size_band: String,
    pub(super) tool: String,
    pub(super) runtime_s: Option<f64>,
    pub(super) exit_code: Option<i64>,
    pub(super) report_json: String,
}

#[derive(Debug)]
pub(super) struct CorpusArtifactSet {
    pub(super) summary: CorpusSummary,
    pub(super) sample_rows: Vec<CorpusSampleResultRow>,
    pub(super) tool_runtime_rows: Vec<BTreeMap<String, String>>,
    pub(super) cohort_runtime_rows: Vec<BTreeMap<String, String>>,
    pub(super) outlier_rows: Vec<BTreeMap<String, String>>,
    pub(super) benchmark_markdown: String,
}

#[derive(Debug, Serialize)]
pub(super) struct DossierIndex {
    pub(super) corpus_id: String,
    pub(super) stage_count: usize,
    pub(super) published_stage_count: usize,
    pub(super) missing_stage_count: usize,
    pub(super) stages: Vec<DossierStageEntry>,
}

#[derive(Debug, Serialize)]
pub(super) struct DossierStageEntry {
    pub(super) stage_id: String,
    pub(super) sample_scope: String,
    pub(super) status: String,
    pub(super) summary_path: String,
    pub(super) dossier_path: String,
    pub(super) expected_remote_run_root: String,
    pub(super) expected_local_cache_mirror_run_root: String,
    pub(super) expected_local_results_run_root: String,
    pub(super) generated_at_utc: Option<String>,
    pub(super) platform: Option<String>,
    pub(super) corpus_root: Option<String>,
    pub(super) run_root: Option<String>,
    pub(super) run_root_source: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct PublishedResultsStatusReport {
    pub(super) corpus_id: String,
    pub(super) applicable_stage_count: usize,
    pub(super) published_stage_count: usize,
    pub(super) complete_stage_count: usize,
    pub(super) incomplete_stage_count: usize,
    pub(super) issue_count: usize,
    pub(super) stages: Vec<PublishedResultsStageReport>,
}

#[derive(Debug, Serialize)]
pub(super) struct PublishedResultsStageReport {
    pub(super) stage_id: String,
    pub(super) status: String,
    pub(super) issue_count: usize,
    pub(super) reported_run_root: String,
    pub(super) selected_run_root: String,
    pub(super) newest_available_run_root: String,
    pub(super) selected_run_root_is_newest: bool,
    pub(super) available_run_roots: Vec<String>,
    pub(super) issues: Vec<StageResultIssue>,
}

#[derive(Debug, Serialize)]
pub(super) struct StageResultIssue {
    pub(super) stage_id: String,
    pub(super) issue_id: String,
    pub(super) detail: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct PublicationCorpusSpec {
    pub(super) corpus_id: String,
    #[serde(default)]
    pub(super) target_ancient_se: usize,
    #[serde(default)]
    pub(super) target_ancient_pe: usize,
    #[serde(default)]
    pub(super) target_modern_se: usize,
    #[serde(default)]
    pub(super) target_modern_pe: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct BenchmarkPublicationStatusReport {
    pub(super) corpus_id: String,
    pub(super) docs_root: String,
    pub(super) benchmarkable_stage_count: usize,
    pub(super) applicable_stage_count: usize,
    pub(super) completed_stage_count: usize,
    pub(super) incomplete_stage_count: usize,
    pub(super) excluded_stage_count: usize,
    pub(super) issue_count: usize,
    pub(super) audit_warning_count: usize,
    pub(super) audit_warnings: Vec<String>,
    pub(super) supplemental_findings_generated_at_utc: Option<String>,
    pub(super) excluded_stages: Vec<ExcludedStageEntry>,
    pub(super) stages: Vec<PublicationStageReport>,
}

#[derive(Debug, Serialize)]
pub(super) struct ExcludedStageEntry {
    pub(super) stage_id: String,
    pub(super) reason: String,
}

#[derive(Debug, Serialize)]
pub(super) struct PublicationStageReport {
    pub(super) stage_id: String,
    pub(super) scenario_id: String,
    pub(super) sample_scope: String,
    pub(super) contract_tool_roster: Vec<String>,
    pub(super) expected_tool_roster: Vec<String>,
    pub(super) method_path: String,
    pub(super) corpus_path: String,
    pub(super) status: String,
    pub(super) issue_count: usize,
    pub(super) results_status: String,
    pub(super) results_issue_count: usize,
    pub(super) results_selected_run_root: String,
    pub(super) results_newest_available_run_root: String,
    pub(super) results_selected_run_root_is_newest: bool,
    pub(super) issues: Vec<StageAuditIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct StageAuditIssue {
    pub(super) stage_id: String,
    pub(super) issue_id: String,
    pub(super) severity: String,
    pub(super) detail: String,
}

#[derive(Debug, Serialize)]
pub(super) struct RemediationQueue {
    pub(super) corpus_id: String,
    pub(super) stage_count: usize,
    pub(super) open_stage_count: usize,
    pub(super) clear_stage_count: usize,
    pub(super) stages: Vec<RemediationStageEntry>,
}

#[derive(Debug, Serialize)]
pub(super) struct RemediationStageEntry {
    pub(super) stage_id: String,
    pub(super) owner: String,
    pub(super) status: String,
    pub(super) issue_count: usize,
    pub(super) issue_group_count: usize,
    pub(super) recommended_action: String,
    pub(super) publication_status: String,
    pub(super) results_status: String,
    pub(super) sample_scope: String,
    pub(super) published_generated_at_utc: Option<String>,
    pub(super) run_root_source: Option<String>,
    pub(super) issue_groups: Vec<RemediationIssueGroup>,
    pub(super) issues: Vec<RemediationIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct RemediationIssue {
    pub(super) issue_id: String,
    pub(super) detail: String,
    pub(super) severity: String,
    pub(super) source: String,
}

#[derive(Debug, Serialize)]
pub(super) struct RemediationIssueGroup {
    pub(super) issue_id: String,
    pub(super) count: usize,
    pub(super) sources: Vec<String>,
    pub(super) severity: String,
    pub(super) example_details: Vec<String>,
    pub(super) additional_detail_count: usize,
}

#[derive(Debug, Clone)]
pub(super) struct StageRunRootCandidate {
    pub(super) path: PathBuf,
}

#[derive(Debug)]
pub(super) struct StageRunRootSelection {
    pub(super) selected_path: PathBuf,
    pub(super) newest_available_path: Option<PathBuf>,
}
