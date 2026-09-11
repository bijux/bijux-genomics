use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BamBenchmarkStatus {
    BenchmarkReady,
    NotBenchmarkReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BamAdapterCoverageKind {
    Covered,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BamReadinessGapKind {
    None,
    Corpus,
    Parser,
    Adapter,
    Support,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BamCommandAdapterCoverageRow {
    pub tool_id: String,
    pub stage_id: String,
    pub benchmark_status: BamBenchmarkStatus,
    pub adapter_coverage: BamAdapterCoverageKind,
    pub readiness_gap: BamReadinessGapKind,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
    pub corpus_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamCommandAdapterCoverageReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub stage_count: usize,
    pub tool_count: usize,
    pub row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub benchmark_ready_adapter_covered_row_count: usize,
    pub benchmark_ready_adapter_missing_row_count: usize,
    pub readiness_gap_counts: BTreeMap<String, usize>,
    pub rows: Vec<BamCommandAdapterCoverageRow>,
}

impl BamBenchmarkStatus {
    /// Stable label used by the persisted evidence schema.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            BamBenchmarkStatus::BenchmarkReady => "benchmark_ready",
            BamBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
        }
    }
}

impl BamAdapterCoverageKind {
    /// Stable label used by the persisted evidence schema.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            BamAdapterCoverageKind::Covered => "covered",
            BamAdapterCoverageKind::Missing => "missing",
        }
    }
}

impl BamReadinessGapKind {
    /// Stable label used by the persisted evidence schema.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            BamReadinessGapKind::None => "none",
            BamReadinessGapKind::Corpus => "corpus",
            BamReadinessGapKind::Parser => "parser",
            BamReadinessGapKind::Adapter => "adapter",
            BamReadinessGapKind::Support => "support",
        }
    }
}
