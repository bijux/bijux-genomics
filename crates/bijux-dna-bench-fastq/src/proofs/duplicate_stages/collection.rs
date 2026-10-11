use super::assessment::build_fastq_duplicate_stages_ready_row;
use super::binding::binding_key;
use super::contracts::duplicate_stage_spec;
use super::evaluation_inputs::RowInputs;
use super::models::FastqDuplicateStagesReadyRow;
use super::source_inputs::AdmittedBindings;
use crate::proofs::inputs::SchemaContext;
use anyhow::anyhow;
use anyhow::Result;
pub(super) fn collect(
    context: &SchemaContext<'_>,
    bindings: AdmittedBindings,
) -> Result<Vec<FastqDuplicateStagesReadyRow>> {
    let AdmittedBindings {
        active_rows,
        output_by_binding,
        parser_by_binding,
        expected_by_binding,
        command_by_binding,
        report_map_by_binding,
        schema_report,
        schema_contracts,
    } = bindings;
    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in active_rows {
        let key = binding_key(&active_row.stage_id, &active_row.tool_id);
        let spec = duplicate_stage_spec(&active_row.stage_id)
            .ok_or_else(|| anyhow!("admitted duplicate stage has no governed specification"))?;
        let output_row = output_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing duplicate-stage output-contract row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let parser_row = parser_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing duplicate-stage parser row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let expected_row = expected_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing duplicate-stage expected-result row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let command_row = command_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing duplicate-stage rendered-command row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let report_map_row = report_map_by_binding.get(&key).ok_or_else(|| {
            anyhow!(
                "missing duplicate-stage report-map row for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        let schema_contract = schema_contracts.get(&active_row.stage_id).ok_or_else(|| {
            anyhow!("missing duplicate-stage schema contract for `{}`", active_row.stage_id)
        })?;

        rows.push(build_fastq_duplicate_stages_ready_row(
            context,
            RowInputs {
                active_row,
                output_row,
                parser_row,
                expected_row,
                command_row,
                report_map_row,
                schema_contract,
                spec,
                schema_proof_path: &schema_report.output_path,
            },
        ));
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });

    Ok(rows)
}
