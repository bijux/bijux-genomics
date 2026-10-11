use crate::adapters::observations::VcfAdapterObservations;
use crate::evidence::{
    commands::{render_vcf_commands_with_observations, DEFAULT_VCF_RENDERED_COMMANDS_PATH},
    expectations::collect_vcf_expected_benchmark_result_rows as collect_vcf_expected_benchmark_result_rows_with_observations,
    output_coverage::collect_vcf_adapter_output_coverage_rows as collect_vcf_adapter_output_coverage_rows_with_observations,
    parsers::collect_vcf_parser_fixture_coverage_rows_from_retained,
    reports::collect_vcf_report_map_rows_from_expected,
    scope::collect_vcf_active_stage_tool_matrix_rows_from_expected,
};
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::{
    model::command_reports::VcfRenderedCommandsReport,
    snapshot::{ActiveVcfBinding, VcfEvidenceRows, VcfEvidenceSnapshot},
};
use std::path::Path;
/// Full active binding observations supplied by cross-domain composition.
pub type ActiveEvidenceProvider = fn(&Path) -> Result<Vec<ActiveVcfBinding>>;
/// Acquire and admit complete stage readiness evidence with one shared adapter context.
///
/// # Errors
/// Rejects source, coverage, publication or full binding-contract inconsistencies.
pub fn collect_vcf_readiness_evidence_with_observations(
    observations: &VcfAdapterObservations<'_>,
    run_id: &str,
    active_bindings: ActiveEvidenceProvider,
) -> Result<(VcfRenderedCommandsReport, VcfEvidenceSnapshot)> {
    let inputs = observations.inputs();
    let repo_root = observations.repo_root();
    let expected = collect_vcf_expected_benchmark_result_rows_with_observations(observations)?;
    let active = active_bindings(repo_root)?;
    let active_keys = active.iter().map(|row| row.key.clone()).collect::<Vec<_>>();
    let retained = collect_vcf_active_stage_tool_matrix_rows_from_expected(
        inputs,
        repo_root,
        &active_keys,
        &expected,
    )?;
    let command_report = render_vcf_commands_with_observations(
        repo_root,
        Path::new(DEFAULT_VCF_RENDERED_COMMANDS_PATH),
        observations,
        &retained,
    )?;
    let evidence = VcfEvidenceSnapshot::admit(VcfEvidenceRows {
        parsers: collect_vcf_parser_fixture_coverage_rows_from_retained(
            inputs, repo_root, &retained,
        )?
        .2,
        retained,
        active,
        commands: command_report.rows.clone(),
        outputs: collect_vcf_adapter_output_coverage_rows_with_observations(observations, run_id)?,
        reports: collect_vcf_report_map_rows_from_expected(inputs, expected.clone())?,
        expected,
    })?;
    Ok((command_report, evidence))
}
