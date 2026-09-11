use super::super::keys::binding_key_from_matrix_row;
use super::super::{
    model::{
        VcfAllRetainedToolsCompleteGoalCheck, VcfBindingKey, EXPECTED_ACTIVE_STAGE_COUNT,
        EXPECTED_RETAINED_ROW_COUNT, EXPECTED_RETAINED_STAGE_COUNT, EXPECTED_RETAINED_TOOL_COUNT,
    },
    ports::VcfCompletionPorts,
    state::CompletionState,
};
use super::record_goal_check;
use anyhow::bail;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;
pub(super) fn retained_scope(
    repo_root: &Path,
    ports: &VcfCompletionPorts,
    state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        336,
        "vcf active stage tool matrix",
        Some(crate::evidence::scope::DEFAULT_VCF_ACTIVE_STAGE_TOOL_MATRIX_PATH.to_string()),
        || {
            let report = (ports.evidence.matrix)(
                repo_root,
                PathBuf::from(crate::evidence::scope::DEFAULT_VCF_ACTIVE_STAGE_TOOL_MATRIX_PATH),
            )?;
            if report.row_count != EXPECTED_RETAINED_ROW_COUNT
                || report.stage_count != EXPECTED_RETAINED_STAGE_COUNT
                || report.tool_count != EXPECTED_RETAINED_TOOL_COUNT
                || report.complete_row_count != 0
                || report.active_row_count + report.complete_row_count + report.removed_row_count
                    != report.row_count
            {
                bail!(
                    "VCF retained matrix drifted: rows={}, stages={}, tools={}, active={}, complete={}, removed={}",
                    report.row_count,
                    report.stage_count,
                    report.tool_count,
                    report.active_row_count,
                    report.complete_row_count,
                    report.removed_row_count
                );
            }

            state.retained_bindings =
                report.rows.iter().map(binding_key_from_matrix_row).collect::<BTreeSet<_>>();
            state.active_bindings = report
                .rows
                .iter()
                .filter(|row| row.scope_state == "active")
                .map(binding_key_from_matrix_row)
                .collect::<BTreeSet<_>>();
            for required in [
                VcfBindingKey { stage_id: "vcf.call_gl".to_string(), tool_id: "angsd".to_string() },
                VcfBindingKey {
                    stage_id: "vcf.call_pseudohaploid".to_string(),
                    tool_id: "angsd".to_string(),
                },
                VcfBindingKey {
                    stage_id: "vcf.imputation_metrics".to_string(),
                    tool_id: "impute5".to_string(),
                },
                VcfBindingKey {
                    stage_id: "vcf.impute".to_string(),
                    tool_id: "impute5".to_string(),
                },
            ] {
                if !state.active_bindings.contains(&required) {
                    bail!(
                        "VCF retained matrix is missing supported licensed binding `{}` / `{}`",
                        required.stage_id,
                        required.tool_id
                    );
                }
            }
            state.retained_row = report.row_count;
            state.retained_stage = report.stage_count;
            state.retained_tool = report.tool_count;
            state.active_row = report.active_row_count;
            state.removed_row = report.removed_row_count;
            state.active_stage = report
                .rows
                .iter()
                .filter(|row| row.scope_state == "active")
                .map(|row| row.stage_id.as_str())
                .collect::<BTreeSet<_>>()
                .len();
            state.active_tool = report
                .rows
                .iter()
                .filter(|row| row.scope_state == "active")
                .map(|row| row.tool_id.as_str())
                .collect::<BTreeSet<_>>()
                .len();
            if state.active_stage != EXPECTED_ACTIVE_STAGE_COUNT {
                bail!(
                    "VCF active retained matrix drifted: active_stages={active_stage}, active_tools={active_tool}"
                , active_stage=state.active_stage,active_tool=state.active_tool);
            }
            Ok(format!(
                "validated the governed {retained_row}-row retained VCF matrix with {active_row} active bindings across {active_stage} active stages"
            , retained_row=state.retained_row,active_row=state.active_row,active_stage=state.active_stage))
        },
    );
}
