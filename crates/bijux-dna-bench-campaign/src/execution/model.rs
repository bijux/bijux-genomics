use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_bench_corpus::normalized::NormalizedFastqSample as CorpusNormalizedSample;

#[derive(Debug, Clone)]
pub(crate) struct PendingSampleRun {
    pub(crate) sample: CorpusNormalizedSample,
    pub(crate) report_json: PathBuf,
    pub(crate) command_args: Vec<String>,
    pub(crate) command: Vec<String>,
    pub(crate) env_bindings: BTreeMap<String, String>,
    pub(crate) extra_fields: BTreeMap<String, serde_json::Value>,
    pub(crate) post_success_action: Option<PostSuccessAction>,
}

#[derive(Debug, Clone)]
pub(crate) enum PostSuccessAction {
    PromoteAndPruneSortmernaCache { out_root: PathBuf, sample_id: String, rrna_bundle_id: String },
}
