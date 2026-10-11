//! Owner: recorded benchmark command diagnostics and report reconciliation.
//! Consumes typed observations without filesystem or process access.
mod rows;
mod shell;
use crate::{
    AllDomainNoPlaceholderCommandCheckReport, AllDomainRenderedCommandsReport,
    ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_SCHEMA_VERSION,
};
use anyhow::{anyhow, ensure, Result};
use rows::audit_row;
use std::collections::{BTreeMap, BTreeSet};

/// Diagnose recorded command projections; a clean result is not scientific execution evidence.
///
/// # Errors
/// Rejects malformed step identities and inconsistent derived report counts.
pub fn evaluate_recorded_command_audit(
    rendered_report: &AllDomainRenderedCommandsReport,
    output_path: String,
) -> Result<AllDomainNoPlaceholderCommandCheckReport> {
    ensure!(
        rendered_report.schema_version == crate::ALL_DOMAIN_RENDERED_COMMANDS_SCHEMA_VERSION,
        "unsupported rendered command observation schema"
    );
    ensure!(
        !rendered_report.rows.is_empty() && rendered_report.row_count == rendered_report.rows.len(),
        "recorded command audit requires a complete nonempty row set"
    );
    let identities = rendered_report.rows.iter().map(|row| &row.result_id).collect::<BTreeSet<_>>();
    ensure!(
        identities.len() == rendered_report.rows.len(),
        "recorded command audit repeats a result identity"
    );
    let mut rows = rendered_report.rows.iter().map(audit_row).collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let row_count = rows.len();
    let command_step_count = rows.iter().map(|row| row.command_step_count).sum::<usize>();
    let shell_wrapped_step_count = rows
        .iter()
        .flat_map(|row| row.step_audits.iter())
        .filter(|step| step.is_shell_wrapper)
        .count();
    let direct_step_count = command_step_count.saturating_sub(shell_wrapped_step_count);
    let valid_row_count = rows.iter().filter(|row| row.ok).count();
    let invalid_row_count = row_count.saturating_sub(valid_row_count);
    let result_id_count =
        rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
    let stage_count = rows
        .iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
        .collect::<BTreeSet<_>>()
        .len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut command_source_counts = BTreeMap::<String, usize>::new();
    let mut finding_type_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *command_source_counts.entry(row.command_source.clone()).or_default() += 1;
        for finding_type in &row.finding_types {
            *finding_type_counts.entry(finding_type.clone()).or_default() += 1;
        }
    }
    let violations = rows.iter().filter(|row| !row.ok).cloned().collect::<Vec<_>>();

    let report = AllDomainNoPlaceholderCommandCheckReport {
        schema_version: ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_SCHEMA_VERSION.to_string(),
        output_path,
        script_output_path: rendered_report.output_path.clone(),
        argv_output_path: rendered_report.argv_output_path.clone(),
        row_count,
        result_id_count,
        stage_count,
        tool_count,
        command_step_count,
        shell_wrapped_step_count,
        direct_step_count,
        valid_row_count,
        invalid_row_count,
        domain_counts,
        command_source_counts,
        finding_type_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };
    ensure_all_domain_no_placeholder_command_check_contract(&report)?;
    Ok(report)
}

fn ensure_all_domain_no_placeholder_command_check_contract(
    report: &AllDomainNoPlaceholderCommandCheckReport,
) -> Result<()> {
    if report.row_count != report.rows.len() {
        return Err(anyhow!(
            "all-domain no-placeholder command check report drifted from its row set"
        ));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!(
            "all-domain no-placeholder command check report drifted from its violation set"
        ));
    }
    if report.valid_row_count + report.invalid_row_count != report.row_count {
        return Err(anyhow!(
            "all-domain no-placeholder command check row counts no longer reconcile"
        ));
    }
    let actual_step_count = report.rows.iter().map(|row| row.command_step_count).sum::<usize>();
    if report.command_step_count != actual_step_count {
        return Err(anyhow!(
            "all-domain no-placeholder command check step count drifted from its rows"
        ));
    }
    if report.shell_wrapped_step_count + report.direct_step_count != report.command_step_count {
        return Err(anyhow!(
            "all-domain no-placeholder command check shell/direct step counts no longer reconcile"
        ));
    }
    if report.ok && report.violation_count != 0 {
        return Err(anyhow!(
            "all-domain no-placeholder command check cannot be ok while violations remain"
        ));
    }
    if !report.ok && report.violation_count == 0 {
        return Err(anyhow!(
            "all-domain no-placeholder command check must keep explicit violations when failing"
        ));
    }
    for row in &report.rows {
        if row.result_id.trim().is_empty()
            || row.domain.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.command_source.trim().is_empty()
        {
            return Err(anyhow!(
                "all-domain no-placeholder command check row is missing required identity fields"
            ));
        }
        if row.command_step_count != row.step_audits.len() {
            return Err(anyhow!(
                "all-domain no-placeholder command row `{}` drifted from its step audit set",
                row.result_id
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/contracts/command_audit.rs"]
mod tests;
