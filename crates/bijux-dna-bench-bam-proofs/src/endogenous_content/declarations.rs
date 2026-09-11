use bijux_dna_core::id_catalog::BAM_ENDOGENOUS_CONTENT;
pub const DEFAULT_BAM_ENDOGENOUS_CONTENT_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/stages/bam.endogenous_content.complete.json";

pub(super) const BAM_ENDOGENOUS_CONTENT_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_endogenous_content_complete.v1";

pub(super) const EXPECTED_STAGE_ID: &str = BAM_ENDOGENOUS_CONTENT;

pub(super) const EXPECTED_TOOL_ID: &str = "samtools";

pub(super) const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.endogenous_content.v1";

pub(super) const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.endogenous_content.local_smoke.metrics.v1";

pub(super) const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bam.endogenous_content.local_smoke.report.v1";

pub(super) const EXPECTED_SAMPLE_ID: &str = "human_like_endogenous_partial_mapping";

pub(super) const EXPECTED_METHOD: &str = "mapped_fraction_from_flagstat";

pub(super) const EXPECTED_HOST_REFERENCE_SCOPE: &str = "human_host";

pub(super) const EXPECTED_MAPPED_READS: u64 = 3;

pub(super) const EXPECTED_ENDOGENOUS_READS: u64 = 3;

pub(super) const EXPECTED_TOTAL_READS: u64 = 5;

pub(super) const EXPECTED_ENDOGENOUS_FRACTION: f64 = 0.6;

pub(super) const CHECKED_SURFACE_COUNT: usize = 11;

pub(super) const REQUIRED_SUMMARY_METRIC_NAMES: [&str; 9] = [
    "method",
    "mapped_reads",
    "endogenous_reads",
    "total_reads",
    "endogenous_fraction",
    "prealignment_fraction",
    "postalignment_fraction",
    "host_reference_scope",
    "caveats",
];

pub(super) const REQUIRED_NORMALIZED_METRIC_NAMES: [&str; 17] = [
    "expected_method",
    "method",
    "expected_host_reference_scope",
    "host_reference_scope",
    "expected_mapped_reads",
    "mapped_reads",
    "mapped_read_delta",
    "expected_endogenous_reads",
    "endogenous_reads",
    "expected_total_reads",
    "total_reads",
    "total_read_delta",
    "expected_endogenous_fraction",
    "endogenous_fraction",
    "endogenous_fraction_delta",
    "prealignment_fraction",
    "expectation_matched",
];

pub(super) const OPTIONAL_METRIC_NAMES: [&str; 1] = ["contaminant_reads"];
