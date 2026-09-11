use super::models::FilterStageSpec;
use bijux_dna_core::id_catalog::{FASTQ_FILTER, FASTQ_LOW_COMPLEXITY};
pub const DEFAULT_FASTQ_FILTER_STAGES_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/fastq/filter-stages-ready.json";
pub const FASTQ_FILTER_STAGES_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_filter_stages_ready.v1";
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const CHECKED_SURFACE_COUNT: usize = 7;
pub(super) const REQUIRED_REPORT_SECTION_ID: &str = "read_cleanup";
pub(super) const REQUIRED_SUMMARY_TABLE_ID: &str = "cleanup_retention";
pub(super) const FILTER_READS_TOOL_IDS: [&str; 4] = ["bbduk", "fastp", "prinseq", "seqkit"];
pub(super) const FILTER_LOW_COMPLEXITY_TOOL_IDS: [&str; 3] = ["bbduk", "fastp", "prinseq"];
pub(super) const FILTER_READS_REQUIRED_OUTPUT_IDS: [&str; 1] = ["filtered_reads_r1"];
pub(super) const FILTER_LOW_COMPLEXITY_REQUIRED_OUTPUT_IDS: [&str; 1] = ["filtered_fastq_r1"];
pub(super) const FILTER_READS_REQUIRED_METRIC_FIELDS: [&str; 9] = [
    "filtered_reads_r1",
    "reads_retained",
    "reads_dropped",
    "reads_removed",
    "reads_removed_by_n",
    "reads_removed_by_entropy",
    "reads_removed_low_complexity",
    "reads_removed_by_kmer",
    "reads_removed_by_length",
];
pub(super) const FILTER_LOW_COMPLEXITY_REQUIRED_METRIC_FIELDS: [&str; 5] = [
    "filtered_fastq_r1",
    "reads_retained",
    "reads_dropped",
    "reads_removed",
    "reads_removed_low_complexity",
];
pub(super) const FILTER_STAGE_SPECS: [FilterStageSpec; 2] = [
    FilterStageSpec {
        stage_id: FASTQ_FILTER,
        expected_tool_ids: &FILTER_READS_TOOL_IDS,
        required_output_ids: &FILTER_READS_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "report_json",
        required_metric_fields: &FILTER_READS_REQUIRED_METRIC_FIELDS,
    },
    FilterStageSpec {
        stage_id: FASTQ_LOW_COMPLEXITY,
        expected_tool_ids: &FILTER_LOW_COMPLEXITY_TOOL_IDS,
        required_output_ids: &FILTER_LOW_COMPLEXITY_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "filter_report_json",
        required_metric_fields: &FILTER_LOW_COMPLEXITY_REQUIRED_METRIC_FIELDS,
    },
];
