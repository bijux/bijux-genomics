use crate::cache::DepleteRrnaStageOptions;
use crate::catalog::CorpusFastqStageCatalogEntry;
use crate::execution::PendingSampleRun;
use crate::ports::{CampaignExecution, CampaignPaths};
use crate::request::CorpusCampaignRequest;
use bijux_dna_bench_corpus::run::BenchmarkCorpusSampleRun;
use std::collections::BTreeMap;
pub(super) struct CampaignScope<'a> {
    pub args: &'a CorpusCampaignRequest,
    pub workspace_config: &'a dyn CampaignPaths,
    pub executor: &'a dyn CampaignExecution,
    pub stage_spec: CorpusFastqStageCatalogEntry,
    pub tools: Vec<String>,
    pub resolved_stage_args: Vec<String>,
    pub deplete_rrna_options: Option<DepleteRrnaStageOptions>,
    pub runtime_env: BTreeMap<String, String>,
}
pub(super) struct PlannedSamples {
    pub runs: Vec<BenchmarkCorpusSampleRun>,
    pub pending: Vec<PendingSampleRun>,
    pub extra_manifest_fields: BTreeMap<String, serde_json::Value>,
}
