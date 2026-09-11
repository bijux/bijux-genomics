use std::path::{Path, PathBuf};

use anyhow::Result;

use super::all_domain_active_stage_tool_matrix::collect_all_domain_active_stage_tool_matrix_rows;
use super::vcf_active_stage_tool_matrix::collect_vcf_active_stage_tool_matrix_rows;
use super::vcf_adapter_output_coverage::collect_vcf_adapter_output_coverage_rows;
use super::vcf_expected_benchmark_results::collect_vcf_expected_benchmark_result_rows;
use super::vcf_parser_fixture_coverage::collect_vcf_parser_fixture_coverage_rows;
use super::vcf_rendered_commands::{
    render_vcf_commands, VcfRenderedCommandsReport, DEFAULT_VCF_RENDERED_COMMANDS_PATH,
};
use super::vcf_report_map::collect_vcf_report_map_rows;

pub(crate) use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBinding;
use bijux_dna_bench_vcf_evidence::snapshot::{
    ActiveVcfBinding, VcfEvidenceRows, VcfEvidenceSnapshot, VcfStageReadinessBindingKey,
};

pub(crate) fn collect_vcf_stage_readiness_bindings(
    repo_root: &Path,
    stage_id: &str,
) -> Result<(VcfRenderedCommandsReport, Vec<VcfStageReadinessBinding>)> {
    let retained = collect_vcf_active_stage_tool_matrix_rows(repo_root)?;
    let active = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| row.domain == "vcf")
        .map(|row| ActiveVcfBinding {
            key: VcfStageReadinessBindingKey {
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                corpus_id: row.corpus_id,
                asset_profile_id: row.asset_profile_id,
            },
            adapter_id: row.adapter_id,
            parser_id: row.parser_id,
            schema_id: row.schema_id,
        })
        .collect();
    let command_report =
        render_vcf_commands(repo_root, PathBuf::from(DEFAULT_VCF_RENDERED_COMMANDS_PATH))?;
    let evidence = VcfEvidenceSnapshot::admit(VcfEvidenceRows {
        retained,
        active,
        commands: command_report.rows.clone(),
        outputs: collect_vcf_adapter_output_coverage_rows(repo_root)?,
        parsers: collect_vcf_parser_fixture_coverage_rows(repo_root)?.2,
        expected: collect_vcf_expected_benchmark_result_rows(repo_root)?,
        reports: collect_vcf_report_map_rows(repo_root)?,
    })?;
    Ok((command_report, evidence.stage_bindings(stage_id)?.to_vec()))
}
