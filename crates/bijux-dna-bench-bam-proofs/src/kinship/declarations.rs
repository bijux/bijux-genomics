#[cfg(feature = "bam_downstream")]
use bijux_dna_core::id_catalog::{BAM_GENOTYPING, BAM_KINSHIP, BAM_OVERLAP_CORRECTION};
pub const DEFAULT_BAM_KINSHIP_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/stages/bam.kinship.complete.json";

#[cfg(feature = "bam_downstream")]
pub(super) const BAM_KINSHIP_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_kinship_complete.v1";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_STAGE_ID: &str = BAM_KINSHIP;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_READY_CASE_SAMPLE_ID: &str = "human_like_kinship_related_pair";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_INSUFFICIENT_CASE_SAMPLE_ID: &str = "human_like_kinship_low_overlap_pair";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_METHOD: &str = "king";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_REFERENCE_PANEL: &str = "human_like_relatedness_panel";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_REFERENCE_BUILD: &str = "grch38";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_POPULATION_SCOPE: &str = "human_diploid_panel";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_READY_STATUS: &str = "ok";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_INSUFFICIENT_STATUS: &str = "insufficient";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_INSUFFICIENCY_REASON: &str = "insufficient_overlap_snps";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_PARSER_SCHEMA_VERSION: &str = "bijux.bam.kinship_summary.v1";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_REPORT_SCHEMA_VERSION: &str = "bijux.bam.kinship.v1";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.kinship_summary.v1";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.kinship.local_smoke.metrics.v1";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bam.kinship.local_smoke.report.v1";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_BAM_PIPELINE_ID: &str = "bam-kinship";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_VCF_PIPELINE_ID: &str = "bam-genotyping-to-vcf-downstream";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_READY_CASE_PAIR_COUNT: u64 = 1;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_INSUFFICIENT_CASE_PAIR_COUNT: u64 = 0;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_READY_CASE_OBSERVED_MAX_OVERLAP_SNPS: u64 = 6;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_INSUFFICIENT_CASE_OBSERVED_MAX_OVERLAP_SNPS: u64 = 4;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_READY_CASE_SAMPLE_A: &str = "sample_a";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_READY_CASE_SAMPLE_B: &str = "sample_b";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_READY_CASE_OVERLAP_SNPS: u64 = 6;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_READY_CASE_KINSHIP_COEFFICIENT: f64 = 0.416_667;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_READY_CASE_RELATIONSHIP_LABEL: &str = "first_degree";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_PAIRWISE_TABLE_ROW: &str =
    "sample_a\tsample_b\t6\t5\t1\t0.833333\t0.416667\tfirst_degree";

#[cfg(feature = "bam_downstream")]
pub(super) const CHECKED_SURFACE_COUNT: usize = 19;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_TOOL_IDS: [&str; 1] = ["king"];

#[cfg(feature = "bam_downstream")]
pub(super) const REQUIRED_OUTPUT_IDS: [&str; 3] = ["kinship_report", "summary", "stage_metrics"];

#[cfg(feature = "bam_downstream")]
pub(super) const REQUIRED_BAM_PIPELINE_DEPENDENCIES: [&str; 2] =
    [BAM_GENOTYPING, BAM_OVERLAP_CORRECTION];

#[cfg(feature = "bam_downstream")]
pub(super) const REQUIRED_BAM_PIPELINE_EXTERNAL_INPUTS: [&str; 3] = [
    "kinship_pairing_contract",
    "kinship_population_scope_contract",
    "kinship_reference_panel_contract",
];

#[cfg(feature = "bam_downstream")]
pub(super) const REQUIRED_BAM_PIPELINE_UPSTREAM_INPUTS: [&str; 4] = [
    "genotyping_report_json",
    "overlap_corrected_bai",
    "overlap_corrected_bam",
    "overlap_correction_summary_json",
];

#[cfg(feature = "bam_downstream")]
pub(super) const REQUIRED_BAM_PIPELINE_OUTPUTS: [&str; 4] = [
    "kinship_report_json",
    "kinship_segments_tsv",
    "kinship_stage_metrics",
    "kinship_summary_json",
];
