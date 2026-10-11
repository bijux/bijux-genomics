use super::super::state::ReadinessState;
use crate::adapters::observations::VcfAdapterObservations;
use crate::evidence::registry::matrix::{
    render_vcf_matrix_registry_consistency, DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH,
};
use crate::evidence::registry::orphans::{render_vcf_orphan_tools, DEFAULT_VCF_ORPHAN_TOOLS_PATH};
use crate::evidence::registry::undercoverage::{
    render_vcf_undercovered_stages, DEFAULT_VCF_UNDERCOVERED_STAGES_PATH,
};
use crate::evidence::serving::{render_vcf_tool_serving_map, DEFAULT_VCF_TOOL_SERVING_MAP_PATH};
use anyhow::{bail, Result};
use std::path::Path;
pub(super) fn serving(
    observations: &VcfAdapterObservations<'_>,
    state: &mut ReadinessState,
) -> Result<String> {
    let repo_root = observations.repo_root();

    let report = render_vcf_tool_serving_map(
        observations.inputs(),
        repo_root,
        Path::new(DEFAULT_VCF_TOOL_SERVING_MAP_PATH),
    )?;
    if report.row_count != report.benchmark_ready_row_count
        || report.stage_count != 20
        || report.not_benchmark_ready_row_count != 0
    {
        bail!(
                    "VCF tool-serving map drifted: rows={}, stages={}, tools={}, benchmark_ready={}, not_benchmark_ready={}",
                    report.row_count,
                    report.stage_count,
                    report.tool_count,
                    report.benchmark_ready_row_count,
                    report.not_benchmark_ready_row_count
                );
    }
    state.counts.benchmark_ready = report.benchmark_ready_row_count;
    let row_count = report.row_count;
    state.tool_serving_map_report = Some(report);
    Ok(format!("validated {row_count} governed VCF stage-tool rows across all 20 supported stages"))
}

pub(super) fn orphans(observations: &VcfAdapterObservations<'_>) -> Result<String> {
    let repo_root = observations.repo_root();

    let report = render_vcf_orphan_tools(
        observations.inputs(),
        repo_root,
        Path::new(DEFAULT_VCF_ORPHAN_TOOLS_PATH),
    )?;
    if report.required_tool_count != 16
        || report.registered_tool_count != 16
        || report.served_tool_count != report.required_tool_count
        || report.orphan_count != 0
        || !report.rows.is_empty()
    {
        bail!("VCF orphan-tool report must be empty when every governed tool is served");
    }
    Ok("validated that every governed VCF tool serves at least one stage".to_string())
}

pub(super) fn undercoverage(observations: &VcfAdapterObservations<'_>) -> Result<String> {
    let repo_root = observations.repo_root();

    let report = render_vcf_undercovered_stages(
        observations.inputs(),
        repo_root,
        Path::new(DEFAULT_VCF_UNDERCOVERED_STAGES_PATH),
    )?;
    if report.stage_count != 20
        || report.undercovered_stage_count != 0
        || !report.decision_counts.is_empty()
        || !report.rows.is_empty()
    {
        bail!(
            "VCF undercovered-stage report drifted: stage_count={}, undercovered_stage_count={}",
            report.stage_count,
            report.undercovered_stage_count
        );
    }
    Ok("validated complete supported-tool coverage for every VCF stage".to_string())
}

pub(super) fn consistency(
    observations: &VcfAdapterObservations<'_>,
    state: &mut ReadinessState,
) -> Result<String> {
    let repo_root = observations.repo_root();

    let output_path = Path::new(DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH);
    let report =
        render_vcf_matrix_registry_consistency(observations.inputs(), repo_root, output_path)?;
    if !report.passes_gate
        || report.stage_count != 20
        || report.matrix_row_count != state.counts.benchmark_ready
        || report.registry_pair_count < report.matrix_row_count
        || report.benchmark_ready_registry_pair_count > report.matrix_row_count
        || report.unregistered_matrix_pair_count != 0
        || report.missing_benchmark_ready_registry_pair_count != 0
        || !report.rows.is_empty()
    {
        bail!("VCF matrix-registry consistency gate drifted from the governed clean pass state");
    }
    Ok("validated clean agreement between the governed VCF stage matrix and registry".to_string())
}
