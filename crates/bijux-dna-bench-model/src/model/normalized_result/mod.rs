//! Canonical persisted normalized result records grouped by benchmark domain.

pub mod bam;
pub mod bam_envelope;
pub mod fastq;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

pub const BAM_DOWNSTREAM_PROBE_SCHEMA_V1: &str = "bijux.benchmark.downstream_probe.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDownstreamProbeRecord {
    #[serde(deserialize_with = "deserialize_probe_schema_version")]
    #[schemars(schema_with = "probe_schema_version")]
    pub schema_version: String,
    #[serde(deserialize_with = "deserialize_probe_stage_id")]
    #[schemars(schema_with = "probe_stage_id")]
    pub stage_id: String,
    #[serde(deserialize_with = "deserialize_probe_input_contract")]
    #[schemars(schema_with = "probe_input_contract")]
    pub input_contract: String,
    pub complete: bool,
}

#[must_use]
pub fn bam_downstream_probe_schema() -> Schema {
    schemars::schema_for!(BenchmarkDownstreamProbeRecord)
}

fn probe_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": BAM_DOWNSTREAM_PROBE_SCHEMA_V1})
}

fn probe_stage_id(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "enum": [
            "benchmark.bam.align.compare",
            "benchmark.bam.coverage.compare",
            "benchmark.bam.duplication_metrics.compare",
            "benchmark.bam.endogenous_content.consume",
            "benchmark.bam.genotyping.consume",
            "benchmark.bam.gc_bias.consume",
            "benchmark.bam.insert_size.consume",
            "benchmark.bam.mapping_summary.compare"
        ]
    })
}

fn probe_input_contract(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "enum": [
            bam::BAM_ALIGN_RESULT_SCHEMA_V1,
            bam::BAM_COVERAGE_RESULT_SCHEMA_V1,
            bam::BAM_DUPLICATION_METRICS_RESULT_SCHEMA_V1,
            bam::BAM_ENDOGENOUS_CONTENT_RESULT_SCHEMA_V1,
            bam::BAM_GENOTYPING_RESULT_SCHEMA_V1,
            bam::BAM_GC_BIAS_RESULT_SCHEMA_V1,
            bam::BAM_INSERT_SIZE_RESULT_SCHEMA_V1,
            bam::BAM_MAPPING_SUMMARY_RESULT_SCHEMA_V1
        ]
    })
}

fn deserialize_probe_schema_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_probe_identity(deserializer, &[BAM_DOWNSTREAM_PROBE_SCHEMA_V1])
}

fn deserialize_probe_stage_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_probe_identity(
        deserializer,
        &[
            "benchmark.bam.align.compare",
            "benchmark.bam.coverage.compare",
            "benchmark.bam.duplication_metrics.compare",
            "benchmark.bam.endogenous_content.consume",
            "benchmark.bam.genotyping.consume",
            "benchmark.bam.gc_bias.consume",
            "benchmark.bam.insert_size.consume",
            "benchmark.bam.mapping_summary.compare",
        ],
    )
}

fn deserialize_probe_input_contract<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_probe_identity(
        deserializer,
        &[
            bam::BAM_ALIGN_RESULT_SCHEMA_V1,
            bam::BAM_COVERAGE_RESULT_SCHEMA_V1,
            bam::BAM_DUPLICATION_METRICS_RESULT_SCHEMA_V1,
            bam::BAM_ENDOGENOUS_CONTENT_RESULT_SCHEMA_V1,
            bam::BAM_GENOTYPING_RESULT_SCHEMA_V1,
            bam::BAM_GC_BIAS_RESULT_SCHEMA_V1,
            bam::BAM_INSERT_SIZE_RESULT_SCHEMA_V1,
            bam::BAM_MAPPING_SUMMARY_RESULT_SCHEMA_V1,
        ],
    )
}

fn deserialize_probe_identity<'de, D>(deserializer: D, allowed: &[&str]) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if allowed.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported normalized BAM probe identity: {value}")))
    }
}
