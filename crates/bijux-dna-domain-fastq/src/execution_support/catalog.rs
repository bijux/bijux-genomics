use std::sync::OnceLock;

use bijux_dna_core::ids::{StageId, ToolId};
use serde::Deserialize;

use super::model::{
    BenchmarkSupport, ExecutionStatus, NormalizationSupport, PlanningSupport, RuntimeSupport,
    StageExecutionSupport,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionSupportManifest {
    schema_version: String,
    stages: Vec<ExecutionSupportRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionSupportRecord {
    stage_id: String,
    execution_status: ExecutionStatus,
    planning_support: PlanningSupport,
    runtime_support: RuntimeSupport,
    normalization_support: NormalizationSupport,
    benchmark_support: BenchmarkSupport,
    default_tool: Option<String>,
    admitted_tools: Vec<String>,
}

fn manifest() -> &'static ExecutionSupportManifest {
    static MANIFEST: OnceLock<ExecutionSupportManifest> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        let manifest: ExecutionSupportManifest = bijux_dna_infra::formats::parse_yaml(
            include_str!("../../../../domain/fastq/execution_support.yaml"),
        )
        .unwrap_or_else(|err| panic!("parse domain/fastq/execution_support.yaml: {err}"));
        assert_eq!(
            manifest.schema_version, "bijux.fastq.execution_support.v1",
            "unexpected FASTQ execution support schema version",
        );
        manifest
    })
}

fn record_to_support(record: &ExecutionSupportRecord) -> StageExecutionSupport {
    StageExecutionSupport {
        stage_id: StageId::new(record.stage_id.clone()),
        execution_status: record.execution_status,
        planning_support: record.planning_support,
        runtime_support: record.runtime_support,
        normalization_support: record.normalization_support,
        benchmark_support: record.benchmark_support,
        default_tool: record
            .default_tool
            .as_ref()
            .map(|tool| ToolId::new(tool.clone())),
        admitted_tools: record
            .admitted_tools
            .iter()
            .cloned()
            .map(ToolId::new)
            .collect(),
    }
}

#[must_use]
pub fn execution_support_for_stage(stage_id: &StageId) -> Option<StageExecutionSupport> {
    manifest()
        .stages
        .iter()
        .find(|record| record.stage_id == stage_id.as_str())
        .map(record_to_support)
}

#[must_use]
pub fn admitted_tools_for_stage(stage_id: &StageId) -> Vec<ToolId> {
    execution_support_for_stage(stage_id)
        .map(|support| support.admitted_tools)
        .unwrap_or_default()
}

#[must_use]
pub fn default_tool_for_stage(stage_id: &StageId) -> Option<ToolId> {
    execution_support_for_stage(stage_id).and_then(|support| support.default_tool)
}

#[must_use]
pub fn closed_stage_ids() -> Vec<StageId> {
    manifest()
        .stages
        .iter()
        .filter(|record| record.execution_status == ExecutionStatus::Closed)
        .map(|record| StageId::new(record.stage_id.clone()))
        .collect()
}

#[must_use]
pub fn declared_only_stage_ids() -> Vec<StageId> {
    manifest()
        .stages
        .iter()
        .filter(|record| record.execution_status == ExecutionStatus::DeclaredOnly)
        .map(|record| StageId::new(record.stage_id.clone()))
        .collect()
}

#[must_use]
pub fn plannable_stage_ids() -> Vec<StageId> {
    manifest()
        .stages
        .iter()
        .filter(|record| record.planning_support == PlanningSupport::StageFamily)
        .map(|record| StageId::new(record.stage_id.clone()))
        .collect()
}

#[must_use]
pub fn runnable_stage_ids() -> Vec<StageId> {
    manifest()
        .stages
        .iter()
        .filter(|record| record.runtime_support == RuntimeSupport::Runnable)
        .map(|record| StageId::new(record.stage_id.clone()))
        .collect()
}

#[must_use]
pub fn benchmark_cohort_stage_ids() -> Vec<StageId> {
    manifest()
        .stages
        .iter()
        .filter(|record| {
            matches!(
                record.benchmark_support,
                BenchmarkSupport::Cohort | BenchmarkSupport::Comparable | BenchmarkSupport::Mixed
            )
        })
        .map(|record| StageId::new(record.stage_id.clone()))
        .collect()
}

#[must_use]
pub fn comparable_benchmark_stage_ids() -> Vec<StageId> {
    manifest()
        .stages
        .iter()
        .filter(|record| {
            matches!(
                record.benchmark_support,
                BenchmarkSupport::Comparable | BenchmarkSupport::Mixed
            )
        })
        .map(|record| StageId::new(record.stage_id.clone()))
        .collect()
}

#[must_use]
pub fn all_stage_execution_support() -> Vec<StageExecutionSupport> {
    manifest().stages.iter().map(record_to_support).collect()
}
