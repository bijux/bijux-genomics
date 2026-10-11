use super::super::ports::VcfAdapterReadinessPorts;
use super::super::state::ReadinessState;
use crate::adapters::observations::VcfAdapterObservations;
use crate::evidence::commands::{
    render_vcf_commands_with_observations, DEFAULT_VCF_RENDERED_COMMANDS_PATH,
};
use crate::evidence::expectations::collect_vcf_expected_benchmark_result_rows;
use crate::evidence::scope::collect_vcf_active_stage_tool_matrix_rows_from_expected;
use anyhow::{bail, Result};
use std::path::Path;
pub(super) fn rendered_commands(
    observations: &VcfAdapterObservations<'_>,
    ports: &VcfAdapterReadinessPorts,
    state: &mut ReadinessState,
) -> Result<String> {
    let repo_root = observations.repo_root();

    let expected = collect_vcf_expected_benchmark_result_rows(observations)?;
    let active = (ports.active_bindings)(repo_root)?;
    let retained = collect_vcf_active_stage_tool_matrix_rows_from_expected(
        observations.inputs(),
        repo_root,
        &active,
        &expected,
    )?;
    let report = render_vcf_commands_with_observations(
        repo_root,
        Path::new(DEFAULT_VCF_RENDERED_COMMANDS_PATH),
        observations,
        &retained,
    )?;
    let active_pair_count = retained.iter().filter(|row| row.scope_state == "active").count();
    if report.row_count != active_pair_count {
        bail!("VCF rendered commands drifted from the governed active VCF command slice");
    }
    let script_path = repo_root.join(&report.output_path);
    let argv_path = repo_root.join(&report.argv_output_path);
    super::super::syntax::validate_published_commands(
        repo_root,
        &script_path,
        &argv_path,
        report.row_count,
        ports.inspect_shell,
    )?;
    state.counts.rendered_commands = report.row_count;
    state.rendered_commands_report = Some(report);
    Ok("validated active VCF shell and argv command rendering with bash syntax coverage"
        .to_string())
}
