use super::VcfLocalContainerSmokeRow;
use anyhow::{anyhow, Result};
use std::{collections::BTreeSet, path::Path};
pub(super) fn ensure_vcf_local_container_smoke_contract(
    repo_root: &Path,
    rows: &[VcfLocalContainerSmokeRow],
    retained_bindings: &BTreeSet<(String, String)>,
    host_stage_matrix_row_count: usize,
) -> Result<()> {
    let emitted_bindings =
        rows.iter().map(|row| (row.stage_id.clone(), row.tool_id.clone())).collect::<BTreeSet<_>>();
    if rows.len() != retained_bindings.len() || emitted_bindings != *retained_bindings {
        return Err(anyhow!(
            "VCF local-container smoke bindings drifted from the governed retained stage-tool matrix (expected {retained_bindings:?}, found {emitted_bindings:?})"
        ));
    }
    let host_stage_smoke_row_count =
        rows.iter().filter(|row| row.smoke_path_kind == "host_stage_smoke").count();
    if host_stage_smoke_row_count != host_stage_matrix_row_count {
        return Err(anyhow!(
            "VCF local-container smoke host wrapper count drifted from the governed VCF stage matrix (expected {host_stage_matrix_row_count}, found {host_stage_smoke_row_count})"
        ));
    }

    for super::governed::SmokeBindingContract {
        stage_id,
        tool_id,
        registered_binary,
        smoke_path_kind,
        smoke_runtime,
        smoke_tool_id,
        smoke_command,
        smoke_support_path,
        smoke_minimal_cmd,
    } in super::governed::EXPECTED_SMOKE_BINDINGS
    {
        let row = rows
            .iter()
            .find(|row| row.stage_id == stage_id && row.tool_id == tool_id)
            .ok_or_else(|| {
                anyhow!("VCF local-container smoke report is missing `{stage_id}` / `{tool_id}`")
            })?;
        if row.registered_binary != registered_binary
            || row.smoke_path_kind != smoke_path_kind
            || row.smoke_runtime != smoke_runtime
            || row.smoke_tool_id != smoke_tool_id
            || row.smoke_command != smoke_command
            || row.smoke_support_path != smoke_support_path
            || row.smoke_minimal_cmd != smoke_minimal_cmd
        {
            return Err(anyhow!(
                "VCF local-container smoke row `{stage_id}` / `{tool_id}` drifted from the governed wrapper contract"
            ));
        }
        if !repo_root.join(&row.smoke_support_path).exists() {
            return Err(anyhow!(
                "VCF local-container smoke support path `{}` no longer exists",
                row.smoke_support_path
            ));
        }
    }

    if rows.iter().any(|row| row.smoke_command.trim().is_empty()) {
        return Err(anyhow!("VCF local-container smoke report must not emit empty smoke commands"));
    }

    Ok(())
}
