use super::super::keys::{
    binding_key_from_expected_result_row, binding_key_from_local_smoke_row,
    binding_key_from_parser_fixture_row, binding_key_from_rendered_command_row,
    binding_key_from_report_map_row,
};
use super::super::{
    model::{
        VcfAllRetainedToolsCompleteGoalCheck, EXPECTED_ACTIVE_STAGE_COUNT,
        EXPECTED_CONTAINER_SMOKE_ROW_COUNT, EXPECTED_EXTERNAL_ARTIFACT_ROW_COUNT,
        EXPECTED_HOST_STAGE_SMOKE_ROW_COUNT, EXPECTED_RETAINED_ROW_COUNT,
        EXPECTED_RETAINED_STAGE_COUNT, EXPECTED_RETAINED_TOOL_COUNT,
    },
    ports::VcfCompletionPorts,
    state::CompletionState,
};
use super::record_goal_check;
use anyhow::bail;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;
pub(super) fn smoke(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        356,
        "vcf local and container smoke coverage",
        Some(crate::evidence::smoke::DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH.to_string()),
        || {
            let report = (ports.evidence.smoke)(
                repo_root,
                PathBuf::from(crate::evidence::smoke::DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH),
            )?;
            if report.row_count != EXPECTED_RETAINED_ROW_COUNT
                || report.stage_count != EXPECTED_RETAINED_STAGE_COUNT
                || report.tool_count != EXPECTED_RETAINED_TOOL_COUNT
                || report.host_stage_smoke_row_count != EXPECTED_HOST_STAGE_SMOKE_ROW_COUNT
                || report.container_smoke_row_count != EXPECTED_CONTAINER_SMOKE_ROW_COUNT
                || report.external_artifact_row_count != EXPECTED_EXTERNAL_ARTIFACT_ROW_COUNT
            {
                bail!(
                    "VCF local/container smoke drifted: rows={}, stages={}, tools={}, host_stage_smokes={}, container_smokes={}, external_artifacts={}",
                    report.row_count,
                    report.stage_count,
                    report.tool_count,
                    report.host_stage_smoke_row_count,
                    report.container_smoke_row_count,
                    report.external_artifact_row_count
                );
            }
            let observed_bindings =
                report.rows.iter().map(binding_key_from_local_smoke_row).collect::<BTreeSet<_>>();
            if observed_bindings != state.retained_bindings {
                bail!(
                    "VCF local/container smoke drifted from the retained binding slice: missing={:?} extra={:?}",
                    state.retained_bindings
                        .difference(&observed_bindings)
                        .cloned()
                        .collect::<Vec<_>>(),
                    observed_bindings
                        .difference(&state.retained_bindings)
                        .cloned()
                        .collect::<Vec<_>>()
                );
            }
            state.local_smoke_row = report.row_count;
            state.local_smoke_host_stage_row = report.host_stage_smoke_row_count;
            state.local_smoke_container_row = report.container_smoke_row_count;
            state.local_smoke_external_artifact_row = report.external_artifact_row_count;
            Ok(
                "validated governed host, container, and external-artifact admission coverage for every retained VCF row"
                    .to_string(),
            )
        },
    );
}
pub(super) fn parser_coverage(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        357,
        "vcf parser fixture coverage",
        Some(crate::evidence::parsers::DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH.to_string()),
        || {
            let report = (ports.evidence.parsers)(
                repo_root,
                PathBuf::from(crate::evidence::parsers::DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH),
            )?;
            if report.row_count != state.active_bindings.len()
                || report.stage_count != EXPECTED_ACTIVE_STAGE_COUNT
                || report.tool_count != state.active_tool
                || report.covered_row_count != state.active_bindings.len()
                || report.missing_row_count != 0
                || (report.parser_fixture_coverage_percent - 100.0).abs() > f64::EPSILON
            {
                bail!(
                    "VCF parser fixture coverage drifted: rows={}, stages={}, tools={}, covered={}, missing={}, percent={}",
                    report.row_count,
                    report.stage_count,
                    report.tool_count,
                    report.covered_row_count,
                    report.missing_row_count,
                    report.parser_fixture_coverage_percent
                );
            }
            let observed_bindings = report
                .rows
                .iter()
                .map(binding_key_from_parser_fixture_row)
                .collect::<BTreeSet<_>>();
            if observed_bindings != state.active_bindings {
                bail!(
                    "VCF parser fixture coverage drifted from the active binding slice: missing={:?} extra={:?}",
                    state.active_bindings
                        .difference(&observed_bindings)
                        .cloned()
                        .collect::<Vec<_>>(),
                    observed_bindings
                        .difference(&state.active_bindings)
                        .cloned()
                        .collect::<Vec<_>>()
                );
            }
            state.parser_fixture_row = report.row_count;
            Ok("validated 100% parser fixture coverage for the active VCF binding slice"
                .to_string())
        },
    );
}
pub(super) fn commands(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        358,
        "vcf rendered commands",
        Some(crate::evidence::commands::DEFAULT_VCF_RENDERED_COMMANDS_PATH.to_string()),
        || {
            let report = (ports.evidence.commands)(
                repo_root,
                PathBuf::from(crate::evidence::commands::DEFAULT_VCF_RENDERED_COMMANDS_PATH),
            )?;
            if report.row_count != state.active_bindings.len() {
                bail!(
                    "VCF rendered commands drifted from the active VCF binding slice: rows={}",
                    report.row_count
                );
            }
            let observed_bindings = report
                .rows
                .iter()
                .map(binding_key_from_rendered_command_row)
                .collect::<BTreeSet<_>>();
            if observed_bindings != state.active_bindings {
                bail!(
                    "VCF rendered command bindings drifted from the active slice: missing={:?} extra={:?}",
                    state.active_bindings
                        .difference(&observed_bindings)
                        .cloned()
                        .collect::<Vec<_>>(),
                    observed_bindings
                        .difference(&state.active_bindings)
                        .cloned()
                        .collect::<Vec<_>>()
                );
            }
            state.rendered_command_row = report.row_count;
            Ok("validated one rendered command bundle for every active VCF binding".to_string())
        },
    );
}
pub(super) fn report_placement(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        359,
        "vcf report map",
        Some(crate::evidence::reports::DEFAULT_VCF_REPORT_MAP_PATH.to_string()),
        || {
            let expected_results_report = (ports.evidence.expectations)(
                repo_root,
                PathBuf::from(
                    crate::evidence::expectations::DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH,
                ),
            )?;
            let report = (ports.evidence.reports)(
                repo_root,
                PathBuf::from(crate::evidence::reports::DEFAULT_VCF_REPORT_MAP_PATH),
            )?;
            if expected_results_report.row_count != state.active_bindings.len()
                || report.row_count != state.active_bindings.len()
            {
                bail!(
                    "VCF expected-result/report-map counts drifted: expected_results={}, report_map={}",
                    expected_results_report.row_count,
                    report.row_count
                );
            }
            let expected_bindings = expected_results_report
                .rows
                .iter()
                .map(binding_key_from_expected_result_row)
                .collect::<BTreeSet<_>>();
            let report_bindings =
                report.rows.iter().map(binding_key_from_report_map_row).collect::<BTreeSet<_>>();
            if expected_bindings != state.active_bindings
                || report_bindings != state.active_bindings
            {
                bail!(
                    "VCF expected-result/report-map bindings drifted from the active slice: expected_missing={:?} expected_extra={:?} report_missing={:?} report_extra={:?}",
                    state.active_bindings
                        .difference(&expected_bindings)
                        .cloned()
                        .collect::<Vec<_>>(),
                    expected_bindings
                        .difference(&state.active_bindings)
                        .cloned()
                        .collect::<Vec<_>>(),
                    state.active_bindings
                        .difference(&report_bindings)
                        .cloned()
                        .collect::<Vec<_>>(),
                    report_bindings
                        .difference(&state.active_bindings)
                        .cloned()
                        .collect::<Vec<_>>()
                );
            }
            state.expected_result_row = expected_results_report.row_count;
            state.report_map_row = report.row_count;
            Ok("validated report-map placement for every active VCF expected-result row"
                .to_string())
        },
    );
}
