use bijux_dna_core::id_catalog::{BAM_GC_BIAS, BAM_INSERT_SIZE};

pub(in crate::readiness) const BAM_INSERT_SIZE_GC_BIAS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_insert_size_gc_bias_ready.v1";
pub(in crate::readiness) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(in crate::readiness) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(in crate::readiness) const CHECKED_SURFACE_COUNT: usize = 8;
pub(in crate::readiness) const REQUIRED_REPORT_SECTION_ID: &str = "coverage_quality";
pub(in crate::readiness) const REQUIRED_SUMMARY_TABLE_ID: &str = "coverage_bias_qc";
pub(in crate::readiness) const EXPECTED_SCHEMA_REQUIRED_KEYS: [&str; 9] = [
    "schema_version",
    "stage_id",
    "tool_id",
    "tool_version",
    "execution",
    "outputs_count",
    "artifacts",
    "contracts",
    "normalized_keys",
];
#[derive(Debug, Clone, Copy)]
pub(in crate::readiness) struct BamStageSpec {
    pub(in crate::readiness) stage_id: &'static str,
    pub(in crate::readiness) tool_id: &'static str,
    pub(in crate::readiness) required_output_ids: &'static [&'static str],
    pub(in crate::readiness) normalized_metrics_output_id: &'static str,
    pub(in crate::readiness) schema_extension_id: &'static str,
    pub(in crate::readiness) required_local_smoke_fields: &'static [&'static str],
}
pub(in crate::readiness) const INSERT_SIZE_REQUIRED_OUTPUT_IDS: [&str; 4] =
    ["insert_size_report", "insert_size_histogram", "summary", "stage_metrics"];
pub(in crate::readiness) const GC_BIAS_REQUIRED_OUTPUT_IDS: [&str; 4] =
    ["gc_bias_report", "gc_bias_plot", "summary", "stage_metrics"];
pub(in crate::readiness) const INSERT_SIZE_REQUIRED_LOCAL_SMOKE_FIELDS: [&str; 6] = [
    "read_pairs",
    "median_insert_size",
    "mean_insert_size",
    "standard_deviation",
    "min_insert_size",
    "max_insert_size",
];
pub(in crate::readiness) const GC_BIAS_REQUIRED_LOCAL_SMOKE_FIELDS: [&str; 7] = [
    "gc_bin",
    "normalized_coverage",
    "windows",
    "read_starts",
    "gc_bias_score",
    "at_dropout",
    "gc_dropout",
];
pub(in crate::readiness) const BAM_STAGE_SPECS: [BamStageSpec; 2] = [
    BamStageSpec {
        stage_id: BAM_GC_BIAS,
        tool_id: "picard",
        required_output_ids: &GC_BIAS_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "gc_bias_report",
        schema_extension_id: "bam_gc_bias_normalized_v1",
        required_local_smoke_fields: &GC_BIAS_REQUIRED_LOCAL_SMOKE_FIELDS,
    },
    BamStageSpec {
        stage_id: BAM_INSERT_SIZE,
        tool_id: "picard",
        required_output_ids: &INSERT_SIZE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "insert_size_report",
        schema_extension_id: "bam_insert_size_normalized_v1",
        required_local_smoke_fields: &INSERT_SIZE_REQUIRED_LOCAL_SMOKE_FIELDS,
    },
];
