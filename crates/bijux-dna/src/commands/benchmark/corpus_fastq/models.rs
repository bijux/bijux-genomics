use std::collections::BTreeMap;
use std::path::PathBuf;

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

#[derive(Debug, Clone)]
pub(super) enum PostSuccessAction {
    PromoteAndPruneSortmernaCache { out_root: PathBuf, sample_id: String, rrna_bundle_id: String },
}
