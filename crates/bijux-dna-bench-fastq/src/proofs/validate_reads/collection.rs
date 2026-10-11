use super::assessment::build_fastq_validate_reads_ready_row;
use super::binding::binding_key;
use super::evaluation_inputs::RowInputs;
use super::source_inputs::AdmittedBindings;
use super::source_inputs::EvaluatedRows;
use crate::proofs::inputs::ValidationContext;
use anyhow::anyhow;
use anyhow::Result;
pub(super) fn collect(
    context: &ValidationContext<'_>,
    bindings: AdmittedBindings,
) -> Result<EvaluatedRows> {
    let AdmittedBindings {
        active_rows,
        output_by_binding,
        parser_by_binding,
        expected_by_binding,
        command_by_binding,
        report_map_by_binding,
        comparable_row,
        smoke,
    } = bindings;
    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in active_rows {
        let key = binding_key(&active_row.stage_id, &active_row.tool_id);
        let output_row = output_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing output-contract row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let parser_row = parser_by_binding.get(&key).ok_or_else(|| {
            anyhow!("missing parser row for `{}` / `{}`", active_row.stage_id, active_row.tool_id)
        })?;
        let expected_row = expected_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing expected-result row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let command_row = command_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing rendered-command row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let report_map_row = report_map_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing report-map row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        rows.push(build_fastq_validate_reads_ready_row(
            context,
            RowInputs {
                active_row,
                output_row,
                parser_row,
                expected_row,
                command_row,
                comparable_row: &comparable_row,
                report_map_row,
                smoke: &smoke,
            },
        ));
    }

    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    Ok(EvaluatedRows { rows, smoke })
}
