use super::{declarations::BamStageCompletionGoal, ports::StageProofKind, RetainedContext};
use anyhow::{bail, Result};
use bijux_dna_core::id_catalog::{
    BAM_AUTHENTICITY, BAM_CONTAMINATION, BAM_DAMAGE, BAM_ENDOGENOUS_CONTENT, BAM_GENOTYPING,
    BAM_HAPLOGROUPS, BAM_KINSHIP, BAM_OVERLAP_CORRECTION, BAM_RECALIBRATION, BAM_SEX,
};
use std::path::Path;
pub(super) fn validate_stage_completion_goal(
    context: &RetainedContext<'_>,
    goal: BamStageCompletionGoal,
) -> Result<String> {
    let kind = match goal.stage_id {
        BAM_OVERLAP_CORRECTION => StageProofKind::OverlapCorrection,
        BAM_ENDOGENOUS_CONTENT => StageProofKind::EndogenousContent,
        BAM_DAMAGE => StageProofKind::Damage,
        BAM_AUTHENTICITY => StageProofKind::Authenticity,
        BAM_CONTAMINATION => StageProofKind::Contamination,
        BAM_SEX => StageProofKind::Sex,
        BAM_HAPLOGROUPS => StageProofKind::Haplogroups,
        BAM_RECALIBRATION => StageProofKind::Recalibration,
        BAM_GENOTYPING => StageProofKind::Genotyping,
        BAM_KINSHIP => StageProofKind::Kinship,
        _ => bail!("unknown BAM stage completion goal `{}`", goal.stage_id),
    };
    let report = context.stages.assess(context.repo_root, kind, Path::new(goal.output_path))?;
    validate_stage_completion_counts(
        goal,
        report.complete_row_count,
        report.active_row_count,
        report.checked_surface_count,
    )?;
    Ok(format_stage_completion_detail(
        goal.stage_id,
        report.complete_row_count,
        report.active_row_count,
        report.checked_surface_count,
    ))
}

pub(super) fn validate_stage_completion_counts(
    goal: BamStageCompletionGoal,
    complete_row_count: usize,
    active_row_count: usize,
    checked_surface_count: usize,
) -> Result<()> {
    if active_row_count != goal.expected_active_row_count
        || complete_row_count != goal.expected_active_row_count
        || checked_surface_count != goal.expected_checked_surface_count
    {
        bail!(
            "stage completion counts for `{}` drifted: active_rows={}, complete_rows={}, checked_surfaces={}, expected_active_rows={}, expected_checked_surfaces={}",
            goal.stage_id,
            active_row_count,
            complete_row_count,
            checked_surface_count,
            goal.expected_active_row_count,
            goal.expected_checked_surface_count
        );
    }
    Ok(())
}
pub(super) fn format_stage_completion_detail(
    stage_id: &str,
    complete_row_count: usize,
    active_row_count: usize,
    checked_surface_count: usize,
) -> String {
    format!(
        "validated `{stage_id}` completion across {complete_row_count}/{active_row_count} retained row(s) and {checked_surface_count} governed surface(s)"
    )
}
