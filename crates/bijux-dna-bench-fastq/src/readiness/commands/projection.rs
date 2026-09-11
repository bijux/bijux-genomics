use super::contracts::ensure_fastq_rendered_command_contract;
use super::declarations::BENCHMARK_READY_STATUS;
use super::declarations::COMMAND_SOURCE_FASTQ_BAM_ADAPTER;
use super::models::BindingKey;
use super::models::FastqRenderedCommandRow;
use super::models::FastqRenderedCommandStep;
use crate::readiness::active::FastqActiveStageToolMatrixRow;
use crate::readiness::declarations::observations::RenderedCommandObservation as RenderedCommandRow;
use crate::readiness::declarations::CommandContext;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_core::id_catalog::FASTQ_PREFIX;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
/// Collect canonical FASTQ declaration evidence.
///
/// # Errors
/// Returns source admission, contract or publication failure.
pub fn collect_fastq_rendered_command_rows(
    context: &CommandContext<'_>,
) -> Result<Vec<FastqRenderedCommandRow>> {
    let repo_root = context.repo_root;
    let active_rows = context.evidence.active(repo_root)?.rows;
    let rendered_by_binding = context
        .evidence
        .rendered(repo_root)?
        .into_iter()
        .filter(|row| row.stage_id.starts_with(FASTQ_PREFIX))
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(active_rows.len());
    let mut seen_bindings = BTreeSet::new();
    for active_row in active_rows {
        let binding = binding_key(&active_row.stage_id, &active_row.tool_id);
        let rendered_row = rendered_by_binding.get(&binding).ok_or_else(|| {
            anyhow!(
                "FASTQ rendered commands are missing benchmark-ready command coverage for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        if !seen_bindings.insert(binding.clone()) {
            return Err(anyhow!(
                "FASTQ rendered commands contain duplicate `{}` / `{}` bindings",
                active_row.stage_id,
                active_row.tool_id
            ));
        }
        rows.push(rendered_fastq_row(active_row, rendered_row));
    }

    if rendered_by_binding.len() != rows.len() {
        let active_bindings = rows
            .iter()
            .map(|row| binding_key(&row.stage_id, &row.tool_id))
            .collect::<BTreeSet<_>>();
        let unexpected = rendered_by_binding
            .keys()
            .filter(|binding| !active_bindings.contains(*binding))
            .map(|binding| format!("{}/{}", binding.stage_id, binding.tool_id))
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "FASTQ rendered commands must keep exactly one row per active FASTQ binding, unexpected rows: {}",
            unexpected.join(", ")
        ));
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_fastq_rendered_command_contract(&rows)?;
    Ok(rows)
}

pub(super) fn rendered_fastq_row(
    active_row: FastqActiveStageToolMatrixRow,
    rendered_row: &RenderedCommandRow,
) -> FastqRenderedCommandRow {
    let step = FastqRenderedCommandStep {
        step_id: "invoke".to_string(),
        step_kind: "exec".to_string(),
        consumes_previous_stdout: false,
        argv: rendered_row.argv.clone(),
    };
    FastqRenderedCommandRow {
        stage_id: active_row.stage_id,
        tool_id: active_row.tool_id,
        corpus_id: active_row.corpus_id,
        asset_profile_id: active_row.asset_profile_id,
        adapter_id: active_row.adapter_id,
        parser_id: active_row.parser_id,
        schema_id: active_row.schema_id,
        readiness_kind: rendered_row.readiness_kind.clone(),
        benchmark_status: BENCHMARK_READY_STATUS.to_string(),
        command_source: COMMAND_SOURCE_FASTQ_BAM_ADAPTER.to_string(),
        command_steps: vec![step],
        script_commands: vec![rendered_row.command.clone()],
    }
}

pub(super) fn binding_key(stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey { stage_id: stage_id.to_string(), tool_id: tool_id.to_string() }
}
