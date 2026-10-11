use anyhow::Result;
use bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations;
pub(crate) use bijux_dna_bench_vcf::evidence::commands::collect_vcf_rendered_command_rows_with_observations;
pub(crate) use bijux_dna_bench_vcf_evidence::model::commands::{
    VcfRenderedCommandRow, VcfRenderedCommandStep,
};
use std::{collections::BTreeMap, path::Path};
pub(crate) fn collect_vcf_rendered_command_rows(
    repo_root: &Path,
) -> Result<Vec<VcfRenderedCommandRow>> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    let expected=super::vcf_expected_benchmark_results::collect_vcf_expected_benchmark_result_rows_with_observations(&observations)?;
    let retained=super::vcf_active_stage_tool_matrix::collect_vcf_active_stage_tool_matrix_rows_from_expected(repo_root, &inputs, expected)?;
    collect_vcf_rendered_command_rows_with_observations(&observations, &retained)
}
pub(crate) fn collect_vcf_adapter_command_row_map(
    repo_root: &Path,
) -> Result<BTreeMap<(String, String), VcfRenderedCommandRow>> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    bijux_dna_bench_vcf::evidence::commands::collect_vcf_adapter_command_row_map(&observations)
}
