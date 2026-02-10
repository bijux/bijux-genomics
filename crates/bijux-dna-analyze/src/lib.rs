#![allow(ambiguous_glob_reexports)]
//! Analyze pipeline for Bijux runs.
//!
//! Contract: `analyze_run` is the single public entrypoint. It accepts typed input paths and
//! options, enforces the load → validate → compute → report → render pipeline, and returns
//! versioned artifacts. Outputs are stable, deterministic, and only grow in a backward-compatible
//! way for published schemas.

pub mod aggregate;
pub mod contract;
pub mod decision;
pub mod export;
pub mod failure;
pub mod load;
pub mod model;
mod pipeline;
pub mod report;
mod semantics;

pub use aggregate::*;
pub use bijux_dna_core::metrics::MetricSet;
pub use contract::{analyze_contract_v1, AnalyzeContractV1};
pub use decision::compare::compare_runs;
pub use failure::*;
pub use load::*;
pub use report::*;
pub use semantics::metrics::{metric_semantics, MetricDirection};

pub mod compare {
    pub use crate::decision::compare::*;
}

pub mod ranking {
    pub use crate::decision::score::*;
}

use std::path::PathBuf;

use bijux_dna_core::ids::StageId;
use bijux_dna_core::metrics::{parse_derived_metric_id, parse_metric_id};

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
    Compare {
        run_a: String,
        run_b: String,
    },
    Rank {
        stage_id: StageId,
        metric_id: AnalyzeMetricId,
    },
    Report,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyzeMetricId {
    Metric(bijux_dna_core::metrics::MetricId),
    Derived(bijux_dna_core::metrics::DerivedMetricId),
}

impl AnalyzeMetricId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Metric(id) => id.as_str(),
            Self::Derived(id) => id.as_str(),
        }
    }
}

impl std::str::FromStr for AnalyzeMetricId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Some(id) = parse_metric_id(s) {
            return Ok(Self::Metric(id));
        }
        if let Some(id) = parse_derived_metric_id(s) {
            return Ok(Self::Derived(id));
        }
        Err(anyhow::anyhow!(
            "unknown metric id `{s}`; register in core metric registry first"
        ))
    }
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub json: bool,
    pub html: bool,
    pub output_dir: Option<PathBuf>,
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
/// Delegates to the pipeline implementation (load → validate → normalize → aggregate → compare
/// → rank → explain → render).
///
/// # Errors
/// Returns an error if any pipeline stage fails.
pub fn analyze_run(input: &AnalyzeInput) -> anyhow::Result<AnalyzeOutput> {
    pipeline::analyze_run_pipeline(input)
}

pub use crate::decision::score::{
    build_rankings, decision_trace_for_input, print_rank_explain, RankInput, RankingEntry,
    RankingMode, ScoreBreakdown,
};
