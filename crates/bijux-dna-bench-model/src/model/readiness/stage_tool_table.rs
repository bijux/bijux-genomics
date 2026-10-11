//! Persisted readiness coverage and command-projection wire contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

/// Persisted coverage report schema identity.
pub const ALL_DOMAIN_STAGE_TOOL_TABLE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_stage_tool_table.v1";

/// Ordered identity for stage/tool readiness coverage.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReadinessStageToolBinding {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainStageToolTableRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub benchmark_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainStageToolTableReport {
    #[serde(deserialize_with = "deserialize_schema_version")]
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub benchmark_ready_unique_binding_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub benchmark_ready_domain_counts: BTreeMap<String, usize>,
    pub rows: Vec<AllDomainStageToolTableRow>,
}

fn deserialize_schema_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != ALL_DOMAIN_STAGE_TOOL_TABLE_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(
            "unsupported readiness coverage report schema version",
        ));
    }
    Ok(version)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":ALL_DOMAIN_STAGE_TOOL_TABLE_SCHEMA_VERSION})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wire_report() -> serde_json::Value {
        serde_json::json!({
            "schema_version":ALL_DOMAIN_STAGE_TOOL_TABLE_SCHEMA_VERSION, "output_path":"artifacts/table.tsv",
            "row_count":1, "benchmark_ready_row_count":0, "benchmark_ready_unique_binding_count":0,
            "domain_counts":{"fastq":1}, "benchmark_ready_domain_counts":{},
            "rows":[{"domain":"fastq", "stage_id":"fastq.filter_reads", "tool_id":"fastp",
                "corpus_id":"not_assigned", "asset_profile_id":"not_assigned", "adapter_id":"adapter",
                "parser_id":"parser", "benchmark_status":"blocked"}]
        })
    }

    #[test]
    fn stage_tool_coverage_wire_contract_preserves_blocked_rows() -> anyhow::Result<()> {
        let wire = wire_report();
        let report: AllDomainStageToolTableReport = serde_json::from_value(wire.clone())?;
        assert_eq!(serde_json::to_value(report)?, wire);
        Ok(())
    }

    #[test]
    fn stage_tool_coverage_rejects_unknown_versions_and_nested_fields() {
        let original = wire_report();
        let mut wire = original.clone();
        wire["schema_version"] = serde_json::json!("unknown.v1");
        assert!(serde_json::from_value::<AllDomainStageToolTableReport>(wire).is_err());
        let mut wire = original.clone();
        wire["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<AllDomainStageToolTableReport>(wire).is_err());
        let mut wire = original;
        wire["rows"][0]["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<AllDomainStageToolTableReport>(wire).is_err());
    }
}
