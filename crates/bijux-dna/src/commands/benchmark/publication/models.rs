use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_api::v1::api::bench::{
    BenchmarkCorpusDossierSampleResultRow, BenchmarkCorpusDossierSummary,
};

#[derive(Debug)]
pub(super) struct CorpusArtifactSet {
    pub(super) summary: BenchmarkCorpusDossierSummary,
    pub(super) sample_rows: Vec<BenchmarkCorpusDossierSampleResultRow>,
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
