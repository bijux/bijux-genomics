use bijux_dna_core::id_catalog::FASTQ_VALIDATE_READS;

pub const DEFAULT_FASTQ_VALIDATE_READS_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/fastq/validate-reads-ready.json";
pub const FASTQ_VALIDATE_READS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_validate_reads_ready.v1";
pub(super) const FASTQ_VALIDATE_READS_STAGE_ID: &str = FASTQ_VALIDATE_READS;
pub(super) const FASTQ_VALIDATE_READS_SMOKE_PATH: &str =
    "artifacts/runs/bench/local-smoke/fastq.validate_reads/report.json";
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const VALIDATION_STATUS_FIELD_ID: &str = "strict_pass";
pub(super) const FAILURE_REASON_FIELD_ID: &str = "failure_class";
pub(super) const EXPECTED_SMOKE_CASE_COUNT: usize = 2;
pub(super) const REQUIRED_TOOL_IDS: [&str; 5] =
    ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"];
pub(super) const REQUIRED_SMOKE_SAMPLE_IDS: [&str; 2] = ["toy-pe", "toy-se"];
pub(super) const REQUIRED_SMOKE_LAYOUTS: [&str; 2] = ["paired_end", "single_end"];
pub(super) const REQUIRED_SHARED_METRIC_FIELDS: [&str; 1] = ["format_validation_pass_rate"];
pub(super) const REQUIRED_VALIDATION_REPORT_FIELDS: [&str; 10] = [
    "validated_reads_r1",
    "validated_reads_r2",
    "validated_pairs",
    "status_r1",
    "status_r2",
    "pair_sync_checked",
    "pair_sync_pass",
    "pair_count_match",
    "failure_class",
    "strict_pass",
];
pub(super) const REQUIRED_VALIDATED_READS_MANIFEST_FIELDS: [&str; 5] = [
    "paired_mode",
    "validated_stream_ids",
    "pair_sync_checked",
    "pair_sync_pass",
    "validated_pairs",
];
