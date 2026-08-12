//! Owner: bijux-dna-bench-model
//! Persisted stage-, tool-, and corpus-centric benchmark readiness views.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{
    CORPUS_CENTRIC_REPORT_SCHEMA_V1, STAGE_CENTRIC_REPORT_SCHEMA_V1, TOOL_CENTRIC_REPORT_SCHEMA_V1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessComparisonContractStatus {
    Declared,
    NotDeclared,
    NotApplicable,
}

impl ReadinessComparisonContractStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::NotDeclared => "not_declared",
            Self::NotApplicable => "not_applicable",
        }
    }
}

impl std::fmt::Display for ReadinessComparisonContractStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageCentricToolRow {
    pub tool_id: String,
    pub benchmark_status: String,
    pub readiness_gap: String,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
    pub corpus_status: String,
    pub asset_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageCentricStageReport {
    pub domain: String,
    pub stage_id: String,
    pub canonical_stage_rank: usize,
    pub report_section_id: String,
    pub report_section_title: String,
    pub summary_table_id: String,
    pub summary_table_title: String,
    pub anchor_tool_id: String,
    pub anchor_support_status: String,
    pub comparison_contract_status: ReadinessComparisonContractStatus,
    pub shared_metric_field_count: usize,
    pub shared_metric_fields: Vec<String>,
    pub tool_count: usize,
    pub benchmark_ready_tool_count: usize,
    pub blocked_tool_count: usize,
    pub readiness_gap_counts: BTreeMap<String, usize>,
    pub blocked_tool_ids: Vec<String>,
    pub tools: Vec<StageCentricToolRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StageCentricReport {
    #[schemars(schema_with = "stage_centric_report_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub stage_count: usize,
    pub multi_tool_stage_count: usize,
    pub blocked_stage_count: usize,
    pub declared_shared_metric_stage_count: usize,
    pub not_declared_shared_metric_stage_count: usize,
    pub row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub blocked_row_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub section_counts: BTreeMap<String, usize>,
    pub readiness_gap_counts: BTreeMap<String, usize>,
    pub stages: Vec<StageCentricStageReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolCentricStageRow {
    pub domain: String,
    pub stage_id: String,
    pub report_section_id: String,
    pub report_section_title: String,
    pub summary_table_id: String,
    pub summary_table_title: String,
    pub benchmark_status: String,
    pub readiness_gap: String,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
    pub corpus_status: String,
    pub asset_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolCentricToolReport {
    pub tool_id: String,
    pub domains: Vec<String>,
    pub stage_count: usize,
    pub benchmark_ready_stage_count: usize,
    pub blocked_stage_count: usize,
    pub report_section_ids: Vec<String>,
    pub readiness_gap_counts: BTreeMap<String, usize>,
    pub blocked_stage_ids: Vec<String>,
    pub stages: Vec<ToolCentricStageRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolCentricReport {
    #[schemars(schema_with = "tool_centric_report_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub tool_count: usize,
    pub unique_stage_count: usize,
    pub row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub blocked_row_count: usize,
    pub blocked_tool_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub readiness_gap_counts: BTreeMap<String, usize>,
    pub tools: Vec<ToolCentricToolReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CorpusCentricStageRow {
    pub domain: String,
    pub stage_id: String,
    pub canonical_stage_rank: usize,
    pub fixture_ids: Vec<String>,
    pub report_section_id: String,
    pub report_section_title: String,
    pub summary_table_id: String,
    pub summary_table_title: String,
    pub comparison_contract_status: ReadinessComparisonContractStatus,
    pub shared_metric_field_count: usize,
    pub shared_metric_fields: Vec<String>,
    pub tool_count: usize,
    pub benchmark_ready_tool_count: usize,
    pub blocked_tool_count: usize,
    pub tool_ids: Vec<String>,
    pub blocked_tool_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CorpusCentricCorpusReport {
    pub corpus_family_id: String,
    pub domains: Vec<String>,
    pub fixture_ids: Vec<String>,
    pub stage_count: usize,
    pub tool_row_count: usize,
    pub benchmark_ready_tool_row_count: usize,
    pub blocked_stage_count: usize,
    pub blocked_stage_ids: Vec<String>,
    pub stages: Vec<CorpusCentricStageRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CorpusCentricReport {
    #[schemars(schema_with = "corpus_centric_report_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub corpus_count: usize,
    pub stage_count: usize,
    pub tool_row_count: usize,
    pub benchmark_ready_tool_row_count: usize,
    pub blocked_tool_row_count: usize,
    pub blocked_corpus_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub corpus_stage_counts: BTreeMap<String, usize>,
    pub corpora: Vec<CorpusCentricCorpusReport>,
}

#[must_use]
pub fn stage_centric_report_schema() -> Schema {
    schemars::schema_for!(StageCentricReport)
}

#[must_use]
pub fn tool_centric_report_schema() -> Schema {
    schemars::schema_for!(ToolCentricReport)
}

#[must_use]
pub fn corpus_centric_report_schema() -> Schema {
    schemars::schema_for!(CorpusCentricReport)
}

fn stage_centric_report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": STAGE_CENTRIC_REPORT_SCHEMA_V1})
}

fn tool_centric_report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": TOOL_CENTRIC_REPORT_SCHEMA_V1})
}

fn corpus_centric_report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": CORPUS_CENTRIC_REPORT_SCHEMA_V1})
}
