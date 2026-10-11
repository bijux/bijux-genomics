//! Owner: bijux-dna-bench-model
//! Persisted cross-domain benchmark campaign records.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use super::{AppraisalFinding, HardeningQueueEntry};

pub const CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1: &str = "bijux.hpc.cross_benchmark_campaign.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrossBenchmarkCampaignReport {
    #[serde(deserialize_with = "deserialize_cross_campaign_schema_version")]
    #[schemars(schema_with = "cross_campaign_schema_version")]
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub selected_goals: Vec<String>,
    pub summary: CrossBenchmarkCampaignSummary,
    pub entries: Vec<CrossBenchmarkGoalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrossBenchmarkCampaignSummary {
    pub total_goals: usize,
    pub total_rows: usize,
    pub total_findings: usize,
    pub total_queue_entries: usize,
    pub status_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrossBenchmarkGoalEntry {
    pub goal_id: String,
    pub title: String,
    pub stage_ids: Vec<String>,
    pub scenarios: Vec<CrossScenario>,
    pub matched_rows: Vec<CrossMatrixRowRef>,
    pub appraisal_findings: Vec<AppraisalFinding>,
    pub hardening_entries: Vec<HardeningQueueEntry>,
    pub goal_checks: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrossScenario {
    pub scenario_id: String,
    pub focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrossMatrixRowRef {
    pub row_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub readiness_class: String,
    pub repetitions: u32,
}

#[must_use]
pub fn cross_benchmark_campaign_report_schema() -> Schema {
    schemars::schema_for!(CrossBenchmarkCampaignReport)
}

fn cross_campaign_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1,
    })
}

fn deserialize_cross_campaign_schema_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported cross-benchmark campaign schema: {value}")))
    }
}
