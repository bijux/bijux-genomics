use bijux_dna_core::id_catalog::BAM_HAPLOGROUPS;
pub const DEFAULT_BAM_HAPLOGROUPS_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/stages/bam.haplogroups.complete.json";

pub(super) const BAM_HAPLOGROUPS_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_haplogroups_complete.v1";

pub(super) const EXPECTED_STAGE_ID: &str = BAM_HAPLOGROUPS;

pub(super) const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bam.haplogroups.local_smoke.report.v1";

pub(super) const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.haplogroup_readiness.v1";

pub(super) const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.haplogroups.stage_metrics.v1";

pub(super) const EXPECTED_REPORT_SCHEMA_VERSION: &str = "bijux.bam.haplogroups.v1";

pub(super) const EXPECTED_SAMPLE_ID: &str = "adna_y_haplogroup_panel";

pub(super) const EXPECTED_REFERENCE_PANEL_ID: &str = "adna-y-hg38-mini";

pub(super) const EXPECTED_REFERENCE_BUILD: &str = "hg38";

pub(super) const EXPECTED_POPULATION_SCOPE: &str = "adna_y_haplogroup_panel";

pub(super) const EXPECTED_READY_STATUS: &str = "ready";

pub(super) const EXPECTED_INSUFFICIENT_STATUS: &str = "coverage_gate_not_met";

pub(super) const CHECKED_SURFACE_COUNT: usize = 14;

pub(super) const EXPECTED_TOOL_IDS: [&str; 1] = ["yleaf"];

pub(super) const REQUIRED_OUTPUT_IDS: [&str; 3] = ["haplogroups", "summary", "stage_metrics"];
