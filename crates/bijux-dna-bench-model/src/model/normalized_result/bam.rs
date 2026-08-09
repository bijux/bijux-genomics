//! Canonical normalized BAM result and measurement records.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

pub const BAM_ALIGN_RESULT_SCHEMA_V1: &str = "bijux.benchmark.bam.align.bundle.v1";
pub const BAM_ALIGN_METRICS_SCHEMA_V1: &str = "bijux.benchmark.bam.align.metrics.v1";
pub const BAM_ALIGN_STAGE_ID: &str = "bam.align";
pub const BAM_COVERAGE_RESULT_SCHEMA_V1: &str = "bijux.benchmark.bam.coverage.bundle.v1";
pub const BAM_COVERAGE_METRICS_SCHEMA_V1: &str = "bijux.benchmark.bam.coverage.metrics.v1";
pub const BAM_COVERAGE_STAGE_ID: &str = "bam.coverage";
pub const BAM_DUPLICATION_METRICS_RESULT_SCHEMA_V1: &str =
    "bijux.benchmark.bam.duplication_metrics.bundle.v1";
pub const BAM_DUPLICATION_METRICS_MEASUREMENTS_SCHEMA_V1: &str =
    "bijux.benchmark.bam.duplication_metrics.metrics.v1";
pub const BAM_DUPLICATION_METRICS_STAGE_ID: &str = "bam.duplication_metrics";
pub const BAM_ENDOGENOUS_CONTENT_RESULT_SCHEMA_V1: &str =
    "bijux.benchmark.bam.endogenous_content.bundle.v1";
pub const BAM_ENDOGENOUS_CONTENT_METRICS_SCHEMA_V1: &str =
    "bijux.benchmark.bam.endogenous_content.metrics.v1";
pub const BAM_ENDOGENOUS_CONTENT_STAGE_ID: &str = "bam.endogenous_content";
pub const BAM_GENOTYPING_RESULT_SCHEMA_V1: &str = "bijux.benchmark.bam.genotyping.bundle.v1";
pub const BAM_GENOTYPING_METRICS_SCHEMA_V1: &str = "bijux.benchmark.bam.genotyping.metrics.v1";
pub const BAM_GENOTYPING_STAGE_ID: &str = "bam.genotyping";
pub const BAM_GC_BIAS_RESULT_SCHEMA_V1: &str = "bijux.benchmark.bam.gc_bias.bundle.v1";
pub const BAM_GC_BIAS_METRICS_SCHEMA_V1: &str = "bijux.benchmark.bam.gc_bias.metrics.v1";
pub const BAM_GC_BIAS_STAGE_ID: &str = "bam.gc_bias";
pub const BAM_INSERT_SIZE_RESULT_SCHEMA_V1: &str = "bijux.benchmark.bam.insert_size.bundle.v1";
pub const BAM_INSERT_SIZE_METRICS_SCHEMA_V1: &str = "bijux.benchmark.bam.insert_size.metrics.v1";
pub const BAM_INSERT_SIZE_STAGE_ID: &str = "bam.insert_size";
pub const BAM_MAPPING_SUMMARY_RESULT_SCHEMA_V1: &str =
    "bijux.benchmark.bam.mapping_summary.bundle.v1";
pub const BAM_MAPPING_SUMMARY_METRICS_SCHEMA_V1: &str =
    "bijux.benchmark.bam.mapping_summary.metrics.v1";
pub const BAM_MAPPING_SUMMARY_STAGE_ID: &str = "bam.mapping_summary";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamAlignValues {
    #[schemars(schema_with = "positive_integer_schema")]
    pub total_reads: u64,
    pub mapped_reads: u64,
    #[schemars(schema_with = "fraction_schema")]
    pub alignment_rate: f64,
    pub expected_mapped_reads: u64,
    pub exact_placement_reads: u64,
    #[schemars(schema_with = "fraction_schema")]
    pub placement_precision: f64,
    #[schemars(schema_with = "fraction_schema")]
    pub placement_recall: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamAlignResult {
    #[serde(deserialize_with = "deserialize_align_schema_version")]
    #[schemars(schema_with = "align_schema_version")]
    pub schema_version: String,
    #[serde(deserialize_with = "deserialize_align_stage_id")]
    #[schemars(schema_with = "align_stage_id")]
    pub stage_id: String,
    #[serde(deserialize_with = "deserialize_align_tool_id")]
    #[schemars(schema_with = "align_tool_id")]
    pub tool_id: String,
    #[serde(deserialize_with = "deserialize_sample_id")]
    #[schemars(schema_with = "sample_id_schema")]
    pub sample_id: String,
    #[serde(flatten)]
    pub values: BenchmarkBamAlignValues,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamAlignMetrics {
    #[serde(deserialize_with = "deserialize_align_metrics_schema_version")]
    #[schemars(schema_with = "align_metrics_schema_version")]
    pub schema_version: String,
    #[serde(flatten)]
    pub values: BenchmarkBamAlignValues,
    pub fastq_input_bytes: u64,
    pub reference_input_bytes: u64,
    pub reference_index_input_bytes: u64,
    pub truth_input_bytes: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamCoverageValues {
    #[schemars(schema_with = "positive_integer_schema")]
    pub reference_bases: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub covered_bases: u64,
    #[schemars(schema_with = "positive_fraction_schema")]
    pub coverage_fraction: f64,
    #[schemars(schema_with = "positive_number_schema")]
    pub mean_depth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamCoverageResult {
    #[serde(deserialize_with = "deserialize_coverage_schema_version")]
    #[schemars(schema_with = "coverage_schema_version")]
    pub schema_version: String,
    #[serde(deserialize_with = "deserialize_coverage_stage_id")]
    #[schemars(schema_with = "coverage_stage_id")]
    pub stage_id: String,
    #[serde(deserialize_with = "deserialize_coverage_tool_id")]
    #[schemars(schema_with = "coverage_tool_id")]
    pub tool_id: String,
    #[serde(deserialize_with = "deserialize_sample_id")]
    #[schemars(schema_with = "sample_id_schema")]
    pub sample_id: String,
    #[serde(flatten)]
    pub values: BenchmarkBamCoverageValues,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamCoverageMetrics {
    #[serde(deserialize_with = "deserialize_coverage_metrics_schema_version")]
    #[schemars(schema_with = "coverage_metrics_schema_version")]
    pub schema_version: String,
    #[serde(flatten)]
    pub values: BenchmarkBamCoverageValues,
    pub bam_input_bytes: u64,
    pub bam_index_input_bytes: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamDuplicationMetricsValues {
    #[schemars(schema_with = "positive_integer_schema")]
    pub examined_reads: u64,
    pub duplicate_reads: u64,
    #[schemars(schema_with = "fraction_schema")]
    pub duplication_fraction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamDuplicationMetricsResult {
    #[serde(deserialize_with = "deserialize_duplication_schema_version")]
    #[schemars(schema_with = "duplication_schema_version")]
    pub schema_version: String,
    #[serde(deserialize_with = "deserialize_duplication_stage_id")]
    #[schemars(schema_with = "duplication_stage_id")]
    pub stage_id: String,
    #[serde(deserialize_with = "deserialize_duplication_tool_id")]
    #[schemars(schema_with = "duplication_tool_id")]
    pub tool_id: String,
    #[serde(deserialize_with = "deserialize_sample_id")]
    #[schemars(schema_with = "sample_id_schema")]
    pub sample_id: String,
    #[serde(flatten)]
    pub values: BenchmarkBamDuplicationMetricsValues,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamDuplicationMetricsMeasurements {
    #[serde(deserialize_with = "deserialize_duplication_measurements_schema_version")]
    #[schemars(schema_with = "duplication_measurements_schema_version")]
    pub schema_version: String,
    #[serde(flatten)]
    pub values: BenchmarkBamDuplicationMetricsValues,
    pub bam_input_bytes: u64,
    pub bam_index_input_bytes: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamEndogenousContentValues {
    #[schemars(schema_with = "positive_integer_schema")]
    pub total_primary_reads: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub mapped_primary_reads: u64,
    #[schemars(schema_with = "positive_fraction_schema")]
    pub mapped_fraction: f64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub mapped_nonduplicate_primary_reads: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamEndogenousContentResult {
    #[serde(deserialize_with = "deserialize_endogenous_schema_version")]
    #[schemars(schema_with = "endogenous_schema_version")]
    pub schema_version: String,
    #[serde(deserialize_with = "deserialize_endogenous_stage_id")]
    #[schemars(schema_with = "endogenous_stage_id")]
    pub stage_id: String,
    #[serde(deserialize_with = "deserialize_samtools_tool_id")]
    #[schemars(schema_with = "samtools_tool_id")]
    pub tool_id: String,
    #[serde(deserialize_with = "deserialize_sample_id")]
    #[schemars(schema_with = "sample_id_schema")]
    pub sample_id: String,
    #[serde(flatten)]
    pub values: BenchmarkBamEndogenousContentValues,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamEndogenousContentMetrics {
    #[serde(deserialize_with = "deserialize_endogenous_metrics_schema_version")]
    #[schemars(schema_with = "endogenous_metrics_schema_version")]
    pub schema_version: String,
    #[serde(flatten)]
    pub values: BenchmarkBamEndogenousContentValues,
    pub bam_input_bytes: u64,
    pub bam_index_input_bytes: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamGenotypingResult {
    #[serde(deserialize_with = "deserialize_genotyping_schema_version")]
    #[schemars(schema_with = "genotyping_schema_version")]
    pub schema_version: String,
    #[serde(deserialize_with = "deserialize_genotyping_stage_id")]
    #[schemars(schema_with = "genotyping_stage_id")]
    pub stage_id: String,
    #[serde(deserialize_with = "deserialize_bcftools_tool_id")]
    #[schemars(schema_with = "bcftools_tool_id")]
    pub tool_id: String,
    #[serde(deserialize_with = "deserialize_sample_id")]
    #[schemars(schema_with = "sample_id_schema")]
    pub sample_id: String,
    #[schemars(schema_with = "positive_integer_schema")]
    pub total_records: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub covered_sites: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub called_genotypes: u64,
    pub alternate_genotypes: u64,
    #[schemars(schema_with = "positive_number_schema")]
    pub mean_depth_at_covered_sites: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamGenotypingMetrics {
    #[serde(deserialize_with = "deserialize_genotyping_metrics_schema_version")]
    #[schemars(schema_with = "genotyping_metrics_schema_version")]
    pub schema_version: String,
    pub total_records: u64,
    pub covered_sites: u64,
    pub called_genotypes: u64,
    pub alternate_genotypes: u64,
    pub mean_depth_at_covered_sites: f64,
    pub bam_input_bytes: u64,
    pub reference_input_bytes: u64,
    pub database_input_bytes: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamGcBiasValues {
    #[schemars(schema_with = "fraction_schema")]
    pub gc_bias_score: f64,
    #[schemars(schema_with = "percentage_schema")]
    pub at_dropout: f64,
    #[schemars(schema_with = "percentage_schema")]
    pub gc_dropout: f64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub total_clusters: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub aligned_reads: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub windows: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub read_starts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamGcBiasResult {
    #[serde(deserialize_with = "deserialize_gc_bias_schema_version")]
    #[schemars(schema_with = "gc_bias_schema_version")]
    pub schema_version: String,
    #[serde(deserialize_with = "deserialize_gc_bias_stage_id")]
    #[schemars(schema_with = "gc_bias_stage_id")]
    pub stage_id: String,
    #[serde(deserialize_with = "deserialize_picard_tool_id")]
    #[schemars(schema_with = "picard_tool_id")]
    pub tool_id: String,
    #[serde(deserialize_with = "deserialize_sample_id")]
    #[schemars(schema_with = "sample_id_schema")]
    pub sample_id: String,
    #[serde(flatten)]
    pub values: BenchmarkBamGcBiasValues,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamGcBiasMetrics {
    #[serde(deserialize_with = "deserialize_gc_bias_metrics_schema_version")]
    #[schemars(schema_with = "gc_bias_metrics_schema_version")]
    pub schema_version: String,
    #[serde(flatten)]
    pub values: BenchmarkBamGcBiasValues,
    pub bam_input_bytes: u64,
    pub bam_index_input_bytes: u64,
    pub reference_input_bytes: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamInsertSizeValues {
    #[schemars(schema_with = "positive_number_schema")]
    pub median_insert_size: f64,
    #[schemars(schema_with = "positive_number_schema")]
    pub mean_insert_size: f64,
    #[schemars(schema_with = "nonnegative_number_schema")]
    pub standard_deviation: f64,
    #[schemars(schema_with = "nonnegative_number_schema")]
    pub median_absolute_deviation: f64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub minimum_insert_size: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub maximum_insert_size: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub read_pairs: u64,
    #[schemars(schema_with = "fraction_schema")]
    pub pair_orientation_fr_fraction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamInsertSizeResult {
    #[serde(deserialize_with = "deserialize_insert_size_schema_version")]
    #[schemars(schema_with = "insert_size_schema_version")]
    pub schema_version: String,
    #[serde(deserialize_with = "deserialize_insert_size_stage_id")]
    #[schemars(schema_with = "insert_size_stage_id")]
    pub stage_id: String,
    #[serde(deserialize_with = "deserialize_picard_tool_id")]
    #[schemars(schema_with = "picard_tool_id")]
    pub tool_id: String,
    #[serde(deserialize_with = "deserialize_sample_id")]
    #[schemars(schema_with = "sample_id_schema")]
    pub sample_id: String,
    #[serde(flatten)]
    pub values: BenchmarkBamInsertSizeValues,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamInsertSizeMetrics {
    #[serde(deserialize_with = "deserialize_insert_size_metrics_schema_version")]
    #[schemars(schema_with = "insert_size_metrics_schema_version")]
    pub schema_version: String,
    #[serde(flatten)]
    pub values: BenchmarkBamInsertSizeValues,
    pub bam_input_bytes: u64,
    pub bam_index_input_bytes: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamMappingSummaryResult {
    #[serde(deserialize_with = "deserialize_mapping_schema_version")]
    #[schemars(schema_with = "mapping_schema_version")]
    pub schema_version: String,
    #[serde(deserialize_with = "deserialize_mapping_stage_id")]
    #[schemars(schema_with = "mapping_stage_id")]
    pub stage_id: String,
    #[serde(deserialize_with = "deserialize_mapping_tool_id")]
    #[schemars(schema_with = "mapping_tool_id")]
    pub tool_id: String,
    #[serde(deserialize_with = "deserialize_sample_id")]
    #[schemars(schema_with = "sample_id_schema")]
    pub sample_id: String,
    #[schemars(schema_with = "positive_integer_schema")]
    pub primary_reads: u64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub mapped_primary_reads: u64,
    #[schemars(schema_with = "positive_fraction_schema")]
    pub mapped_primary_fraction: f64,
    #[schemars(schema_with = "positive_integer_schema")]
    pub paired_primary_reads: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBamMappingSummaryMetrics {
    #[serde(deserialize_with = "deserialize_mapping_metrics_schema_version")]
    #[schemars(schema_with = "mapping_metrics_schema_version")]
    pub schema_version: String,
    pub primary_reads: u64,
    pub mapped_primary_reads: u64,
    pub mapped_primary_fraction: f64,
    pub paired_primary_reads: u64,
    pub bam_input_bytes: u64,
    pub bam_index_input_bytes: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[must_use]
pub fn bam_align_result_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamAlignResult)
}

#[must_use]
pub fn bam_align_metrics_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamAlignMetrics)
}

#[must_use]
pub fn bam_coverage_result_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamCoverageResult)
}

#[must_use]
pub fn bam_coverage_metrics_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamCoverageMetrics)
}

#[must_use]
pub fn bam_duplication_metrics_result_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamDuplicationMetricsResult)
}

#[must_use]
pub fn bam_duplication_metrics_measurements_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamDuplicationMetricsMeasurements)
}

#[must_use]
pub fn bam_endogenous_content_result_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamEndogenousContentResult)
}

#[must_use]
pub fn bam_endogenous_content_metrics_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamEndogenousContentMetrics)
}

#[must_use]
pub fn bam_genotyping_result_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamGenotypingResult)
}

#[must_use]
pub fn bam_genotyping_metrics_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamGenotypingMetrics)
}

#[must_use]
pub fn bam_gc_bias_result_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamGcBiasResult)
}

#[must_use]
pub fn bam_gc_bias_metrics_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamGcBiasMetrics)
}

#[must_use]
pub fn bam_insert_size_result_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamInsertSizeResult)
}

#[must_use]
pub fn bam_insert_size_metrics_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamInsertSizeMetrics)
}

#[must_use]
pub fn bam_mapping_summary_result_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamMappingSummaryResult)
}

#[must_use]
pub fn bam_mapping_summary_metrics_schema() -> Schema {
    schemars::schema_for!(BenchmarkBamMappingSummaryMetrics)
}

macro_rules! const_string_schema {
    ($name:ident, $value:expr) => {
        fn $name(_: &mut SchemaGenerator) -> Schema {
            json_schema!({"type": "string", "const": $value})
        }
    };
}

const_string_schema!(coverage_schema_version, BAM_COVERAGE_RESULT_SCHEMA_V1);
const_string_schema!(align_schema_version, BAM_ALIGN_RESULT_SCHEMA_V1);
const_string_schema!(align_metrics_schema_version, BAM_ALIGN_METRICS_SCHEMA_V1);
const_string_schema!(align_stage_id, BAM_ALIGN_STAGE_ID);
const_string_schema!(coverage_metrics_schema_version, BAM_COVERAGE_METRICS_SCHEMA_V1);
const_string_schema!(coverage_stage_id, BAM_COVERAGE_STAGE_ID);
const_string_schema!(duplication_schema_version, BAM_DUPLICATION_METRICS_RESULT_SCHEMA_V1);
const_string_schema!(
    duplication_measurements_schema_version,
    BAM_DUPLICATION_METRICS_MEASUREMENTS_SCHEMA_V1
);
const_string_schema!(duplication_stage_id, BAM_DUPLICATION_METRICS_STAGE_ID);
const_string_schema!(endogenous_schema_version, BAM_ENDOGENOUS_CONTENT_RESULT_SCHEMA_V1);
const_string_schema!(endogenous_metrics_schema_version, BAM_ENDOGENOUS_CONTENT_METRICS_SCHEMA_V1);
const_string_schema!(endogenous_stage_id, BAM_ENDOGENOUS_CONTENT_STAGE_ID);
const_string_schema!(samtools_tool_id, "samtools");
const_string_schema!(genotyping_schema_version, BAM_GENOTYPING_RESULT_SCHEMA_V1);
const_string_schema!(genotyping_metrics_schema_version, BAM_GENOTYPING_METRICS_SCHEMA_V1);
const_string_schema!(genotyping_stage_id, BAM_GENOTYPING_STAGE_ID);
const_string_schema!(bcftools_tool_id, "bcftools");
const_string_schema!(gc_bias_schema_version, BAM_GC_BIAS_RESULT_SCHEMA_V1);
const_string_schema!(gc_bias_metrics_schema_version, BAM_GC_BIAS_METRICS_SCHEMA_V1);
const_string_schema!(gc_bias_stage_id, BAM_GC_BIAS_STAGE_ID);
const_string_schema!(insert_size_schema_version, BAM_INSERT_SIZE_RESULT_SCHEMA_V1);
const_string_schema!(insert_size_metrics_schema_version, BAM_INSERT_SIZE_METRICS_SCHEMA_V1);
const_string_schema!(insert_size_stage_id, BAM_INSERT_SIZE_STAGE_ID);
const_string_schema!(picard_tool_id, "picard");
const_string_schema!(mapping_schema_version, BAM_MAPPING_SUMMARY_RESULT_SCHEMA_V1);
const_string_schema!(mapping_metrics_schema_version, BAM_MAPPING_SUMMARY_METRICS_SCHEMA_V1);
const_string_schema!(mapping_stage_id, BAM_MAPPING_SUMMARY_STAGE_ID);

fn coverage_tool_id(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "enum": ["bedtools", "mosdepth", "samtools"]})
}

fn align_tool_id(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "enum": ["bowtie2", "bwa"]})
}

fn duplication_tool_id(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "enum": ["picard", "samtools"]})
}

fn mapping_tool_id(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "enum": ["picard", "samtools"]})
}

fn sample_id_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]*$"})
}

fn positive_integer_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "integer", "minimum": 1})
}

fn fraction_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "number", "minimum": 0, "maximum": 1})
}

fn percentage_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "number", "minimum": 0, "maximum": 100})
}

fn positive_fraction_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "number", "exclusiveMinimum": 0, "maximum": 1})
}

fn positive_number_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "number", "exclusiveMinimum": 0})
}

fn nonnegative_number_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "number", "minimum": 0})
}

macro_rules! exact_deserializer {
    ($name:ident, $expected:expr) => {
        fn $name<'de, D>(deserializer: D) -> Result<String, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserialize_one_of(deserializer, &[$expected])
        }
    };
}

exact_deserializer!(deserialize_coverage_schema_version, BAM_COVERAGE_RESULT_SCHEMA_V1);
exact_deserializer!(deserialize_align_schema_version, BAM_ALIGN_RESULT_SCHEMA_V1);
exact_deserializer!(deserialize_align_metrics_schema_version, BAM_ALIGN_METRICS_SCHEMA_V1);
exact_deserializer!(deserialize_align_stage_id, BAM_ALIGN_STAGE_ID);
exact_deserializer!(deserialize_coverage_metrics_schema_version, BAM_COVERAGE_METRICS_SCHEMA_V1);
exact_deserializer!(deserialize_coverage_stage_id, BAM_COVERAGE_STAGE_ID);
exact_deserializer!(
    deserialize_duplication_schema_version,
    BAM_DUPLICATION_METRICS_RESULT_SCHEMA_V1
);
exact_deserializer!(
    deserialize_duplication_measurements_schema_version,
    BAM_DUPLICATION_METRICS_MEASUREMENTS_SCHEMA_V1
);
exact_deserializer!(deserialize_duplication_stage_id, BAM_DUPLICATION_METRICS_STAGE_ID);
exact_deserializer!(deserialize_endogenous_schema_version, BAM_ENDOGENOUS_CONTENT_RESULT_SCHEMA_V1);
exact_deserializer!(
    deserialize_endogenous_metrics_schema_version,
    BAM_ENDOGENOUS_CONTENT_METRICS_SCHEMA_V1
);
exact_deserializer!(deserialize_endogenous_stage_id, BAM_ENDOGENOUS_CONTENT_STAGE_ID);
exact_deserializer!(deserialize_samtools_tool_id, "samtools");
exact_deserializer!(deserialize_genotyping_schema_version, BAM_GENOTYPING_RESULT_SCHEMA_V1);
exact_deserializer!(
    deserialize_genotyping_metrics_schema_version,
    BAM_GENOTYPING_METRICS_SCHEMA_V1
);
exact_deserializer!(deserialize_genotyping_stage_id, BAM_GENOTYPING_STAGE_ID);
exact_deserializer!(deserialize_bcftools_tool_id, "bcftools");
exact_deserializer!(deserialize_gc_bias_schema_version, BAM_GC_BIAS_RESULT_SCHEMA_V1);
exact_deserializer!(deserialize_gc_bias_metrics_schema_version, BAM_GC_BIAS_METRICS_SCHEMA_V1);
exact_deserializer!(deserialize_gc_bias_stage_id, BAM_GC_BIAS_STAGE_ID);
exact_deserializer!(deserialize_insert_size_schema_version, BAM_INSERT_SIZE_RESULT_SCHEMA_V1);
exact_deserializer!(
    deserialize_insert_size_metrics_schema_version,
    BAM_INSERT_SIZE_METRICS_SCHEMA_V1
);
exact_deserializer!(deserialize_insert_size_stage_id, BAM_INSERT_SIZE_STAGE_ID);
exact_deserializer!(deserialize_picard_tool_id, "picard");
exact_deserializer!(deserialize_mapping_schema_version, BAM_MAPPING_SUMMARY_RESULT_SCHEMA_V1);
exact_deserializer!(
    deserialize_mapping_metrics_schema_version,
    BAM_MAPPING_SUMMARY_METRICS_SCHEMA_V1
);
exact_deserializer!(deserialize_mapping_stage_id, BAM_MAPPING_SUMMARY_STAGE_ID);

fn deserialize_coverage_tool_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_one_of(deserializer, &["bedtools", "mosdepth", "samtools"])
}

fn deserialize_align_tool_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_one_of(deserializer, &["bowtie2", "bwa"])
}

fn deserialize_duplication_tool_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_one_of(deserializer, &["picard", "samtools"])
}

fn deserialize_mapping_tool_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_one_of(deserializer, &["picard", "samtools"])
}

fn deserialize_one_of<'de, D>(deserializer: D, allowed: &[&str]) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if allowed.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported normalized BAM identity: {value}")))
    }
}

fn deserialize_sample_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    let normalized = !value.is_empty()
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric() || (index > 0 && matches!(byte, b'.' | b'_' | b'-'))
        });
    if normalized {
        Ok(value)
    } else {
        Err(D::Error::custom("normalized BAM sample identity is malformed"))
    }
}
