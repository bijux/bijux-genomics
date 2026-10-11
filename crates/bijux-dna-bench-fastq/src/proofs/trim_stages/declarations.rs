use super::models::TrimStageSpec;
use bijux_dna_core::id_catalog::{FASTQ_TRIM, FASTQ_TRIM_POLYG_TAILS, FASTQ_TRIM_TERMINAL_DAMAGE};
pub const DEFAULT_FASTQ_TRIM_STAGES_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/fastq/trim-stages-ready.json";
pub const FASTQ_TRIM_STAGES_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_trim_stages_ready.v1";
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const CHECKED_SURFACE_COUNT: usize = 7;
pub(super) const REQUIRED_REPORT_SECTION_ID: &str = "read_cleanup";
pub(super) const REQUIRED_SUMMARY_TABLE_ID: &str = "cleanup_retention";
pub(super) const TRIM_READS_TOOL_IDS: [&str; 14] = [
    "adapterremoval",
    "alientrimmer",
    "atropos",
    "bbduk",
    "cutadapt",
    "fastp",
    "fastx_clipper",
    "leehom",
    "prinseq",
    "seqkit",
    "seqprep",
    "skewer",
    "trim_galore",
    "trimmomatic",
];
pub(super) const TRIM_TERMINAL_DAMAGE_TOOL_IDS: [&str; 3] =
    ["adapterremoval", "cutadapt", "seqkit"];
pub(super) const TRIM_POLYG_TAILS_TOOL_IDS: [&str; 2] = ["bbduk", "fastp"];
pub(super) const TRIM_READS_REQUIRED_METRIC_FIELDS: [&str; 3] =
    ["reads_retained", "reads_dropped", "bases_removed"];
pub(super) const TRIM_TERMINAL_DAMAGE_REQUIRED_METRIC_FIELDS: [&str; 4] =
    ["reads_retained", "bases_removed", "trim_5p_bases", "trim_3p_bases"];
pub(super) const TRIM_POLYG_TAILS_REQUIRED_METRIC_FIELDS: [&str; 5] = [
    "reads_retained",
    "reads_dropped",
    "bases_removed",
    "trimmed_tail_count",
    "bases_trimmed_polyg",
];
pub(super) const TRIM_STAGE_SPECS: [TrimStageSpec; 3] = [
    TrimStageSpec {
        stage_id: FASTQ_TRIM,
        expected_tool_ids: &TRIM_READS_TOOL_IDS,
        required_metric_fields: &TRIM_READS_REQUIRED_METRIC_FIELDS,
    },
    TrimStageSpec {
        stage_id: FASTQ_TRIM_TERMINAL_DAMAGE,
        expected_tool_ids: &TRIM_TERMINAL_DAMAGE_TOOL_IDS,
        required_metric_fields: &TRIM_TERMINAL_DAMAGE_REQUIRED_METRIC_FIELDS,
    },
    TrimStageSpec {
        stage_id: FASTQ_TRIM_POLYG_TAILS,
        expected_tool_ids: &TRIM_POLYG_TAILS_TOOL_IDS,
        required_metric_fields: &TRIM_POLYG_TAILS_REQUIRED_METRIC_FIELDS,
    },
];
