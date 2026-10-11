//! Owner: bijux-dna-bench-model
//! Persisted benchmark appraisal and hardening-queue records.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

pub const APPRAISAL_REPORT_SCHEMA_V1: &str = "bijux.hpc.appraisal.v1";
pub const HARDENING_QUEUE_REPORT_SCHEMA_V1: &str = "bijux.hpc.hardening_queue.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AppraisalReport {
    #[serde(deserialize_with = "deserialize_appraisal_report_schema_version")]
    #[schemars(schema_with = "appraisal_report_schema_version")]
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub findings: Vec<AppraisalFinding>,
    pub summary: AppraisalSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AppraisalFinding {
    pub appraiser_id: String,
    pub row_id: String,
    pub severity: String,
    pub confidence: String,
    pub failure_class: String,
    pub result_scope: String,
    pub summary: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AppraisalSummary {
    pub total_findings: usize,
    pub by_appraiser: BTreeMap<String, usize>,
    pub by_severity: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HardeningQueueReport {
    #[serde(deserialize_with = "deserialize_hardening_queue_report_schema_version")]
    #[schemars(schema_with = "hardening_queue_report_schema_version")]
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub entries: Vec<HardeningQueueEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HardeningQueueEntry {
    pub queue_id: String,
    pub severity: String,
    pub failure_class: String,
    pub recommendation: String,
    pub affected_rows: Vec<String>,
    pub source_appraisers: Vec<String>,
}

#[must_use]
pub fn appraisal_report_schema() -> Schema {
    schemars::schema_for!(AppraisalReport)
}

#[must_use]
pub fn hardening_queue_report_schema() -> Schema {
    schemars::schema_for!(HardeningQueueReport)
}

fn appraisal_report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": APPRAISAL_REPORT_SCHEMA_V1,
    })
}

fn hardening_queue_report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": HARDENING_QUEUE_REPORT_SCHEMA_V1,
    })
}

fn deserialize_appraisal_report_schema_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == APPRAISAL_REPORT_SCHEMA_V1 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported appraisal report schema: {value}")))
    }
}

fn deserialize_hardening_queue_report_schema_version<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == HARDENING_QUEUE_REPORT_SCHEMA_V1 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported hardening queue report schema: {value}")))
    }
}
