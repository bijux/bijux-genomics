use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FastqBenchmarkStatus {
    BenchmarkReady,
    NotBenchmarkReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FastqAdapterCoverageKind {
    Covered,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FastqReadinessGapKind {
    None,
    Corpus,
    Parser,
    Adapter,
    Support,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FastqCommandAdapterCoverageRow {
    pub tool_id: String,
    pub stage_id: String,
    pub benchmark_status: FastqBenchmarkStatus,
    pub adapter_coverage: FastqAdapterCoverageKind,
    pub readiness_gap: FastqReadinessGapKind,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
    pub corpus_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FastqCommandAdapterCoverageReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub stage_count: usize,
    pub tool_count: usize,
    pub row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub benchmark_ready_adapter_covered_row_count: usize,
    pub benchmark_ready_adapter_missing_row_count: usize,
    pub readiness_gap_counts: BTreeMap<String, usize>,
    pub rows: Vec<FastqCommandAdapterCoverageRow>,
}

impl FastqBenchmarkStatus {
    /// Stable persisted label for this readiness state.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FastqBenchmarkStatus::BenchmarkReady => "benchmark_ready",
            FastqBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
        }
    }
}

impl FastqAdapterCoverageKind {
    /// Stable persisted label for this readiness state.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FastqAdapterCoverageKind::Covered => "covered",
            FastqAdapterCoverageKind::Missing => "missing",
        }
    }
}

impl FastqReadinessGapKind {
    /// Stable persisted label for this readiness state.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FastqReadinessGapKind::None => "none",
            FastqReadinessGapKind::Corpus => "corpus",
            FastqReadinessGapKind::Parser => "parser",
            FastqReadinessGapKind::Adapter => "adapter",
            FastqReadinessGapKind::Support => "support",
        }
    }
}
