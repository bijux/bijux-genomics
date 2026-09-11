use super::declarations::BENCHMARK_READY_STATUS;
use super::declarations::COMMAND_SOURCE_FASTQ_BAM_ADAPTER;
use super::models::BindingKey;
use super::models::FastqRenderedCommandRow;
use super::projection::binding_key;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeSet;
pub(super) fn ensure_fastq_rendered_command_contract(
    rows: &[FastqRenderedCommandRow],
) -> Result<()> {
    let mut seen_bindings = BTreeSet::<BindingKey>::new();
    for row in rows {
        if row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.adapter_id.trim().is_empty()
            || row.parser_id.trim().is_empty()
            || row.schema_id.trim().is_empty()
            || row.readiness_kind.trim().is_empty()
            || row.benchmark_status.trim().is_empty()
            || row.command_source.trim().is_empty()
        {
            return Err(anyhow!(
                "FASTQ rendered command row `{}` / `{}` is missing required columns",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.benchmark_status != BENCHMARK_READY_STATUS {
            return Err(anyhow!(
                "FASTQ rendered command row `{}` / `{}` drifted from the benchmark-ready slice",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.command_source != COMMAND_SOURCE_FASTQ_BAM_ADAPTER {
            return Err(anyhow!(
                "FASTQ rendered command row `{}` / `{}` drifted from the FASTQ/BAM adapter source",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.command_steps.len() != 1 || row.script_commands.len() != 1 {
            return Err(anyhow!(
                "FASTQ rendered command row `{}` / `{}` must preserve exactly one command step and one shell command",
                row.stage_id,
                row.tool_id
            ));
        }
        let step = &row.command_steps[0];
        if step.step_id != "invoke" || step.step_kind != "exec" || step.argv.is_empty() {
            return Err(anyhow!(
                "FASTQ rendered command row `{}` / `{}` emitted an invalid invoke step",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.script_commands[0].trim().is_empty() {
            return Err(anyhow!(
                "FASTQ rendered command row `{}` / `{}` emitted an empty shell command",
                row.stage_id,
                row.tool_id
            ));
        }
        if !seen_bindings.insert(binding_key(&row.stage_id, &row.tool_id)) {
            return Err(anyhow!(
                "FASTQ rendered commands contain duplicate `{}` / `{}` bindings",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}
