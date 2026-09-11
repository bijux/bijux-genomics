use bijux_dna_core::id_catalog::BAM_SEX;
pub const DEFAULT_BAM_SEX_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/stages/bam.sex.complete.json";

pub(super) const BAM_SEX_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_sex_complete.v1";

pub(super) const EXPECTED_STAGE_ID: &str = BAM_SEX;

pub(super) const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str = "bijux.bam.sex.tool_smoke.report.v1";

pub(super) const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.sex.stage_metrics.v1";

pub(super) const EXPECTED_REPORT_SCHEMA_VERSION: &str = "bijux.bam.sex.v1";

pub(super) const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.sex_summary.v1";

pub(super) const EXPECTED_INSUFFICIENCY_REASON: &str = "insufficient_chromosomes";

pub(super) const CHECKED_SURFACE_COUNT: usize = 15;

pub(super) const EXPECTED_TOOL_IDS: [&str; 2] = ["rxy", "yleaf"];

pub(super) const REQUIRED_OUTPUT_IDS: [&str; 3] = ["sex_report", "summary", "stage_metrics"];
