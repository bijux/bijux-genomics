use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BamParserFixtureCoverageStatus {
    Covered,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BamParserFixtureCoverageRow {
    pub tool_id: String,
    pub stage_id: String,
    pub parser_fixture_reference_kind: String,
    pub parser_fixture_reference: String,
    pub coverage_status: BamParserFixtureCoverageStatus,
    pub parser_status: String,
    pub support_status: String,
    pub adapter_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamParserFixtureCoverageReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub stage_count: usize,
    pub tool_count: usize,
    pub row_count: usize,
    pub covered_row_count: usize,
    pub missing_row_count: usize,
    pub parser_fixture_coverage_percent: f64,
    pub excluded_non_benchmark_ready_row_count: usize,
    pub excluded_readiness_gap_counts: BTreeMap<String, usize>,
    pub rows: Vec<BamParserFixtureCoverageRow>,
}

impl BamParserFixtureCoverageStatus {
    /// Stable label used by the persisted evidence schema.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            BamParserFixtureCoverageStatus::Covered => "covered",
            BamParserFixtureCoverageStatus::Missing => "missing",
        }
    }
}
