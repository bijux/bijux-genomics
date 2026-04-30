use super::Result;
use crate::request_args::{
    OperatorDiagnosisResponseV1, OutputFormatV1, RunBrowserResponseV1,
};

/// Render stable run-browser output in human or JSON form.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn render_run_browser_output(
    response: &RunBrowserResponseV1,
    format: OutputFormatV1,
) -> Result<String> {
    match format {
        OutputFormatV1::Json => Ok(String::from_utf8(
            bijux_dna_core::contract::canonical::to_canonical_json_bytes(response)?,
        )?),
        OutputFormatV1::Human => Ok(render_run_browser_human(response)),
    }
}

/// Render stable operator diagnosis output in human or JSON form.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn render_operator_diagnosis_output(
    response: &OperatorDiagnosisResponseV1,
    format: OutputFormatV1,
) -> Result<String> {
    match format {
        OutputFormatV1::Json => Ok(String::from_utf8(
            bijux_dna_core::contract::canonical::to_canonical_json_bytes(response)?,
        )?),
        OutputFormatV1::Human => Ok(render_operator_diagnosis_human(response)),
    }
}

fn render_run_browser_human(response: &RunBrowserResponseV1) -> String {
    let mut lines = vec![format!(
        "schema={} total_rows={} page_size={}",
        response.schema_version, response.total_rows, response.page_size
    )];
    for row in &response.rows {
        lines.push(format!(
            "run_id={} state={} mode={} failures={} artifacts={} dir={}",
            row.run_id,
            row.state
                .as_ref()
                .map_or_else(|| "unknown".to_string(), ToString::to_string),
            row.mode
                .as_ref()
                .map_or_else(|| "unknown".to_string(), ToString::to_string),
            row.has_failures,
            row.artifact_count,
            row.run_dir.display(),
        ));
    }
    lines.join("\n")
}

fn render_operator_diagnosis_human(response: &OperatorDiagnosisResponseV1) -> String {
    let mut lines = vec![format!(
        "schema={} run_id={} queue_state={} requested_action={} health_ok={} has_failure_record={}",
        response.schema_version,
        response.run_id,
        response
            .queue_state
            .as_ref()
            .map_or_else(|| "unknown".to_string(), ToString::to_string),
        response
            .requested_action
            .as_ref()
            .map_or_else(|| "none".to_string(), ToString::to_string),
        response.health_ok,
        response.has_failure_record,
    )];
    for command in &response.commands {
        lines.push(format!(
            "command={} argv={} purpose={}",
            command.command_id,
            command.argv.join(" "),
            command.purpose
        ));
    }
    lines.join("\n")
}
