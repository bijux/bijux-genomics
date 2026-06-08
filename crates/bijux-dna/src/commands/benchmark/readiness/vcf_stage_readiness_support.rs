use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_rows, AllDomainActiveStageToolMatrixRow,
};
use super::vcf_active_stage_tool_matrix::{
    collect_vcf_active_stage_tool_matrix_rows, VcfActiveStageToolMatrixRow,
};
use super::vcf_adapter_output_coverage::{
    collect_vcf_adapter_output_coverage_rows, VcfAdapterOutputCoverageRow,
};
use super::vcf_expected_benchmark_results::{
    collect_vcf_expected_benchmark_result_rows, VcfExpectedBenchmarkResultRow,
};
use super::vcf_parser_coverage::{collect_vcf_parser_coverage_rows, VcfParserCoverageRow};
use super::vcf_rendered_command_rows::VcfRenderedCommandRow;
use super::vcf_rendered_commands::{
    render_vcf_commands, VcfRenderedCommandsReport, DEFAULT_VCF_RENDERED_COMMANDS_PATH,
};
use super::vcf_report_map::{collect_vcf_report_map_rows, VcfReportMapRow};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct VcfStageReadinessBindingKey {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
}

#[derive(Debug, Clone)]
pub(crate) struct VcfStageReadinessBinding {
    pub(crate) retained_row: VcfActiveStageToolMatrixRow,
    pub(crate) active_row: Option<AllDomainActiveStageToolMatrixRow>,
    pub(crate) command_row: Option<VcfRenderedCommandRow>,
    pub(crate) output_row: Option<VcfAdapterOutputCoverageRow>,
    pub(crate) parser_row: Option<VcfParserCoverageRow>,
    pub(crate) expected_row: Option<VcfExpectedBenchmarkResultRow>,
    pub(crate) report_row: Option<VcfReportMapRow>,
}

pub(crate) fn collect_vcf_stage_readiness_bindings(
    repo_root: &Path,
    stage_id: &str,
) -> Result<(VcfRenderedCommandsReport, Vec<VcfStageReadinessBinding>)> {
    let retained_rows = collect_vcf_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| row.stage_id == stage_id)
        .collect::<Vec<_>>();
    if retained_rows.is_empty() {
        return Err(anyhow!("VCF stage readiness is missing retained `{stage_id}` bindings"));
    }

    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| row.domain == "vcf" && row.stage_id == stage_id)
        .collect::<Vec<_>>();
    let active_by_key = active_rows
        .into_iter()
        .map(|row| (binding_key_from_active_row(&row), row))
        .collect::<BTreeMap<_, _>>();

    let command_report =
        render_vcf_commands(repo_root, PathBuf::from(DEFAULT_VCF_RENDERED_COMMANDS_PATH))?;
    let command_by_tool = command_report
        .rows
        .iter()
        .filter(|row| row.stage_id == stage_id)
        .cloned()
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let output_by_tool = collect_vcf_adapter_output_coverage_rows(repo_root)?
        .into_iter()
        .filter(|row| row.stage_id == stage_id)
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let parser_by_tool = collect_vcf_parser_coverage_rows(repo_root)?
        .2
        .into_iter()
        .filter(|row| row.stage_id == stage_id)
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let expected_by_key = collect_vcf_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .filter(|row| row.stage_id == stage_id)
        .map(|row| (binding_key_from_expected_row(&row), row))
        .collect::<BTreeMap<_, _>>();
    let report_by_tool = collect_vcf_report_map_rows(repo_root)?
        .into_iter()
        .filter(|row| row.stage_id == stage_id)
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(retained_rows.len());
    for retained_row in retained_rows {
        rows.push(VcfStageReadinessBinding {
            active_row: active_by_key.get(&binding_key_from_retained_row(&retained_row)).cloned(),
            command_row: command_by_tool.get(&retained_row.tool_id).cloned(),
            output_row: output_by_tool.get(&retained_row.tool_id).cloned(),
            parser_row: parser_by_tool.get(&retained_row.tool_id).cloned(),
            expected_row: expected_by_key
                .get(&binding_key_from_retained_row(&retained_row))
                .cloned(),
            report_row: report_by_tool.get(&retained_row.tool_id).cloned(),
            retained_row,
        });
    }
    rows.sort_by(|left, right| {
        left.retained_row
            .stage_id
            .cmp(&right.retained_row.stage_id)
            .then_with(|| left.retained_row.tool_id.cmp(&right.retained_row.tool_id))
    });
    Ok((command_report, rows))
}

pub(crate) fn binding_key_from_retained_row(
    row: &VcfActiveStageToolMatrixRow,
) -> VcfStageReadinessBindingKey {
    VcfStageReadinessBindingKey {
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

pub(crate) fn binding_key_from_active_row(
    row: &AllDomainActiveStageToolMatrixRow,
) -> VcfStageReadinessBindingKey {
    VcfStageReadinessBindingKey {
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

pub(crate) fn binding_key_from_expected_row(
    row: &VcfExpectedBenchmarkResultRow,
) -> VcfStageReadinessBindingKey {
    VcfStageReadinessBindingKey {
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}
