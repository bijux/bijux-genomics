//! Evaluate recorded command steps without executing tools.
use super::shell::{
    direct_command_has_real_invocation, extract_shell_command_heads, is_echo_only_execution,
    is_shell_wrapper, is_unconditional_success, shell_command_has_real_invocation, shell_payload,
};
use crate::{
    AllDomainNoPlaceholderCommandRow, AllDomainNoPlaceholderCommandStepAudit,
    AllDomainRenderedCommandRow, AllDomainRenderedCommandStep,
};
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
const FINDING_TO_DO_MARKER: &str = "contains_todo_marker";
const FINDING_PLACEHOLDER: &str = "contains_placeholder";
const FINDING_ECHO_ONLY_EXECUTION: &str = "echo_only_execution";
const FINDING_UNCONDITIONAL_SUCCESS: &str = "unconditional_success";
const FINDING_EMPTY_EXECUTABLE: &str = "empty_executable";
const FINDING_MISSING_REAL_INVOCATION: &str = "missing_real_invocation";

pub(super) fn audit_row(
    row: &AllDomainRenderedCommandRow,
) -> Result<AllDomainNoPlaceholderCommandRow> {
    if row.command_steps.is_empty() {
        return Err(anyhow!("recorded command audit requires steps for `{}`", row.result_id));
    }
    let step_audits =
        row.command_steps.iter().map(|step| audit_step(row, step)).collect::<Result<Vec<_>>>()?;
    let mut finding_types = step_audits
        .iter()
        .flat_map(|step| step.finding_types.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    finding_types.sort();

    Ok(AllDomainNoPlaceholderCommandRow {
        result_id: row.result_id.clone(),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
        command_source: row.command_source.clone(),
        command_step_count: row.command_steps.len(),
        script_command_count: row.script_commands.len(),
        has_real_invocation: step_audits.iter().all(|step| step.has_real_invocation),
        finding_count: step_audits.iter().map(|step| step.finding_types.len()).sum(),
        finding_types,
        ok: step_audits.iter().all(|step| step.ok),
        step_audits,
    })
}

fn audit_step(
    row: &AllDomainRenderedCommandRow,
    step: &AllDomainRenderedCommandStep,
) -> Result<AllDomainNoPlaceholderCommandStepAudit> {
    let executable = step.argv.first().map(|value| value.trim().to_string()).unwrap_or_default();
    let is_shell_wrapper = is_shell_wrapper(&executable);
    let command_heads = if is_shell_wrapper {
        extract_shell_command_heads(shell_payload(&step.argv))
    } else {
        Vec::new()
    };
    let has_real_invocation = if executable.is_empty() {
        false
    } else if is_shell_wrapper {
        shell_command_has_real_invocation(&command_heads)
    } else {
        direct_command_has_real_invocation(&executable)
    };

    let mut finding_types = Vec::new();
    let lowered_command = step.command.to_ascii_lowercase();
    let lowered_argv = step.argv.iter().map(|value| value.to_ascii_lowercase()).collect::<Vec<_>>();
    if executable.is_empty() {
        finding_types.push(FINDING_EMPTY_EXECUTABLE.to_string());
    }
    if lowered_command.contains("todo") || lowered_argv.iter().any(|value| value.contains("todo")) {
        finding_types.push(FINDING_TO_DO_MARKER.to_string());
    }
    if lowered_command.contains("placeholder")
        || lowered_argv.iter().any(|value| value.contains("placeholder"))
    {
        finding_types.push(FINDING_PLACEHOLDER.to_string());
    }
    if is_echo_only_execution(&executable, shell_payload(&step.argv), &command_heads) {
        finding_types.push(FINDING_ECHO_ONLY_EXECUTION.to_string());
    }
    if is_unconditional_success(
        &executable,
        shell_payload(&step.argv),
        &command_heads,
        &lowered_command,
    ) {
        finding_types.push(FINDING_UNCONDITIONAL_SUCCESS.to_string());
    }
    if !has_real_invocation {
        finding_types.push(FINDING_MISSING_REAL_INVOCATION.to_string());
    }
    finding_types.sort();
    finding_types.dedup();

    if step.step_id.trim().is_empty() || step.step_kind.trim().is_empty() {
        return Err(anyhow!(
            "all-domain no-placeholder command check encountered a malformed step in `{}`",
            row.result_id
        ));
    }

    Ok(AllDomainNoPlaceholderCommandStepAudit {
        step_id: step.step_id.clone(),
        step_kind: step.step_kind.clone(),
        executable,
        is_shell_wrapper,
        command_heads,
        has_real_invocation,
        ok: finding_types.is_empty(),
        finding_types,
    })
}
