use crate::adapters::observations::VcfAdapterObservations;
use anyhow::{anyhow, Result};
use bijux_dna_bench_vcf_evidence::{
    commands::admission::{
        ensure_unique_vcf_rendered_command_rows, validate_vcf_rendered_command_row,
    },
    model::commands::VcfRenderedCommandRow,
};
/// Project complete active command coverage from admitted retained bindings.
///
/// # Errors
/// Rejects ambiguous binding identities before acquisition, or incomplete and invalid commands.
pub fn collect_vcf_rendered_command_rows_with_observations(
    observations: &VcfAdapterObservations<'_>,
    retained: &[bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow],
) -> Result<Vec<VcfRenderedCommandRow>> {
    super::admission::admit_retained(observations.inputs(), retained)?;
    let canonical_pairs = retained
        .iter()
        .filter(|row| row.scope_state == "active")
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), "benchmark_ready".to_string()))
        .collect::<Vec<_>>();
    let adapter_rows = super::collect_vcf_adapter_command_row_map(observations)?;

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
