//! Persisted local benchmark stage-inventory contracts.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{
    BenchmarkLocalStageReadinessKind, BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageInventoryEntry {
    pub stage_id: String,
    pub readiness_kind: BenchmarkLocalStageReadinessKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageInventory {
    #[schemars(schema_with = "inventory_schema_version")]
    pub schema_version: String,
    pub domain: String,
    pub stage_matrix_schema_version: String,
    pub stage_matrix_path: String,
    pub stage_count: usize,
    pub stages: Vec<BenchmarkLocalStageInventoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalAllDomainStageInventory {
    #[schemars(schema_with = "all_domain_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub selected_domains: Vec<String>,
    pub domain_counts: BTreeMap<String, usize>,
    pub total_stage_count: usize,
    pub inventories: Vec<BenchmarkLocalStageInventory>,
}

#[must_use]
pub fn benchmark_local_stage_inventory_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalStageInventory)
}

#[must_use]
pub fn benchmark_local_all_domain_stage_inventory_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalAllDomainStageInventory)
}

fn inventory_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1,
    })
}

fn all_domain_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1,
    })
}
