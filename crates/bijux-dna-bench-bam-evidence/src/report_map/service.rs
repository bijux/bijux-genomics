use super::{
    models::BamReportMapRow, projection::project_rows,
    stage_metadata::collect_bam_report_stage_metadata,
};
use crate::EvidenceContext;
use anyhow::Result;
pub(super) fn collect_report(
    context: &EvidenceContext<'_>,
) -> Result<(usize, Vec<BamReportMapRow>)> {
    let stage_admissions = context.reporting.stage_admissions(context.repo_root)?;
    let metadata = collect_bam_report_stage_metadata(context, &stage_admissions)?;
    let expected = context
        .reporting
        .expected_results(context.repo_root)?
        .into_iter()
        .filter(|row| row.domain == "bam")
        .collect::<Vec<_>>();
    let rows = project_rows(&metadata, &stage_admissions, &expected)?;
    Ok((expected.len(), rows))
}
/// Project each expected BAM result into its governed reporting section.
///
/// # Errors
/// Rejects missing domain metadata, unsupported report bindings, or identity drift.
pub fn collect_bam_report_map_rows(context: &EvidenceContext<'_>) -> Result<Vec<BamReportMapRow>> {
    collect_report(context).map(|(_, rows)| rows)
}
