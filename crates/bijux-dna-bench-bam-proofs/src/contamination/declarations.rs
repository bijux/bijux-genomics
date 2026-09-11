use bijux_dna_core::id_catalog::BAM_CONTAMINATION;
pub const DEFAULT_BAM_CONTAMINATION_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/stages/bam.contamination.complete.json";

pub(super) const BAM_CONTAMINATION_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_contamination_complete.v1";

pub(super) const EXPECTED_STAGE_ID: &str = BAM_CONTAMINATION;

pub(super) const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bam.contamination.local_smoke.report.v1";

pub(super) const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.contamination.stage_metrics.v1";

pub(super) const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.contamination_evidence.v1";

pub(super) const CHECKED_SURFACE_COUNT: usize = 14;

pub(super) const EXPECTED_TOOL_IDS: [&str; 3] = ["contammix", "schmutzi", "verifybamid2"];

pub(super) const REQUIRED_OUTPUT_IDS: [&str; 3] =
    ["contamination_report", "summary", "stage_metrics"];
