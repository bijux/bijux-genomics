//! Persisted observations from the governed local smoke subset.
//! Declared counts and execution provenance require separate semantic admission.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Version of the local smoke subset observation envelope.
pub const REAL_SMOKE_CORE_SUBSET_SCHEMA_VERSION: &str =
    "bijux.bench.local_real_smoke_core_subset.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RealSmokeCoreSubsetExecutionKind {
    Stage,
    PipelineBridge,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RealSmokeCoreSubsetRow {
    pub execution_id: String,
    pub execution_kind: RealSmokeCoreSubsetExecutionKind,
    pub domain: String,
    pub bridge_source_domain: Option<String>,
    pub bridge_target_domain: Option<String>,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub evidence_path: String,
    pub parsed_schema_version: String,
    pub stage_result_manifest_path: Option<String>,
    pub manifest_status: Option<String>,
    pub manifest_exit_code: Option<i32>,
    pub normalized_metric_count: usize,
    pub normalized_metrics: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RealSmokeCoreSubsetReport {
    #[serde(deserialize_with = "deserialize_schema_version")]
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub execution_count: usize,
    pub stage_execution_count: usize,
    pub pipeline_bridge_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub passes_behavior_test: bool,
    pub rows: Vec<RealSmokeCoreSubsetRow>,
}

fn deserialize_schema_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != REAL_SMOKE_CORE_SUBSET_SCHEMA_VERSION {
        return Err(serde::de::Error::custom("unsupported local smoke core subset schema version"));
    }
    Ok(version)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":REAL_SMOKE_CORE_SUBSET_SCHEMA_VERSION})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wire_report() -> Value {
        serde_json::json!({
            "schema_version":REAL_SMOKE_CORE_SUBSET_SCHEMA_VERSION, "output_path":"artifacts/smoke.json",
            "execution_count":1, "stage_execution_count":0, "pipeline_bridge_count":1,
            "domain_counts":{"vcf":1}, "passes_behavior_test":false,
            "rows":[{"execution_id":"bridge", "execution_kind":"pipeline_bridge", "domain":"vcf",
                "bridge_source_domain":"bam", "bridge_target_domain":"vcf", "stage_id":"vcf.call",
                "tool_id":"bcftools", "corpus_id":"case", "asset_profile_id":"cohort",
                "evidence_path":"artifacts/evidence.json", "parsed_schema_version":"source.v1",
                "stage_result_manifest_path":"artifacts/manifest.json", "manifest_status":"failed",
                "manifest_exit_code":1, "normalized_metric_count":2,
                "normalized_metrics":{"count":3,"fraction":0.5}}]
        })
    }

    #[test]
    fn smoke_observation_contract_preserves_failure_and_bridge_evidence() -> anyhow::Result<()> {
        let wire = wire_report();
        let report: RealSmokeCoreSubsetReport = serde_json::from_value(wire.clone())?;
        assert_eq!(serde_json::to_value(&report)?, wire);
        assert_eq!(report.rows[0].execution_kind, RealSmokeCoreSubsetExecutionKind::PipelineBridge);
        assert_eq!(report.rows[0].manifest_exit_code, Some(1));
        let schema = serde_json::to_value(schemars::schema_for!(RealSmokeCoreSubsetReport))?;
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["$defs"]["RealSmokeCoreSubsetRow"]["additionalProperties"], false);
        Ok(())
    }

    #[test]
    fn smoke_observation_input_rejects_unknown_schema_fields_and_execution_kinds() {
        let original = wire_report();
        let mut wire = original.clone();
        wire["schema_version"] = serde_json::json!("unknown.v1");
        assert!(serde_json::from_value::<RealSmokeCoreSubsetReport>(wire).is_err());
        let mut wire = original.clone();
        wire["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<RealSmokeCoreSubsetReport>(wire).is_err());
        let mut wire = original.clone();
        wire["rows"][0]["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<RealSmokeCoreSubsetReport>(wire).is_err());
        let mut wire = original;
        wire["rows"][0]["execution_kind"] = serde_json::json!("pretend");
        assert!(serde_json::from_value::<RealSmokeCoreSubsetReport>(wire).is_err());
    }
}
