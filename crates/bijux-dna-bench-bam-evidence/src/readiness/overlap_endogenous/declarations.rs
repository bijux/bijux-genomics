use bijux_dna_core::id_catalog::{BAM_ENDOGENOUS_CONTENT, BAM_OVERLAP_CORRECTION};

pub(in crate::readiness) const BAM_OVERLAP_ENDOGENOUS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_overlap_endogenous_ready.v1";
pub(in crate::readiness) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(in crate::readiness) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(in crate::readiness) const CHECKED_SURFACE_COUNT: usize = 8;
pub(in crate::readiness) const EXPECTED_SCHEMA_REQUIRED_KEYS: [&str; 9] = [
    "schema_version",
    "stage_id",
    "tool_id",
    "tool_version",
    "execution",
    "outputs_count",
    "artifacts",
    "contracts",
    "normalized_keys",
];
#[derive(Debug, Clone, Copy)]
pub(in crate::readiness) struct BamStageSpec {
    pub(in crate::readiness) stage_id: &'static str,
    pub(in crate::readiness) tool_id: &'static str,
    pub(in crate::readiness) required_output_ids: &'static [&'static str],
    pub(in crate::readiness) normalized_metrics_output_id: &'static str,
    pub(in crate::readiness) schema_extension_id: &'static str,
    pub(in crate::readiness) required_local_smoke_fields: &'static [&'static str],
    pub(in crate::readiness) report_section_id: &'static str,
    pub(in crate::readiness) summary_table_id: &'static str,
}
pub(in crate::readiness) const ENDOGENOUS_REQUIRED_OUTPUT_IDS: [&str; 3] =
    ["endogenous_report", "summary", "stage_metrics"];
pub(in crate::readiness) const OVERLAP_REQUIRED_OUTPUT_IDS: [&str; 4] =
    ["overlap_corrected_bam", "overlap_corrected_bai", "summary", "stage_metrics"];
pub(in crate::readiness) const ENDOGENOUS_REQUIRED_LOCAL_SMOKE_FIELDS: [&str; 4] =
    ["mapped_reads", "total_reads", "endogenous_fraction", "method"];
pub(in crate::readiness) const OVERLAP_REQUIRED_LOCAL_SMOKE_FIELDS: [&str; 4] =
    ["pair_count", "corrected_pairs", "corrected_overlap_bases", "method"];
pub(in crate::readiness) const BAM_STAGE_SPECS: [BamStageSpec; 2] = [
    BamStageSpec {
        stage_id: BAM_ENDOGENOUS_CONTENT,
        tool_id: "samtools",
        required_output_ids: &ENDOGENOUS_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "endogenous_report",
        schema_extension_id: "bam_endogenous_content_normalized_v1",
        required_local_smoke_fields: &ENDOGENOUS_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "coverage_quality",
        summary_table_id: "coverage_bias_qc",
    },
    BamStageSpec {
        stage_id: BAM_OVERLAP_CORRECTION,
        tool_id: "bamutil",
        required_output_ids: &OVERLAP_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "summary",
        schema_extension_id: "bam_overlap_correction_normalized_v1",
        required_local_smoke_fields: &OVERLAP_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "alignment_refinement",
        summary_table_id: "filter_retention",
    },
];
