//! Persisted summary metrics for the domain-spanning benchmark dashboard.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};

/// Version of the persisted benchmark dashboard envelope.
pub const FULL_BENCHMARK_DASHBOARD_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.full_benchmark_dashboard.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkDashboardMetric {
    pub metric_id: String,
    pub count: usize,
    pub source_path: String,
    pub source_field: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkDashboardReport {
    #[serde(deserialize_with = "deserialize_schema_version")]
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub markdown_output_path: String,
    pub json_output_path: String,
    pub total_stages: usize,
    pub total_tools: usize,
    pub total_expected_jobs: usize,
    pub ready_jobs: usize,
    pub blocked_jobs: usize,
    pub missing_parsers: usize,
    pub missing_adapters: usize,
    pub missing_assets: usize,
    pub failed_real_smokes: usize,
    pub explicit_unsupported_pairs: usize,
    pub passes_behavior_test: bool,
    pub metrics: Vec<FullBenchmarkDashboardMetric>,
}

fn deserialize_schema_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != FULL_BENCHMARK_DASHBOARD_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(
            "unsupported full benchmark dashboard schema version",
        ));
    }
    Ok(version)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": FULL_BENCHMARK_DASHBOARD_SCHEMA_VERSION})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wire_report() -> serde_json::Value {
        serde_json::json!({
            "schema_version": FULL_BENCHMARK_DASHBOARD_SCHEMA_VERSION,
            "markdown_output_path":"artifacts/dashboard.md", "json_output_path":"artifacts/dashboard.json",
            "total_stages":71, "total_tools":5, "total_expected_jobs":7, "ready_jobs":4,
            "blocked_jobs":3, "missing_parsers":1, "missing_adapters":1, "missing_assets":1,
            "failed_real_smokes":1, "explicit_unsupported_pairs":2, "passes_behavior_test":false,
            "metrics":[{"metric_id":"ready_jobs", "count":4, "source_path":"artifacts/report.json",
                "source_field":"present_row_count", "detail":"observed ready jobs"}]
        })
    }

    #[test]
    fn dashboard_wire_contract_round_trips_without_changing_persisted_fields() -> anyhow::Result<()>
    {
        let wire = wire_report();
        let report: FullBenchmarkDashboardReport = serde_json::from_value(wire.clone())?;
        assert_eq!(serde_json::to_value(&report)?, wire);
        let schema = serde_json::to_value(schemars::schema_for!(FullBenchmarkDashboardReport))?;
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(
            schema["properties"]["schema_version"]["const"],
            FULL_BENCHMARK_DASHBOARD_SCHEMA_VERSION
        );
        assert_eq!(schema["$defs"]["FullBenchmarkDashboardMetric"]["additionalProperties"], false);
        Ok(())
    }

    #[test]
    fn dashboard_input_rejects_unknown_versions_fields_and_invalid_counts() {
        let original = wire_report();
        for invalid in [serde_json::json!("other.v1"), serde_json::Value::Null] {
            let mut wire = original.clone();
            wire["schema_version"] = invalid;
            assert!(serde_json::from_value::<FullBenchmarkDashboardReport>(wire).is_err());
        }
        let mut wire = original.clone();
        wire["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<FullBenchmarkDashboardReport>(wire).is_err());
        let mut wire = original.clone();
        wire["metrics"][0]["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<FullBenchmarkDashboardReport>(wire).is_err());
        for invalid in [serde_json::json!(-1), serde_json::json!(1.5)] {
            let mut wire = original.clone();
            wire["ready_jobs"] = invalid;
            assert!(serde_json::from_value::<FullBenchmarkDashboardReport>(wire).is_err());
        }
    }
}
