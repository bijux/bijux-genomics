use bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations;
use bijux_dna_bench_vcf_evidence::commands::admission::{
    ensure_unique_vcf_rendered_command_rows, validate_vcf_rendered_command_row,
};
pub(crate) use bijux_dna_bench_vcf_evidence::model::commands::{
    VcfRenderedCommandRow, VcfRenderedCommandStep,
};
use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Result};

pub(crate) fn collect_vcf_rendered_command_rows(
    repo_root: &Path,
) -> Result<Vec<VcfRenderedCommandRow>> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    let expected=super::vcf_expected_benchmark_results::collect_vcf_expected_benchmark_result_rows_with_observations(&observations)?;
    let retained=super::vcf_active_stage_tool_matrix::collect_vcf_active_stage_tool_matrix_rows_from_expected(repo_root, expected)?;
    collect_vcf_rendered_command_rows_with_observations(&observations, &retained)
}

pub(crate) fn collect_vcf_rendered_command_rows_with_observations(
    observations: &VcfAdapterObservations<'_>,
    retained: &[bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow],
) -> Result<Vec<VcfRenderedCommandRow>> {
    let canonical_pairs = retained
        .iter()
        .filter(|row| row.scope_state == "active")
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), "benchmark_ready".to_string()))
        .collect::<Vec<_>>();
    let adapter_rows =
        bijux_dna_bench_vcf::evidence::commands::collect_vcf_adapter_command_row_map(observations)?;

    let mut rows = Vec::with_capacity(canonical_pairs.len());
    for ((stage_id, tool_id), benchmark_status) in canonical_pairs {
        let row = adapter_rows
            .get(&(stage_id.clone(), tool_id.clone()))
            .cloned()
            .ok_or_else(|| {
                anyhow!(
                    "VCF rendered command rows are missing adapter command coverage for `{stage_id}` / `{tool_id}`"
                )
            })?;
        if row.command_steps.is_empty() {
            return Err(anyhow!(
                "VCF rendered command row `{stage_id}` / `{tool_id}` has no command steps"
            ));
        }
        if row.benchmark_status != benchmark_status {
            return Err(anyhow!(
                "VCF rendered command row `{stage_id}` / `{tool_id}` drifted from canonical benchmark status (`{}` vs `{}`)",
                row.benchmark_status,
                benchmark_status
            ));
        }
        validate_vcf_rendered_command_row(&row)?;
        rows.push(row);
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_unique_vcf_rendered_command_rows(&rows)?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::commands::benchmark::readiness::vcf_tool_serving_map::collect_vcf_tool_serving_map_rows;

    use super::collect_vcf_rendered_command_rows;

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn vcf_rendered_command_rows_follow_canonical_benchmark_ready_slice() {
        let rows = collect_vcf_rendered_command_rows(&repo_root())
            .expect("collect VCF rendered command rows");

        let benchmark_ready_row_count = collect_vcf_tool_serving_map_rows()
            .expect("collect VCF tool-serving rows")
            .into_iter()
            .filter(|row| row.benchmark_status == "benchmark_ready")
            .count();
        assert_eq!(rows.len(), benchmark_ready_row_count);
        assert!(rows.iter().all(|row| row.benchmark_status == "benchmark_ready"));
        assert!(rows.iter().all(|row| !row.script_commands.is_empty()));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.call" && row.tool_id == "bcftools"));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.stats" && row.tool_id == "bcftools"));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.impute" && row.tool_id == "beagle"));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.qc" && row.tool_id == "plink2"));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.phasing" && row.tool_id == "shapeit5"));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.phasing" && row.tool_id == "shapeit"));
    }
}

pub(crate) fn collect_vcf_adapter_command_row_map(
    repo_root: &Path,
) -> Result<BTreeMap<(String, String), VcfRenderedCommandRow>> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    bijux_dna_bench_vcf::evidence::commands::collect_vcf_adapter_command_row_map(&observations)
}
