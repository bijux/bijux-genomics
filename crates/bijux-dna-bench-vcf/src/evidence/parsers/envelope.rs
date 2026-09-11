use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use std::path::Path;
const VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.vcf_raw_parser.v1";
#[derive(Deserialize)]
struct FixtureEnvelope {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    parser_id: String,
    #[serde(rename = "normalized")]
    _normalized: serde_json::Map<String, serde_json::Value>,
}
pub(super) fn validate_expected_normalized_fixture(
    expected_path: &Path,
    stage_id: &str,
    tool_id: &str,
    parser_id: &str,
) -> Result<()> {
    let raw = std::fs::read(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    let parsed: FixtureEnvelope = serde_json::from_slice(&raw)
        .with_context(|| format!("parse {}", expected_path.display()))?;
    ensure!(
        parsed.schema_version == VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION,
        "schema_version must be `{VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION}`"
    );
    ensure!(parsed.stage_id == stage_id, "stage_id must be `{stage_id}`");
    ensure!(parsed.tool_id == tool_id, "tool_id must be `{tool_id}`");
    ensure!(parsed.parser_id == parser_id, "parser_id must be `{parser_id}`");
    Ok(())
}
