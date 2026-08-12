//! Persisted local HPC result-collection simulation contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::BENCHMARK_LOCAL_HPC_RESULT_COLLECTION_SIMULATION_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcResultCollectionSimulationRow {
    pub record_id: String,
    pub source_surface: String,
    pub collection_status: String,
    pub job_id_local: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub result_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub evidence_path: String,
    pub manifest_path: Option<String>,
    pub declared_output_count: usize,
    pub present_output_count: usize,
    pub detail: String,
    pub insufficient_data_reason: Option<String>,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchmarkLocalHpcResultCollectionSimulationBehavior {
    pub complete_result_job_id: String,
    pub failed_result_job_id: String,
    pub missing_result_job_id: String,
    pub insufficient_result_job_id: String,
    pub unavailable_result_job_id: String,
    pub complete_row_present: bool,
    pub failed_row_present: bool,
    pub missing_row_present: bool,
    pub insufficient_row_present: bool,
    pub unavailable_row_present: bool,
    pub failed_distinct_from_missing: bool,
    pub insufficient_distinct_from_unavailable: bool,
    pub proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcResultCollectionSimulationReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub simulation_root: String,
    pub execution_resolver_path: String,
    pub row_count: usize,
    pub complete_row_count: usize,
    pub failed_row_count: usize,
    pub missing_row_count: usize,
    pub insufficient_row_count: usize,
    pub unavailable_row_count: usize,
    pub behavior: BenchmarkLocalHpcResultCollectionSimulationBehavior,
    pub rows: Vec<BenchmarkLocalHpcResultCollectionSimulationRow>,
}

#[must_use]
pub fn benchmark_local_hpc_result_collection_simulation_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalHpcResultCollectionSimulationReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_HPC_RESULT_COLLECTION_SIMULATION_SCHEMA_V1,
    })
}
