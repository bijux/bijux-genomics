use super::{
    anchors::select_anchor_tool, metadata::BamReportStageMetadata, placement::placement_for_stage,
};
use crate::{observations::StageAdmissions, EvidenceContext};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
pub(super) fn collect_bam_report_stage_metadata(
    context: &EvidenceContext<'_>,
    stage_admissions: &StageAdmissions,
) -> Result<BTreeMap<String, BamReportStageMetadata>> {
    let repo_root = context.repo_root;
    let inventory = context.reporting.stage_inventory(repo_root)?;
    let mut rows = BTreeMap::new();
    for stage in inventory {
        let placement = placement_for_stage(stage.stage_id.as_str()).ok_or_else(|| {
            anyhow!("BAM report map is missing stage placement for `{}`", stage.stage_id)
        })?;
        let domain = context.reporting.report_stage(repo_root, &stage.stage_id)?;
        let criticality = domain.criticality.ok_or_else(|| {
            anyhow!("BAM report map is missing stage criticality for `{}`", stage.stage_id)
        })?;
        let admissions = stage_admissions.get(stage.stage_id.as_str()).ok_or_else(|| {
            anyhow!("BAM report map is missing admitted benchmark tools for `{}`", stage.stage_id)
        })?;
        let preferred_default_tool_id = domain.preferred_tool_id;
        let (anchor_tool_id, anchor_support_status) =
            select_anchor_tool(admissions, preferred_default_tool_id.as_str())?;
        let canonical_stage_rank = domain
            .canonical_rank
            .ok_or_else(|| anyhow!("BAM canonical stage order is missing `{}`", stage.stage_id))?;

        rows.insert(
            stage.stage_id.clone(),
            BamReportStageMetadata {
                canonical_stage_rank,
                readiness_kind: stage.readiness_kind.to_string(),
                criticality: criticality.to_string(),
                anchor_tool_id: anchor_tool_id.to_string(),
                anchor_support_status: anchor_support_status.to_string(),
                report_section_id: placement.section_id.to_string(),
                report_section_title: placement.section_title.to_string(),
                summary_table_id: placement.summary_table_id.to_string(),
                summary_table_title: placement.summary_table_title.to_string(),
                workflow_branch_id: domain.workflow_branch_id,
                scientific_context_required: domain.scientific_context_required,
                report_focus: placement.report_focus.to_string(),
            },
        );
    }

    Ok(rows)
}
