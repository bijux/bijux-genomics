use bijux_dna_core::id_catalog::BAM_RECALIBRATION;
pub const DEFAULT_BAM_RECALIBRATION_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/stages/bam.recalibration.complete.json";

pub(super) const BAM_RECALIBRATION_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_recalibration_complete.v1";

pub(super) const EXPECTED_STAGE_ID: &str = BAM_RECALIBRATION;

pub(super) const EXPECTED_TOOL_ID: &str = "gatk";

pub(super) const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bam.recalibration.local_smoke.report.v1";

pub(super) const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.recalibration.v1";

pub(super) const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.recalibration.local_smoke.metrics.v1";

pub(super) const EXPECTED_SAMPLE_ID: &str = "human_like_recalibration_low_coverage";

pub(super) const EXPECTED_KNOWN_SITES_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf";

pub(super) const EXPECTED_KNOWN_SITES_ASSET_ID: &str = "human_like_recalibration_known_sites";

pub(super) const EXPECTED_REFERENCE_FASTA: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta";

pub(super) const EXPECTED_REQUESTED_MODE: &str = "standard";

pub(super) const EXPECTED_EFFECTIVE_MODE: &str = "skip";

pub(super) const EXPECTED_STATUS: &str = "skipped";

pub(super) const EXPECTED_REASON: &str = "coverage_below_gate";

pub(super) const EXPECTED_MIN_MEAN_COVERAGE: f64 = 0.2;

pub(super) const EXPECTED_MIN_BREADTH_1X: f64 = 0.2;

pub(super) const EXPECTED_OBSERVED_MEAN_COVERAGE: f64 = 0.192;

pub(super) const EXPECTED_OBSERVED_BREADTH_1X: f64 = 0.192;

pub(super) const CHECKED_SURFACE_COUNT: usize = 13;

pub(super) const REQUIRED_OUTPUT_IDS: [&str; 5] =
    ["recal_bam", "recal_bai", "recal_report", "summary", "stage_metrics"];
