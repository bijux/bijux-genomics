use super::{
    runtime::{local_smoke_command_name, resolve_container_smoke_wrapper},
    VcfLocalContainerSmokeRow,
};
use crate::evidence::registry::smoke::VcfRegistrySmokeRecord;
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow;
use std::path::Path;
pub(super) fn external_row(
    retained_row: &VcfActiveStageToolMatrixRow,
    registry_row: &VcfRegistrySmokeRecord,
) -> VcfLocalContainerSmokeRow {
    VcfLocalContainerSmokeRow {
            stage_id: retained_row.stage_id.clone(),
            tool_id: retained_row.tool_id.clone(),
            registered_binary: registry_row.registered_binary.clone(),
            tool_status: retained_row.tool_status.clone(),
            stage_support_status: retained_row.stage_support_status.clone(),
            scope_state: retained_row.scope_state.clone(),
            scope_detail: retained_row.scope_detail.clone(),
            smoke_path_kind: "external_artifact_admission".to_string(),
            smoke_runtime: "external".to_string(),
            smoke_tool_id: registry_row.registered_binary.clone(),
            smoke_command: format!("{} --version", registry_row.registered_binary),
            smoke_support_path: "containers/contracts/catalog.toml".to_string(),
            smoke_minimal_cmd: registry_row.smoke_minimal_cmd.clone(),
            reason: format!(
                "binding `{}` / `{}` requires checksum-admitted operator-provided artifact evidence; no redistributable container wrapper is authorized",
                retained_row.stage_id, retained_row.tool_id
            ),
        }
}
pub(super) fn host_row(
    retained_row: &VcfActiveStageToolMatrixRow,
    registry_row: &VcfRegistrySmokeRecord,
    smoke_support_path: String,
) -> Result<VcfLocalContainerSmokeRow> {
    let smoke_command = format!(
        "bijux-dna bench local {} --tool-id {}",
        local_smoke_command_name(&retained_row.stage_id)?,
        retained_row.tool_id
    );
    Ok(VcfLocalContainerSmokeRow {
                stage_id: retained_row.stage_id.clone(),
                tool_id: retained_row.tool_id.clone(),
                registered_binary: registry_row.registered_binary.clone(),
                tool_status: retained_row.tool_status.clone(),
                stage_support_status: retained_row.stage_support_status.clone(),
                scope_state: retained_row.scope_state.clone(),
                scope_detail: retained_row.scope_detail.clone(),
                smoke_path_kind: "host_stage_smoke".to_string(),
                smoke_runtime: "host".to_string(),
                smoke_tool_id: retained_row.tool_id.clone(),
                smoke_command,
                smoke_support_path,
                smoke_minimal_cmd: String::new(),
                reason: format!(
                    "binding `{}` / `{}` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host",
                    retained_row.stage_id, retained_row.tool_id
                ),
            })
}
pub(super) fn container_row(
    repo_root: &Path,
    retained_row: &VcfActiveStageToolMatrixRow,
    registry_row: &VcfRegistrySmokeRecord,
    is_governed_default: bool,
) -> Result<VcfLocalContainerSmokeRow> {
    let (smoke_path_kind, smoke_runtime, smoke_command, smoke_support_path) =
        resolve_container_smoke_wrapper(repo_root, registry_row)?;
    let alias_clause = if is_governed_default {
        format!(
            "binding `{}` / `{}` matches the governed VCF stage-matrix default tool, but no exact tiny-fixture stage smoke wrapper is checked in",
            retained_row.stage_id, retained_row.tool_id
        )
    } else if registry_row.registered_binary != retained_row.tool_id {
        format!(
            "retained tool `{}` resolves through registered binary `{}`",
            retained_row.tool_id, registry_row.registered_binary
        )
    } else {
        format!(
            "retained tool `{}` has no exact tiny-fixture stage smoke wrapper",
            retained_row.tool_id
        )
    };
    let rationale_clause = if registry_row.smoke_minimal_rationale.trim().is_empty() {
        String::new()
    } else {
        format!("; {}", registry_row.smoke_minimal_rationale.trim())
    };

    Ok(VcfLocalContainerSmokeRow {
        stage_id: retained_row.stage_id.clone(),
        tool_id: retained_row.tool_id.clone(),
        registered_binary: registry_row.registered_binary.clone(),
        tool_status: retained_row.tool_status.clone(),
        stage_support_status: retained_row.stage_support_status.clone(),
        scope_state: retained_row.scope_state.clone(),
        scope_detail: retained_row.scope_detail.clone(),
        smoke_path_kind,
        smoke_runtime,
        smoke_tool_id: registry_row.registered_binary.clone(),
        smoke_command,
        smoke_support_path,
        smoke_minimal_cmd: registry_row.smoke_minimal_cmd.clone(),
        reason: format!(
            "{alias_clause}, so the governed container smoke wrapper is the available local exercise path for `{}` / `{}`{rationale_clause}",
            retained_row.stage_id, retained_row.tool_id
        ),
    })
}
