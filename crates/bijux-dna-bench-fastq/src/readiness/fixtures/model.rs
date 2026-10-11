use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FastqParserFixtureCoverageStatus {
    Covered,
    MissingFixtureBinding,
    MissingFixtureCase,
    InvalidFixtureCase,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FastqParserFixtureCoverageRow {
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
    pub parser_fixture_parser_id: String,
    pub parser_fixture_schema_id: String,
    pub parser_fixture_reference_kind: String,
    pub parser_fixture_reference: String,
    pub parser_fixture_surface: String,
    pub parser_fixture_canonical_tool_id: String,
    pub coverage_status: FastqParserFixtureCoverageStatus,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FastqParserFixtureCoverageReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub stage_count: usize,
    pub tool_count: usize,
    pub row_count: usize,
    pub covered_row_count: usize,
    pub missing_row_count: usize,
    pub parser_fixture_coverage_percent: f64,
    pub coverage_status_counts: BTreeMap<String, usize>,
    pub rows: Vec<FastqParserFixtureCoverageRow>,
}
