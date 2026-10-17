use super::format::criticality_label;
use super::format::metric_class_label;
use super::format::readiness_kind_label;
use super::format::stage_kind_label;
use super::models::FastqReportStageMetadata;
use super::placement::placement_for_stage;
use crate::readiness::declarations::observations::StageAdmission;
use crate::readiness::declarations::ReportContext;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::default_execution_tool_for_stage;
use bijux_dna_domain_fastq::stage_criticality;
use bijux_dna_domain_fastq::stage_metric_classes;
use bijux_dna_domain_fastq::stage_semantics::stage_kind;
use bijux_dna_domain_fastq::stage_semantics::stage_semantics;
use bijux_dna_domain_fastq::stages::semantics::STAGES;
use std::collections::BTreeMap;

/// Metadata and support admissions from the same request observation.
pub(super) struct ReportStageObservations {
    pub(super) metadata: BTreeMap<String, FastqReportStageMetadata>,
    pub(super) admissions: BTreeMap<String, Vec<StageAdmission>>,
}

/// Collect canonical FASTQ declaration evidence.
///
/// # Errors
/// Returns source admission, contract or publication failure.
pub fn collect_fastq_report_stage_metadata(
    context: &ReportContext<'_>,
) -> Result<BTreeMap<String, FastqReportStageMetadata>> {
    Ok(collect_report_stage_observations(context)?.metadata)
}

pub(super) fn collect_report_stage_observations(
    context: &ReportContext<'_>,
) -> Result<ReportStageObservations> {
    let repo_root = context.repo_root;
    let inventory = context.evidence.inventory(repo_root)?;
    let stage_admissions = context.evidence.admissions(repo_root)?;
    let canonical_rank_by_stage = STAGES
        .iter()
        .enumerate()
        .map(|(idx, stage)| (stage.stage_id.as_str().to_string(), idx + 1))
        .collect::<BTreeMap<_, _>>();

    let mut rows = BTreeMap::new();
    for stage in inventory.stages {
        let stage_id = StageId::new(stage.stage_id.clone());
        let placement = placement_for_stage(stage_id.as_str()).ok_or_else(|| {
            anyhow!("FASTQ report map is missing stage placement for `{stage_id}`")
        })?;
        let semantics = stage_semantics(&stage_id).ok_or_else(|| {
            anyhow!("FASTQ report map is missing stage semantics for `{stage_id}`")
        })?;
        let stage_kind = stage_kind(&stage_id)
            .ok_or_else(|| anyhow!("FASTQ report map is missing stage kind for `{stage_id}`"))?;
        let criticality = stage_criticality(&stage_id)
            .ok_or_else(|| anyhow!("FASTQ report map is missing criticality for `{stage_id}`"))?;
        let admissions = stage_admissions.get(stage_id.as_str()).ok_or_else(|| {
            anyhow!("FASTQ report map is missing admitted benchmark tools for `{stage_id}`")
        })?;
        let preferred_default_tool_id =
            default_execution_tool_for_stage(&stage_id).map(|tool_id| tool_id.to_string());
        let (anchor_tool_id, anchor_support_status) =
            select_anchor_tool(admissions, preferred_default_tool_id.as_deref())?;
        let metric_classes = stage_metric_classes(&stage_id)
            .unwrap_or(&[])
            .iter()
            .copied()
            .map(metric_class_label)
            .map(str::to_string)
            .collect::<Vec<_>>();
        let canonical_stage_rank = canonical_rank_by_stage
            .get(stage_id.as_str())
            .copied()
            .ok_or_else(|| anyhow!("FASTQ canonical stage order is missing `{stage_id}`"))?;

        rows.insert(
            stage_id.to_string(),
            FastqReportStageMetadata {
                canonical_stage_rank,
                readiness_kind: readiness_kind_label(stage.readiness_kind).to_string(),
                stage_kind: stage_kind_label(stage_kind).to_string(),
                criticality: criticality_label(criticality).to_string(),
                anchor_tool_id: anchor_tool_id.to_string(),
                anchor_support_status: anchor_support_status.to_string(),
                report_section_id: placement.section_id.to_string(),
                report_section_title: placement.section_title.to_string(),
                summary_table_id: placement.summary_table_id.to_string(),
                summary_table_title: placement.summary_table_title.to_string(),
                metric_classes,
                mutates_fastq: semantics.mutates_fastq,
                produces_reports_only: semantics.produces_reports_only,
                report_focus: placement.report_focus.to_string(),
            },
        );
    }

    Ok(ReportStageObservations { metadata: rows, admissions: stage_admissions })
}

pub(super) fn admission_support_status<'a>(
    admissions: &'a [StageAdmission],
    tool_id: &str,
) -> Result<&'a str> {
    admissions
        .iter()
        .find(|admission| admission.tool_id == tool_id)
        .map(|admission| admission.support_status.as_str())
        .ok_or_else(|| anyhow!("FASTQ report map stage is missing support status for `{tool_id}`"))
}

pub(super) fn select_anchor_tool<'a>(
    admissions: &'a [StageAdmission],
    preferred_tool_id: Option<&str>,
) -> Result<(&'a str, &'a str)> {
    if let Some(preferred_tool_id) = preferred_tool_id {
        if let Some(admission) =
            admissions.iter().find(|admission| admission.tool_id == preferred_tool_id)
        {
            return Ok((admission.tool_id.as_str(), admission.support_status.as_str()));
        }
    }

    let admission = admissions
        .iter()
        .min_by(|left, right| {
            admission_support_rank(&left.support_status)
                .cmp(&admission_support_rank(&right.support_status))
                .then_with(|| left.tool_id.cmp(&right.tool_id))
        })
        .ok_or_else(|| anyhow!("FASTQ report map stage has no admitted benchmark tools"))?;
    Ok((admission.tool_id.as_str(), admission.support_status.as_str()))
}

pub(super) fn admission_support_rank(status: &str) -> usize {
    match status {
        "supported" => 0,
        "planned" => 1,
        _ => 2,
    }
}
