//! Canonical normalized FASTQ result, measurement, and probe records.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const FASTQ_DETECT_ADAPTERS_RESULT_SCHEMA_V1: &str =
    "bijux.benchmark.fastq.detect_adapters.bundle.v1";
pub const FASTQ_DETECT_ADAPTERS_METRICS_SCHEMA_V1: &str =
    "bijux.benchmark.fastq.detect_adapters.metrics.v1";
pub const FASTQ_DETECT_ADAPTERS_PROBE_SCHEMA_V1: &str =
    "bijux.benchmark.fastq.detect_adapters.probe.v1";
pub const FASTQ_DETECT_ADAPTERS_STAGE_ID: &str = "fastq.detect_adapters";
pub const FASTQ_PROFILE_READ_LENGTHS_RESULT_SCHEMA_V1: &str =
    "bijux.benchmark.fastq.profile_read_lengths.bundle.v1";
pub const FASTQ_PROFILE_READ_LENGTHS_METRICS_SCHEMA_V1: &str =
    "bijux.benchmark.fastq.profile_read_lengths.metrics.v1";
pub const FASTQ_PROFILE_READ_LENGTHS_STAGE_ID: &str = "fastq.profile_read_lengths";
pub const FASTQ_PROFILE_READS_RESULT_SCHEMA_V1: &str =
    "bijux.benchmark.fastq.profile_reads.bundle.v1";
pub const FASTQ_PROFILE_READS_METRICS_SCHEMA_V1: &str =
    "bijux.benchmark.fastq.profile_reads.metrics.v1";
pub const FASTQ_PROFILE_READS_PROBE_SCHEMA_V1: &str = "bijux.benchmark.fastq.report_qc.probe.v1";
pub const FASTQ_PROFILE_READS_STAGE_ID: &str = "fastq.profile_reads";
pub const FASTQ_VALIDATE_READS_RESULT_SCHEMA_V1: &str =
    "bijux.benchmark.fastq.validate_reads.bundle.v1";
pub const FASTQ_VALIDATE_READS_METRICS_SCHEMA_V1: &str =
    "bijux.benchmark.fastq.validate_reads.metrics.v1";
pub const FASTQ_VALIDATE_READS_PROBE_SCHEMA_V1: &str =
    "bijux.benchmark.fastq.report_qc.validation_probe.v1";
pub const FASTQ_VALIDATE_READS_STAGE_ID: &str = "fastq.validate_reads";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqAdapterResultRow {
    #[schemars(schema_with = "nonempty_string_schema")]
    pub sample_id: String,
    #[schemars(schema_with = "paired_layout_schema")]
    pub input_layout: String,
    #[schemars(schema_with = "file_count_schema")]
    pub file_count: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub reads_in: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub reads_out: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub bases_in: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub bases_out: u64,
    #[schemars(schema_with = "nonnegative_number_schema")]
    pub mean_q: f64,
    pub candidate_adapter_count: u64,
    #[schemars(schema_with = "unique_nonempty_strings_schema")]
    pub detected_adapter_ids: Vec<String>,
    #[schemars(schema_with = "nonnegative_optional_number_schema")]
    pub detection_confidence: Option<f64>,
    #[schemars(schema_with = "nonnegative_number_schema")]
    pub detection_threshold: f64,
    #[schemars(schema_with = "nonnegative_number_schema")]
    pub adapter_trimmed_fraction: f64,
    #[schemars(schema_with = "nonnegative_number_schema")]
    pub adapter_content_max: f64,
    #[schemars(schema_with = "nonnegative_number_schema")]
    pub adapter_content_mean: f64,
    #[schemars(schema_with = "nonnegative_number_schema")]
    pub duplication_rate: f64,
    #[schemars(schema_with = "nonnegative_number_schema")]
    pub n_rate: f64,
    pub kmer_warning_count: u64,
    pub overrepresented_sequence_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqAdapterResult {
    #[schemars(schema_with = "detect_adapters_schema_version")]
    pub schema_version: String,
    #[schemars(schema_with = "detect_adapters_stage_id")]
    pub stage_id: String,
    #[schemars(schema_with = "fastqc_tool_id")]
    pub tool_id: String,
    #[schemars(length(min = 1))]
    pub results: Vec<BenchmarkFastqAdapterResultRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqAdapterMetrics {
    pub schema_version: String,
    pub sample_count: u64,
    pub file_count: u64,
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub mean_q_mean: f64,
    pub candidate_adapter_count_total: u64,
    pub samples_with_detected_adapters: u64,
    pub max_detection_confidence: f64,
    pub max_adapter_content_max: f64,
    pub adapter_content_mean_mean: f64,
    pub duplication_rate_mean: f64,
    pub n_rate_mean: f64,
    pub kmer_warning_count: u64,
    pub overrepresented_sequence_count: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqAdapterProbe {
    pub schema_version: String,
    pub consumer_stage_id: String,
    pub input_contract: String,
    pub sample_count: u64,
    pub file_count: u64,
    pub reads_in: u64,
    pub candidate_adapter_count_total: u64,
    pub samples_with_detected_adapters: u64,
    pub max_detection_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqReadLengthResultRow {
    #[schemars(schema_with = "sample_id_schema")]
    pub sample_id: String,
    #[schemars(schema_with = "paired_layout_schema")]
    pub input_layout: String,
    #[schemars(schema_with = "file_count_schema")]
    pub file_count: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub read_count: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub minimum_read_length: u64,
    #[schemars(schema_with = "positive_number_schema")]
    pub mean_read_length: f64,
    #[schemars(schema_with = "positive_number_schema")]
    pub median_read_length: f64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub maximum_read_length: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub distinct_read_lengths: u64,
    #[schemars(schema_with = "histogram_path_schema")]
    pub histogram_artifact: String,
    #[schemars(schema_with = "sha256_schema")]
    pub histogram_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqReadLengthResult {
    #[schemars(schema_with = "profile_read_lengths_schema_version")]
    pub schema_version: String,
    #[schemars(schema_with = "profile_read_lengths_stage_id")]
    pub stage_id: String,
    #[schemars(schema_with = "read_length_tool_id")]
    pub tool_id: String,
    #[schemars(length(min = 1))]
    pub results: Vec<BenchmarkFastqReadLengthResultRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqReadLengthMetrics {
    pub schema_version: String,
    pub sample_count: u64,
    pub file_count: u64,
    pub read_count: u64,
    pub minimum_read_length: u64,
    pub maximum_read_length: u64,
    pub exact_observation_mismatch_count: u64,
    pub histogram_mismatch_count: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqProfileResultRow {
    #[schemars(schema_with = "nonempty_string_schema")]
    pub sample_id: String,
    #[schemars(schema_with = "profile_layout_schema")]
    pub input_layout: String,
    #[schemars(schema_with = "file_count_schema")]
    pub file_count: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub read_count: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub base_count: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub minimum_read_length: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub maximum_read_length: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqProfileResult {
    #[schemars(schema_with = "profile_reads_schema_version")]
    pub schema_version: String,
    #[schemars(schema_with = "profile_reads_stage_id")]
    pub stage_id: String,
    #[schemars(schema_with = "profile_reads_tool_id")]
    pub tool_id: String,
    #[schemars(length(min = 1))]
    pub results: Vec<BenchmarkFastqProfileResultRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqProfileMetrics {
    pub schema_version: String,
    pub sample_count: u64,
    pub file_count: u64,
    pub read_count: u64,
    pub base_count: u64,
    pub minimum_read_length: u64,
    pub maximum_read_length: u64,
    pub admission_mismatch_count: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqProfileProbe {
    pub schema_version: String,
    pub consumer_stage_id: String,
    pub input_contract: String,
    pub sample_count: u64,
    pub file_count: u64,
    pub read_count: u64,
    pub base_count: u64,
    pub minimum_read_length: u64,
    pub maximum_read_length: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqValidationResult {
    #[schemars(schema_with = "validate_reads_schema_version")]
    pub schema_version: String,
    #[schemars(schema_with = "validate_reads_stage_id")]
    pub stage_id: String,
    #[schemars(schema_with = "validate_reads_tool_id")]
    pub tool_id: String,
    #[schemars(schema_with = "twenty_schema")]
    pub sample_count: u64,
    #[schemars(schema_with = "thirty_schema")]
    pub file_count: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub expected_read_count: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub validated_read_count: u64,
    #[schemars(schema_with = "twenty_schema")]
    pub strict_pass_count: u64,
    #[schemars(schema_with = "positive_at_most_twenty_schema")]
    pub pair_sync_checked_count: u64,
    #[schemars(schema_with = "zero_schema")]
    pub pair_sync_failure_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqValidationMetrics {
    pub schema_version: String,
    pub sample_count: u64,
    pub file_count: u64,
    pub expected_read_count: u64,
    pub validated_read_count: u64,
    pub strict_pass_count: u64,
    pub pair_sync_checked_count: u64,
    pub pair_sync_failure_count: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFastqValidationProbe {
    pub schema_version: String,
    pub consumer_stage_id: String,
    pub input_contract: String,
    pub sample_count: u64,
    pub file_count: u64,
    pub validated_read_count: u64,
    pub strict_pass_count: u64,
    pub pair_sync_failure_count: u64,
}

#[must_use]
pub fn fastq_detect_adapters_result_schema() -> Schema {
    unique_results(schemars::schema_for!(BenchmarkFastqAdapterResult))
}

#[must_use]
pub fn fastq_profile_read_lengths_result_schema() -> Schema {
    unique_results(schemars::schema_for!(BenchmarkFastqReadLengthResult))
}

#[must_use]
pub fn fastq_profile_reads_result_schema() -> Schema {
    unique_results(schemars::schema_for!(BenchmarkFastqProfileResult))
}

#[must_use]
pub fn fastq_validate_reads_result_schema() -> Schema {
    schemars::schema_for!(BenchmarkFastqValidationResult)
}

macro_rules! const_string_schema {
    ($name:ident, $value:expr) => {
        fn $name(_: &mut SchemaGenerator) -> Schema {
            json_schema!({"type": "string", "const": $value})
        }
    };
}

const_string_schema!(detect_adapters_schema_version, FASTQ_DETECT_ADAPTERS_RESULT_SCHEMA_V1);
const_string_schema!(detect_adapters_stage_id, FASTQ_DETECT_ADAPTERS_STAGE_ID);
const_string_schema!(fastqc_tool_id, "fastqc");
const_string_schema!(
    profile_read_lengths_schema_version,
    FASTQ_PROFILE_READ_LENGTHS_RESULT_SCHEMA_V1
);
const_string_schema!(profile_read_lengths_stage_id, FASTQ_PROFILE_READ_LENGTHS_STAGE_ID);
const_string_schema!(profile_reads_schema_version, FASTQ_PROFILE_READS_RESULT_SCHEMA_V1);
const_string_schema!(profile_reads_stage_id, FASTQ_PROFILE_READS_STAGE_ID);
const_string_schema!(validate_reads_schema_version, FASTQ_VALIDATE_READS_RESULT_SCHEMA_V1);
const_string_schema!(validate_reads_stage_id, FASTQ_VALIDATE_READS_STAGE_ID);

fn read_length_tool_id(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "enum": ["seqfu", "seqkit"]})
}

fn profile_reads_tool_id(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "enum": ["seqfu", "seqkit", "seqkit_stats"]})
}

fn validate_reads_tool_id(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "enum": ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"]
    })
}

fn unique_results(mut schema: Schema) -> Schema {
    if let Some(results) = schema
        .get_mut("properties")
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|properties| properties.get_mut("results"))
        .and_then(serde_json::Value::as_object_mut)
    {
        results.insert("uniqueItems".to_string(), serde_json::Value::Bool(true));
    }
    schema
}

fn nonempty_string_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "minLength": 1})
}

fn unique_nonempty_strings_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "array",
        "items": {"type": "string", "minLength": 1},
        "uniqueItems": true
    })
}

fn sample_id_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]*$"})
}

fn paired_layout_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "enum": ["single_end", "paired_end"]})
}

fn profile_layout_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "enum": ["single_end", "paired_end", "merged"]})
}

fn file_count_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "integer", "minimum": 1, "maximum": 2})
}

fn positive_integer_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "integer", "minimum": 1})
}

fn nonnegative_number_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "number", "minimum": 0})
}

fn nonnegative_optional_number_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": ["number", "null"], "minimum": 0})
}

fn positive_number_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "number", "exclusiveMinimum": 0})
}

fn histogram_path_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "pattern": "^[A-Za-z0-9._/-]+[.]tsv$"})
}

fn sha256_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "pattern": "^[0-9a-f]{64}$"})
}

fn twenty_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "integer", "const": 20})
}

fn thirty_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "integer", "const": 30})
}

fn positive_at_most_twenty_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "integer", "minimum": 1, "maximum": 20})
}

fn zero_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "integer", "const": 0})
}
