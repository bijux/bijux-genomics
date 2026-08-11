//! Owner: bijux-dna-bench-model
//! Persisted FASTQ, BAM, VCF, and hardening benchmark campaign records.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use super::{AppraisalFinding, HardeningQueueEntry};

pub const FASTQ_BENCHMARK_CAMPAIGN_SCHEMA_V1: &str = "bijux.hpc.fastq_benchmark_campaign.v1";
pub const BAM_BENCHMARK_CAMPAIGN_SCHEMA_V1: &str = "bijux.hpc.bam_benchmark_campaign.v1";
pub const VCF_BENCHMARK_CAMPAIGN_SCHEMA_V1: &str = "bijux.hpc.vcf_benchmark_campaign.v1";
pub const HARDENING_BENCHMARK_CAMPAIGN_SCHEMA_V1: &str =
    "bijux.hpc.hardening_benchmark_campaign.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FastqBenchmarkCampaignReport {
    #[serde(deserialize_with = "deserialize_fastq_campaign_schema_version")]
    #[schemars(schema_with = "fastq_campaign_schema_version")]
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub selected_goals: Vec<String>,
    pub summary: DomainBenchmarkCampaignSummary,
    pub entries: Vec<DomainBenchmarkGoalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamBenchmarkCampaignReport {
    #[serde(deserialize_with = "deserialize_bam_campaign_schema_version")]
    #[schemars(schema_with = "bam_campaign_schema_version")]
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub selected_goals: Vec<String>,
    pub summary: DomainBenchmarkCampaignSummary,
    pub entries: Vec<DomainBenchmarkGoalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfBenchmarkCampaignReport {
    #[serde(deserialize_with = "deserialize_vcf_campaign_schema_version")]
    #[schemars(schema_with = "vcf_campaign_schema_version")]
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub selected_goals: Vec<String>,
    pub summary: DomainBenchmarkCampaignSummary,
    pub entries: Vec<DomainBenchmarkGoalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HardeningBenchmarkCampaignReport {
    #[serde(deserialize_with = "deserialize_hardening_campaign_schema_version")]
    #[schemars(schema_with = "hardening_campaign_schema_version")]
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub selected_goals: Vec<String>,
    pub summary: DomainBenchmarkCampaignSummary,
    pub entries: Vec<DomainBenchmarkGoalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomainBenchmarkCampaignSummary {
    pub total_goals: usize,
    pub total_rows: usize,
    pub total_findings: usize,
    pub total_queue_entries: usize,
    pub status_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomainBenchmarkGoalEntry {
    pub goal_id: String,
    pub title: String,
    pub stage_ids: Vec<String>,
    pub scenarios: Vec<DomainBenchmarkScenario>,
    pub matched_rows: Vec<DomainBenchmarkMatrixRowRef>,
    pub appraisal_findings: Vec<AppraisalFinding>,
    pub hardening_entries: Vec<HardeningQueueEntry>,
    pub goal_checks: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomainBenchmarkScenario {
    pub scenario_id: String,
    pub focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomainBenchmarkMatrixRowRef {
    pub row_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub readiness_class: String,
    pub repetitions: u32,
}

pub type FastqBenchmarkCampaignSummary = DomainBenchmarkCampaignSummary;
pub type FastqBenchmarkGoalEntry = DomainBenchmarkGoalEntry;
pub type FastqScenario = DomainBenchmarkScenario;
pub type FastqMatrixRowRef = DomainBenchmarkMatrixRowRef;

pub type BamBenchmarkCampaignSummary = DomainBenchmarkCampaignSummary;
pub type BamBenchmarkGoalEntry = DomainBenchmarkGoalEntry;
pub type BamScenario = DomainBenchmarkScenario;
pub type BamMatrixRowRef = DomainBenchmarkMatrixRowRef;

pub type VcfBenchmarkCampaignSummary = DomainBenchmarkCampaignSummary;
pub type VcfBenchmarkGoalEntry = DomainBenchmarkGoalEntry;
pub type VcfScenario = DomainBenchmarkScenario;
pub type VcfMatrixRowRef = DomainBenchmarkMatrixRowRef;

pub type HardeningBenchmarkCampaignSummary = DomainBenchmarkCampaignSummary;
pub type HardeningBenchmarkGoalEntry = DomainBenchmarkGoalEntry;
pub type HardeningScenario = DomainBenchmarkScenario;
pub type HardeningMatrixRowRef = DomainBenchmarkMatrixRowRef;

#[must_use]
pub fn fastq_benchmark_campaign_report_schema() -> Schema {
    schemars::schema_for!(FastqBenchmarkCampaignReport)
}

#[must_use]
pub fn bam_benchmark_campaign_report_schema() -> Schema {
    schemars::schema_for!(BamBenchmarkCampaignReport)
}

#[must_use]
pub fn vcf_benchmark_campaign_report_schema() -> Schema {
    schemars::schema_for!(VcfBenchmarkCampaignReport)
}

#[must_use]
pub fn hardening_benchmark_campaign_report_schema() -> Schema {
    schemars::schema_for!(HardeningBenchmarkCampaignReport)
}

fn fastq_campaign_schema_version(_: &mut SchemaGenerator) -> Schema {
    exact_version_schema(FASTQ_BENCHMARK_CAMPAIGN_SCHEMA_V1)
}

fn bam_campaign_schema_version(_: &mut SchemaGenerator) -> Schema {
    exact_version_schema(BAM_BENCHMARK_CAMPAIGN_SCHEMA_V1)
}

fn vcf_campaign_schema_version(_: &mut SchemaGenerator) -> Schema {
    exact_version_schema(VCF_BENCHMARK_CAMPAIGN_SCHEMA_V1)
}

fn hardening_campaign_schema_version(_: &mut SchemaGenerator) -> Schema {
    exact_version_schema(HARDENING_BENCHMARK_CAMPAIGN_SCHEMA_V1)
}

fn exact_version_schema(expected: &'static str) -> Schema {
    json_schema!({
        "type": "string",
        "const": expected,
    })
}

fn deserialize_fastq_campaign_schema_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_exact_version(deserializer, FASTQ_BENCHMARK_CAMPAIGN_SCHEMA_V1, "FASTQ")
}

fn deserialize_bam_campaign_schema_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_exact_version(deserializer, BAM_BENCHMARK_CAMPAIGN_SCHEMA_V1, "BAM")
}

fn deserialize_vcf_campaign_schema_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_exact_version(deserializer, VCF_BENCHMARK_CAMPAIGN_SCHEMA_V1, "VCF")
}

fn deserialize_hardening_campaign_schema_version<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_exact_version(deserializer, HARDENING_BENCHMARK_CAMPAIGN_SCHEMA_V1, "hardening")
}

fn deserialize_exact_version<'de, D>(
    deserializer: D,
    expected: &str,
    label: &str,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == expected {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported {label} benchmark campaign schema: {value}")))
    }
}
