use bijux_dna_core::id_catalog::BAM_AUTHENTICITY;
pub const DEFAULT_BAM_AUTHENTICITY_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/stages/bam.authenticity.complete.json";

pub(super) const BAM_AUTHENTICITY_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_authenticity_complete.v1";

pub(super) const EXPECTED_STAGE_ID: &str = BAM_AUTHENTICITY;

pub(super) const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bam.authenticity.local_smoke.report.v1";

pub(super) const EXPECTED_AUTHENTICITY_REPORT_SCHEMA_VERSION: &str = "bijux.bam.authenticity.v1";

pub(super) const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.authenticity_advisory.v1";

pub(super) const EXPECTED_COMPOSITION_SCHEMA_VERSION: &str =
    "bijux.bam.authenticity.composition.v1";

pub(super) const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.authenticity.local_smoke.metrics.v1";

pub(super) const EXPECTED_SAMPLE_ID: &str = "adna_damage_non_udg";

pub(super) const EXPECTED_METHOD: &str = "authenticct";

pub(super) const EXPECTED_SCORE: f64 = 0.533_333_333_333_333_3;

pub(super) const EXPECTED_CONFIDENCE: f64 = 0.813_333_333_333_333_4;

pub(super) const EXPECTED_STATUS: &str = "pass";

pub(super) const EXPECTED_PMD_LIKE_SIGNAL_PRESENT: bool = true;

pub(super) const EXPECTED_CONTAMINATION_ESTIMATE: f64 = 0.03;

pub(super) const CHECKED_SURFACE_COUNT: usize = 12;

pub(super) const EXPECTED_TOOL_IDS: [&str; 3] = ["authenticct", "damageprofiler", "pmdtools"];

pub(super) const REQUIRED_OUTPUT_IDS: [&str; 3] =
    ["authenticity_report", "summary", "stage_metrics"];

pub(super) const REQUIRED_SUMMARY_METRIC_NAMES: [&str; 5] =
    ["score", "confidence", "status", "pmd_like_signal_present", "advisory_boundary"];

pub(super) const REQUIRED_EVIDENCE_METRIC_IDS: [&str; 5] =
    ["damage", "contamination", "complexity", "coverage", "mapping"];

pub(super) const REQUIRED_REPORT_FIELDS: [&str; 3] = ["schema_version", "summary", "composition"];

pub(super) const REQUIRED_NORMALIZED_METRIC_NAMES: [&str; 15] = [
    "expected_score",
    "score",
    "score_delta",
    "expected_confidence",
    "confidence",
    "confidence_delta",
    "expected_status",
    "status",
    "expected_pmd_like_signal_present",
    "pmd_like_signal_present",
    "contamination_estimate",
    "expected_consumed_metric_ids",
    "consumed_metric_ids",
    "missing_metric_ids",
    "expectation_matched",
];
