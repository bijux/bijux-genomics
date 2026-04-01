use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;

use super::CorpusNormalizedSample;

#[derive(Debug, Clone)]
pub(super) struct PendingSampleRun {
    pub(super) sample: CorpusNormalizedSample,
    pub(super) report_json: PathBuf,
    pub(super) command_args: Vec<String>,
    pub(super) command: Vec<String>,
    pub(super) env_bindings: BTreeMap<String, String>,
    pub(super) extra_fields: BTreeMap<String, serde_json::Value>,
    pub(super) post_success_action: Option<PostSuccessAction>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct SampleRunRecord {
    pub(super) sample_id: String,
    pub(super) r1: String,
    pub(super) r2: Option<String>,
    pub(super) layout: String,
    pub(super) status: String,
    pub(super) exit_code: i32,
    pub(super) command: Vec<String>,
    pub(super) report_json: String,
    #[serde(flatten)]
    pub(super) extra_fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub(super) struct CorpusRunManifest {
    pub(super) schema_version: String,
    pub(super) generated_at_utc: String,
    pub(super) corpus_id: String,
    pub(super) stage_id: String,
    pub(super) scenario_id: String,
    pub(super) sample_scope: String,
    pub(super) tool_kind: String,
    pub(super) platform: String,
    pub(super) tools: Vec<String>,
    pub(super) threads: u32,
    pub(super) jobs: u32,
    pub(super) sample_jobs: usize,
    pub(super) sample_limit: Option<usize>,
    pub(super) dry_run: bool,
    pub(super) config_path: String,
    pub(super) publication_config_path: String,
    pub(super) repo_root: String,
    pub(super) corpus_root: String,
    pub(super) out_root: String,
    pub(super) stage_args: Vec<String>,
    pub(super) samples_total: usize,
    pub(super) samples_failed: usize,
    pub(super) runs: Vec<SampleRunRecord>,
    #[serde(flatten)]
    pub(super) extra_fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub(super) enum PostSuccessAction {
    PromoteAndPruneSortmernaCache {
        out_root: PathBuf,
        sample_id: String,
        rrna_bundle_id: String,
    },
}
