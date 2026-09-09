//! Persisted readiness coverage and command-projection wire contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

/// Persisted coverage report schema identity.
pub const ALL_DOMAIN_RENDERED_COMMANDS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_rendered_commands.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainRenderedCommandStep {
    pub step_id: String,
    pub step_kind: String,
    pub consumes_previous_stdout: bool,
    pub argv: Vec<String>,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainRenderedCommandRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub readiness_kind: String,
    pub benchmark_status: String,
    pub command_source: String,
    pub command_steps: Vec<AllDomainRenderedCommandStep>,
    pub script_commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainRenderedCommandArgvStep {
    pub step_id: String,
    pub step_kind: String,
    pub consumes_previous_stdout: bool,
    pub argv: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainRenderedCommandArgvRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub readiness_kind: String,
    pub benchmark_status: String,
    pub command_source: String,
    pub command_steps: Vec<AllDomainRenderedCommandArgvStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainRenderedCommandsReport {
    #[serde(deserialize_with = "deserialize_schema_version")]
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub argv_output_path: String,
    pub row_count: usize,
    pub result_id_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub benchmark_status_counts: BTreeMap<String, usize>,
    pub command_source_counts: BTreeMap<String, usize>,
    pub rows: Vec<AllDomainRenderedCommandRow>,
}

fn deserialize_schema_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != ALL_DOMAIN_RENDERED_COMMANDS_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(
            "unsupported readiness coverage report schema version",
        ));
    }
    Ok(version)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":ALL_DOMAIN_RENDERED_COMMANDS_SCHEMA_VERSION})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wire_report() -> serde_json::Value {
        serde_json::json!({
            "schema_version":ALL_DOMAIN_RENDERED_COMMANDS_SCHEMA_VERSION,
            "output_path":"artifacts/commands.sh", "argv_output_path":"artifacts/commands.jsonl",
            "row_count":1, "result_id_count":1, "domain_counts":{"fastq":1},
            "benchmark_status_counts":{"benchmark_ready":1}, "command_source_counts":{"adapter":1},
            "rows":[{"result_id":"result", "domain":"fastq", "stage_id":"fastq.filter_reads",
                "tool_id":"fastp", "corpus_id":"corpus", "asset_profile_id":"asset", "readiness_kind":"stage",
                "benchmark_status":"benchmark_ready", "command_source":"adapter",
                "command_steps":[{"step_id":"filter", "step_kind":"command", "consumes_previous_stdout":false,
                    "argv":["fastp","--in1","reads with spaces.fq.gz"], "command":"fastp --in1 'reads with spaces.fq.gz'"},
                    {"step_id":"normalize", "step_kind":"command", "consumes_previous_stdout":true,
                    "argv":["normalize","--stdin"], "command":"normalize --stdin"}],
                "script_commands":["fastp --in1 'reads with spaces.fq.gz'", "normalize --stdin"]}]
        })
    }

    #[test]
    fn rendered_command_contract_preserves_argv_tokens_and_pipeline_handoffs() -> anyhow::Result<()>
    {
        let wire = wire_report();
        let report: AllDomainRenderedCommandsReport = serde_json::from_value(wire.clone())?;
        assert_eq!(serde_json::to_value(&report)?, wire);
        assert_eq!(report.rows[0].command_steps[0].argv[2], "reads with spaces.fq.gz");
        assert!(report.rows[0].command_steps[1].consumes_previous_stdout);
        let mut argv = wire["rows"][0].clone();
        assert!(argv
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("row object"))?
            .remove("script_commands")
            .is_some());
        for step in
            argv["command_steps"].as_array_mut().ok_or_else(|| anyhow::anyhow!("steps array"))?
        {
            assert!(step
                .as_object_mut()
                .ok_or_else(|| anyhow::anyhow!("step object"))?
                .remove("command")
                .is_some());
        }
        let row: AllDomainRenderedCommandArgvRow = serde_json::from_value(argv.clone())?;
        assert_eq!(serde_json::to_value(row)?, argv);
        argv["command_steps"][0]["command"] = serde_json::json!("undeclared shell field");
        assert!(serde_json::from_value::<AllDomainRenderedCommandArgvRow>(argv).is_err());
        Ok(())
    }

    #[test]
    fn rendered_command_contract_rejects_unknown_versions_and_nested_fields() {
        let original = wire_report();
        let mut wire = original.clone();
        wire["schema_version"] = serde_json::json!("unknown.v1");
        assert!(serde_json::from_value::<AllDomainRenderedCommandsReport>(wire).is_err());
        let mut wire = original.clone();
        wire["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<AllDomainRenderedCommandsReport>(wire).is_err());
        let mut wire = original.clone();
        wire["rows"][0]["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<AllDomainRenderedCommandsReport>(wire).is_err());
        let mut wire = original;
        wire["rows"][0]["command_steps"][0]["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<AllDomainRenderedCommandsReport>(wire).is_err());
    }
}
