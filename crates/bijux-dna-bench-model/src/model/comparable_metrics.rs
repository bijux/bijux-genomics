//! Owner: bijux-dna-bench-model
//! Persisted comparable-metric readiness reports for benchmark domains.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{
    BAM_COMPARABLE_METRICS_SCHEMA_V1, FASTQ_COMPARABLE_METRICS_SCHEMA_V1,
    VCF_COMPARABLE_METRICS_SCHEMA_V1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ComparableMetricContractStatus {
    Declared,
    MissingSharedMetrics,
}

impl ComparableMetricContractStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::MissingSharedMetrics => "missing_shared_metrics",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ComparableMetricDirection {
    ExactMatchPreferred,
    HigherIsBetter,
    LowerIsBetter,
}

impl ComparableMetricDirection {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactMatchPreferred => "exact_match_preferred",
            Self::HigherIsBetter => "higher_is_better",
            Self::LowerIsBetter => "lower_is_better",
        }
    }
}

impl std::fmt::Display for ComparableMetricDirection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FastqComparableMetricsRow {
    pub stage_id: String,
    pub comparison_contract_status: ComparableMetricContractStatus,
    pub tool_count: usize,
    pub tool_ids: Vec<String>,
    pub default_tool_id: String,
    pub corpus_status: String,
    pub shared_metric_field_count: usize,
    pub shared_metric_fields: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FastqComparableMetricsReport {
    #[schemars(schema_with = "fastq_comparable_metrics_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub comparable_stage_count: usize,
    pub multi_tool_stage_count: usize,
    pub comparable_tool_row_count: usize,
    pub row_count: usize,
    pub declared_stage_count: usize,
    pub missing_shared_metric_stage_count: usize,
    pub shared_metric_field_count: usize,
    pub rows: Vec<FastqComparableMetricsRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BamComparableMetricsRow {
    pub stage_id: String,
    pub comparison_contract_status: ComparableMetricContractStatus,
    pub tool_count: usize,
    pub tool_ids: Vec<String>,
    pub default_tool_id: String,
    pub corpus_status: String,
    pub shared_metric_field_count: usize,
    pub shared_metric_fields: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BamComparableMetricsReport {
    #[schemars(schema_with = "bam_comparable_metrics_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub comparable_stage_count: usize,
    pub multi_tool_stage_count: usize,
    pub comparable_tool_row_count: usize,
    pub row_count: usize,
    pub declared_stage_count: usize,
    pub missing_shared_metric_stage_count: usize,
    pub shared_metric_field_count: usize,
    pub rows: Vec<BamComparableMetricsRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VcfComparableMetricsRow {
    pub stage_id: String,
    pub metric_id: String,
    pub metric_name: String,
    pub unit: String,
    pub direction: ComparableMetricDirection,
    pub required: bool,
    pub tools_covered: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VcfComparableMetricsReport {
    #[schemars(schema_with = "vcf_comparable_metrics_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub stage_count: usize,
    pub multi_tool_stage_count: usize,
    pub retained_tool_row_count: usize,
    pub row_count: usize,
    pub rows: Vec<VcfComparableMetricsRow>,
}

#[must_use]
pub fn fastq_comparable_metrics_schema() -> Schema {
    schemars::schema_for!(FastqComparableMetricsReport)
}

#[must_use]
pub fn bam_comparable_metrics_schema() -> Schema {
    schemars::schema_for!(BamComparableMetricsReport)
}

#[must_use]
pub fn vcf_comparable_metrics_schema() -> Schema {
    schemars::schema_for!(VcfComparableMetricsReport)
}

fn fastq_comparable_metrics_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": FASTQ_COMPARABLE_METRICS_SCHEMA_V1})
}

fn bam_comparable_metrics_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": BAM_COMPARABLE_METRICS_SCHEMA_V1})
}

fn vcf_comparable_metrics_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": VCF_COMPARABLE_METRICS_SCHEMA_V1})
}
