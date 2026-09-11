use super::model::{BindingKey, FastqActiveStageToolMatrixRow};
use super::{BENCHMARK_READY_STATUS, DECLARED_ONLY_ADAPTER_STATUS, PLANNED_SUPPORT_STATUS};
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;

pub(crate) fn ensure_fastq_active_stage_tool_matrix_contract(
    rows: &[FastqActiveStageToolMatrixRow],
) -> Result<()> {
    let mut seen = BTreeSet::<BindingKey>::new();
    for row in rows {
        if !seen.insert(BindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() })
        {
            return Err(anyhow!(
                "FASTQ active-stage-tool matrix contains duplicate `{}` / `{}`",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.benchmark_status != BENCHMARK_READY_STATUS {
            return Err(anyhow!(
                "FASTQ active-stage-tool matrix row `{}` / `{}` must stay benchmark_ready",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.support_status == PLANNED_SUPPORT_STATUS
            || row.adapter_status == DECLARED_ONLY_ADAPTER_STATUS
            || row.corpus_status == "planner_only"
        {
            return Err(anyhow!(
                "FASTQ active-stage-tool matrix row `{}` / `{}` leaked a non-active readiness status",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.adapter_id.trim().is_empty()
            || row.parser_id.trim().is_empty()
            || row.schema_id.trim().is_empty()
        {
            return Err(anyhow!(
                "FASTQ active-stage-tool matrix row `{}` / `{}` contains a blank required column",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}
