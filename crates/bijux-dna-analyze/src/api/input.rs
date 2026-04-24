use std::path::PathBuf;

use bijux_dna_core::ids::StageId;

use super::AnalyzeMetricId;

#[derive(Debug, Clone)]
pub struct AnalyzeInput {
    pub run_id: Option<String>,
    pub sources: AnalyzeSources,
    pub options: AnalyzeOptions,
}

#[derive(Debug, Clone)]
pub enum AnalyzeSources {
    FactsJsonl(PathBuf),
    FactsParquet(PathBuf),
    RunIndexSqlite(PathBuf),
    RunSummaryJson(PathBuf),
}

#[derive(Debug, Clone)]
pub struct AnalyzeOptions {
    pub mode: AnalyzeMode,
    pub strict: bool,
    pub render: RenderOptions,
}

#[derive(Debug, Clone)]
pub enum AnalyzeMode {
    Summary,
    Compare { run_a: String, run_b: String },
    Rank { stage_id: StageId, metric_id: AnalyzeMetricId },
    Report,
}

use super::RenderOptions;
