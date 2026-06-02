use anyhow::{Context, Result};

use super::{
    ClusterOtusReportV1, DetectDuplicatesPremergeReportV1, FilterReadsReportV1,
    IndexReferenceReportV1, InferAsvsReportV1, MergePairsReportV1, NormalizeAbundanceReportV1,
    NormalizePrimersReportV1, ReportQcReportV1, ScreenTaxonomyReportV1, TerminalDamageReportV1,
    TrimPolygReportV1, TrimReadsReportV1, ValidatedReadsManifestV1, ValidationReportV1,
};

/// # Errors
/// Returns an error if the governed validation report JSON cannot be parsed.
pub fn parse_validation_report(report_json: &str) -> Result<ValidationReportV1> {
    serde_json::from_str(report_json).context("parse validation report")
}

/// # Errors
/// Returns an error if the governed filter-reads report JSON cannot be parsed.
pub fn parse_filter_reads_report(report_json: &str) -> Result<FilterReadsReportV1> {
    serde_json::from_str(report_json).context("parse filter-reads report")
}

/// # Errors
/// Returns an error if the governed detect-duplicates-premerge report JSON cannot be parsed.
pub fn parse_detect_duplicates_premerge_report(
    report_json: &str,
) -> Result<DetectDuplicatesPremergeReportV1> {
    serde_json::from_str(report_json).context("parse detect duplicates premerge report")
}

/// # Errors
/// Returns an error if the governed index-reference report JSON cannot be parsed.
pub fn parse_index_reference_report(report_json: &str) -> Result<IndexReferenceReportV1> {
    serde_json::from_str(report_json).context("parse index-reference report")
}

/// # Errors
/// Returns an error if the governed validated-reads lineage manifest JSON cannot be parsed.
pub fn parse_validated_reads_manifest(manifest_json: &str) -> Result<ValidatedReadsManifestV1> {
    serde_json::from_str(manifest_json).context("parse validated reads manifest")
}

/// # Errors
/// Returns an error if the governed terminal-damage report JSON cannot be parsed.
pub fn parse_terminal_damage_report(report_json: &str) -> Result<TerminalDamageReportV1> {
    serde_json::from_str(report_json).context("parse terminal damage report")
}

/// # Errors
/// Returns an error if the governed trim report JSON cannot be parsed.
pub fn parse_trim_reads_report(report_json: &str) -> Result<TrimReadsReportV1> {
    serde_json::from_str(report_json).context("parse trim reads report")
}

/// # Errors
/// Returns an error if the governed trim-polyg report JSON cannot be parsed.
pub fn parse_trim_polyg_report(report_json: &str) -> Result<TrimPolygReportV1> {
    serde_json::from_str(report_json).context("parse trim polyg report")
}

/// # Errors
/// Returns an error if the governed normalize-primers report JSON cannot be parsed.
pub fn parse_normalize_primers_report(report_json: &str) -> Result<NormalizePrimersReportV1> {
    serde_json::from_str(report_json).context("parse normalize primers report")
}

/// # Errors
/// Returns an error if the governed normalize-abundance report JSON cannot be parsed.
pub fn parse_normalize_abundance_report(report_json: &str) -> Result<NormalizeAbundanceReportV1> {
    serde_json::from_str(report_json).context("parse normalize abundance report")
}

/// # Errors
/// Returns an error if the governed infer-asvs report JSON cannot be parsed.
pub fn parse_infer_asvs_report(report_json: &str) -> Result<InferAsvsReportV1> {
    serde_json::from_str(report_json).context("parse infer asvs report")
}

/// # Errors
/// Returns an error if the governed cluster-otus report JSON cannot be parsed.
pub fn parse_cluster_otus_report(report_json: &str) -> Result<ClusterOtusReportV1> {
    serde_json::from_str(report_json).context("parse cluster otus report")
}

/// # Errors
/// Returns an error if the governed merge-pairs report JSON cannot be parsed.
pub fn parse_merge_pairs_report(report_json: &str) -> Result<MergePairsReportV1> {
    serde_json::from_str(report_json).context("parse merge pairs report")
}

/// # Errors
/// Returns an error if the governed report-qc report JSON cannot be parsed.
pub fn parse_report_qc_report(report_json: &str) -> Result<ReportQcReportV1> {
    serde_json::from_str(report_json).context("parse report qc report")
}

/// # Errors
/// Returns an error if the governed taxonomy-screen report JSON cannot be parsed.
pub fn parse_screen_taxonomy_report(report_json: &str) -> Result<ScreenTaxonomyReportV1> {
    serde_json::from_str(report_json).context("parse screen taxonomy report")
}
