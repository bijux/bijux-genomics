use serde::Serialize;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FastqParserCoverageKind {
    Covered,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FastqParserCoverageRow {
    pub tool_id: String,
    pub stage_id: String,
    pub parser_coverage: FastqParserCoverageKind,
    pub parser_status: String,
    pub support_status: String,
    pub adapter_status: String,
    pub corpus_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FastqParserCoverageReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub stage_count: usize,
    pub tool_count: usize,
    pub row_count: usize,
    pub parser_covered_row_count: usize,
    pub parser_missing_row_count: usize,
    pub parser_coverage_percent: f64,
    pub rows: Vec<FastqParserCoverageRow>,
}

impl FastqParserCoverageKind {
    /// Stable persisted label for this readiness state.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FastqParserCoverageKind::Covered => "covered",
            FastqParserCoverageKind::Missing => "missing",
        }
    }
}
