use bijux_dna_core::id_catalog::BAM_OVERLAP_CORRECTION;
pub const DEFAULT_BAM_OVERLAP_CORRECTION_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/stages/bam.overlap_correction.complete.json";

pub(super) const BAM_OVERLAP_CORRECTION_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_overlap_correction_complete.v1";

pub(super) const EXPECTED_STAGE_ID: &str = BAM_OVERLAP_CORRECTION;

pub(super) const EXPECTED_TOOL_ID: &str = "bamutil";

pub(super) const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.overlap_correction.v1";

pub(super) const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.overlap_correction.local_smoke.metrics.v1";

pub(super) const EXPECTED_CORRECTED_PAIR_COUNT: u64 = 1;

pub(super) const EXPECTED_CORRECTED_OVERLAP_BASES: u64 = 7;

pub(super) const EXPECTED_PAIR_COUNT: u64 = 2;

pub(super) const CHECKED_SURFACE_COUNT: usize = 11;

pub(super) const REQUIRED_NORMALIZED_METRIC_NAMES: [&str; 11] = [
    "method",
    "expected_pair_count",
    "pair_count",
    "pair_count_delta",
    "expected_corrected_pairs",
    "corrected_pairs",
    "corrected_pair_delta",
    "expected_corrected_overlap_bases",
    "corrected_overlap_bases",
    "corrected_overlap_base_delta",
    "expectation_matched",
];
