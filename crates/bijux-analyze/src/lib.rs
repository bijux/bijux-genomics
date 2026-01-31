pub mod aggregate;
pub mod contract;
pub mod decision;
pub mod facts;
pub mod failure;
pub mod load;
pub mod model;
mod pipeline;
pub mod report;
pub mod semantic;
mod semantics;

pub use aggregate::*;
pub use bijux_core::metrics::MetricSet;
pub use contract::{analyze_contract_v1, AnalyzeContractV1};
pub use decision::compare::compare_runs;
pub use failure::*;
pub use load::*;
pub use report::*;
pub use semantic::*;

pub mod compare {
    pub use crate::decision::compare::*;
}

pub mod ranking {
    pub use crate::decision::score::*;
}

use std::path::PathBuf;

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
    Rank { stage_id: String, metric_id: String },
    Report,
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub json: bool,
    pub html: bool,
}

#[derive(Debug, Clone)]
pub struct AnalyzeOutput {
    pub run_id: Option<String>,
    pub report_json: Option<PathBuf>,
    pub report_html: Option<PathBuf>,
    pub summary_json: Option<PathBuf>,
    pub compare_json: Option<PathBuf>,
    pub ranking_json: Option<PathBuf>,
    pub decision_trace_json: Option<PathBuf>,
}

/// Analyze a run through the canonical pipeline.
///
/// This is a placeholder for the refactor pipeline (load → validate → normalize → aggregate →
/// compare → rank → explain → render).
///
/// # Errors
/// Returns an error until the pipeline is wired.
pub fn analyze_run(input: &AnalyzeInput) -> anyhow::Result<AnalyzeOutput> {
    pipeline::analyze_run_pipeline(input)
}

pub use crate::decision::score::{
    build_rankings, decision_trace_for_input, print_rank_explain, RankInput, RankingEntry,
    RankingMode, ScoreBreakdown,
};
