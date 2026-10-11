//! Owner: bijux-dna-bench-model
//! Persisted local VCF identity-by-descent smoke records.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const BENCHMARK_LOCAL_VCF_IBD_SMOKE_SCHEMA_V1: &str = "bijux.bench.local_vcf_ibd_smoke.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalVcfIbdSmokeRow {
    pub sample_a: String,
    pub sample_b: String,
    pub segment_count: u64,
    pub total_length: f64,
    pub overlap_marker_count: u64,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalVcfIbdInsufficientOverlapProbe {
    pub input_vcf_path: String,
    pub ibd_status: String,
    pub insufficient_data_reason: String,
    pub overlap_marker_count: u64,
    pub filtered_segment_count: u64,
    pub source_ibd_summary_path: String,
    pub source_ibd_filtered_segments_path: String,
    pub unrelated_stage_id: String,
    pub unrelated_stage_status: String,
    pub unrelated_stage_segment_count: u64,
    pub source_roh_summary_path: String,
    pub source_roh_segments_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalVcfIbdSmokeReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub fixture_manifest_path: String,
    pub input_vcf_path: String,
    pub sample_metadata_path: String,
    pub output_root: String,
    pub ibd_tsv_path: String,
    pub ibd_json_path: String,
    pub source_ibd_input_path: String,
    pub source_ibd_segments_path: String,
    pub source_ibd_merged_segments_path: String,
    pub source_ibd_filtered_segments_path: String,
    pub source_ibd_summary_path: String,
    pub source_ibd_metrics_path: String,
    pub source_logs_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub execution_mode: String,
    pub germline_tool_ok: bool,
    pub ibdseq_tool_ok: bool,
    pub ibdhap_tool_ok: bool,
    pub pair_count: u64,
    pub rows: Vec<BenchmarkLocalVcfIbdSmokeRow>,
    pub insufficient_overlap_probe: BenchmarkLocalVcfIbdInsufficientOverlapProbe,
    pub status: String,
}

#[must_use]
pub fn benchmark_local_vcf_ibd_smoke_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalVcfIbdSmokeReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_VCF_IBD_SMOKE_SCHEMA_V1
    })
}
