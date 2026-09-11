use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VcfParserFixtureCoverageStatus {
    Covered,
    MissingFixtureInventory,
    MissingFixtureDirectory,
    MissingExpectedNormalizedJson,
    MissingRawFixtures,
    InvalidExpectedNormalizedJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfParserFixtureCoverageRow {
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
    pub parser_fixture_parser_id: String,
    pub parser_fixture_schema_id: String,
    pub parser_fixture_root_path: String,
    pub expected_normalized_path: String,
    pub raw_fixture_count: usize,
    pub raw_fixture_paths: Vec<String>,
    pub coverage_status: VcfParserFixtureCoverageStatus,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfParserFixtureCoverageReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub stage_count: usize,
    pub tool_count: usize,
    pub row_count: usize,
    pub covered_row_count: usize,
    pub missing_row_count: usize,
    pub parser_fixture_coverage_percent: f64,
    pub coverage_status_counts: BTreeMap<String, usize>,
    pub rows: Vec<VcfParserFixtureCoverageRow>,
}

pub const DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-parser-fixture-coverage.tsv";
