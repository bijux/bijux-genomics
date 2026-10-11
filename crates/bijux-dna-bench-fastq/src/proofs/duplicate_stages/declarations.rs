use super::models::DuplicateStageSpec;
use bijux_dna_core::id_catalog::{FASTQ_DEDUPLICATE, FASTQ_DETECT_DUPLICATES_PREMERGE};
pub const DEFAULT_FASTQ_DUPLICATE_STAGES_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/fastq/duplicate-stages-ready.json";
pub const FASTQ_DUPLICATE_STAGES_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_duplicate_stages_ready.v1";
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const CHECKED_SURFACE_COUNT: usize = 7;
pub(super) const DETECT_DUPLICATES_PREMERGE_TOOL_IDS: [&str; 1] = ["bijux_dna"];
pub(super) const REMOVE_DUPLICATES_TOOL_IDS: [&str; 2] = ["clumpify", "fastuniq"];
pub(super) const DETECT_DUPLICATES_PREMERGE_REQUIRED_OUTPUT_IDS: [&str; 1] =
    ["duplicate_signal_report"];
pub(super) const REMOVE_DUPLICATES_REQUIRED_OUTPUT_IDS: [&str; 1] = ["dedup_reads_r1"];
pub(super) const DETECT_DUPLICATES_PREMERGE_REQUIRED_METRIC_FIELDS: [&str; 4] =
    ["duplicate_count", "duplicate_fraction", "inspected_pair_count", "report_json"];
pub(super) const REMOVE_DUPLICATES_REQUIRED_METRIC_FIELDS: [&str; 5] =
    ["unique_reads", "output_reads", "duplicates_removed", "dedup_rate", "report_json"];
pub(super) const DUPLICATE_STAGE_SPECS: [DuplicateStageSpec; 2] = [
    DuplicateStageSpec {
        stage_id: FASTQ_DETECT_DUPLICATES_PREMERGE,
        expected_tool_ids: &DETECT_DUPLICATES_PREMERGE_TOOL_IDS,
        required_output_ids: &DETECT_DUPLICATES_PREMERGE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "duplicate_signal_report",
        required_metric_fields: &DETECT_DUPLICATES_PREMERGE_REQUIRED_METRIC_FIELDS,
        required_report_section_id: "quality_profiling",
        required_summary_table_id: "premerge_complexity",
    },
    DuplicateStageSpec {
        stage_id: FASTQ_DEDUPLICATE,
        expected_tool_ids: &REMOVE_DUPLICATES_TOOL_IDS,
        required_output_ids: &REMOVE_DUPLICATES_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "report_json",
        required_metric_fields: &REMOVE_DUPLICATES_REQUIRED_METRIC_FIELDS,
        required_report_section_id: "read_cleanup",
        required_summary_table_id: "cleanup_retention",
    },
];
