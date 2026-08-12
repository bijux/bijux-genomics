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

#[derive(Debug, Clone)]
pub(super) struct StageRunRootCandidate {
    pub(super) path: PathBuf,
}

#[derive(Debug)]
pub(super) struct StageRunRootSelection {
    pub(super) selected_path: PathBuf,
    pub(super) newest_available_path: Option<PathBuf>,
}
