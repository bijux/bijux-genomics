//! Fully resolved campaign inputs; loading configuration and selecting an executable belong to composition.
use bijux_dna_bench_corpus::source::FastqCorpusSourceSpec;
use std::path::PathBuf;

/// Publication metadata admitted by the canonical publication configuration owner.
#[derive(Debug, Clone)]
pub struct CorpusCampaignContract {
    pub stage_id: String,
    pub scenario_id: String,
    pub sample_scope: String,
    pub tools: Vec<String>,
}

/// Optional configured defaults in canonical stage/manifest flag order.
/// The campaign admits flag/value pairs and applies these only for absent caller keys.
#[derive(Debug, Clone, Default)]
pub struct StageArgumentDefaults {
    pub stage: Vec<(String, String)>,
    pub manifest: Vec<(String, String)>,
}

/// All data needed to plan and execute one corpus campaign, independent of CLI parsing.
#[derive(Debug, Clone)]
pub struct CorpusCampaignRequest {
    pub corpus_id: String,
    pub source: FastqCorpusSourceSpec,
    pub contract: CorpusCampaignContract,
    pub repo_root: PathBuf,
    pub corpus_root: PathBuf,
    pub out_root: PathBuf,
    pub config_path: PathBuf,
    pub publication_config_path: PathBuf,
    pub program: PathBuf,
    pub platform: String,
    pub tools: Vec<String>,
    pub threads: u32,
    pub jobs: u32,
    pub sample_jobs: usize,
    pub sample_limit: usize,
    pub dry_run: bool,
    pub resume: bool,
    pub stage_args: Vec<String>,
    pub manifest_args: Vec<String>,
    pub defaults: StageArgumentDefaults,
}
