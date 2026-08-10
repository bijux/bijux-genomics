//! Owner: bijux-dna-bench-model
//! Persisted container report-portfolio identity and freshness records.

use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};

pub const CONTAINER_REPORT_PORTFOLIO_INDEX_SCHEMA_V1: &str =
    "bijux.container.report_portfolio_index.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContainerReportFreshness {
    Current,
    HistoricalUnvalidated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerIndexedReportFile {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerIndexedReport {
    pub report_id: String,
    pub freshness: ContainerReportFreshness,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<ContainerIndexedReportFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<ContainerIndexedReportFile>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerReportPortfolioSummary {
    pub report_count: usize,
    pub current_count: usize,
    pub historical_unvalidated_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerReportPortfolioIndex {
    pub schema_version: String,
    pub summary: ContainerReportPortfolioSummary,
    pub reports: Vec<ContainerIndexedReport>,
}

#[must_use]
pub fn container_report_portfolio_index_schema() -> Schema {
    schemars::schema_for!(ContainerReportPortfolioIndex)
}
