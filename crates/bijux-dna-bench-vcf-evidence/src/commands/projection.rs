use crate::model::command_reports::{VcfRenderedCommandArgvRow, VcfRenderedCommandArgvStep};
use crate::model::commands::VcfRenderedCommandRow;
use anyhow::{Context, Result};

/// Project the admitted rows into the established executable script format.
#[must_use]
pub fn render_vcf_commands_shell_script(
    rows: &[VcfRenderedCommandRow],
    repo_root_relative_to_output: &str,
) -> String {
    let mut rendered = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\nrepo_root=\"$(cd \"$(dirname \"${{BASH_SOURCE[0]}}\")/{repo_root_relative_to_output}\" && pwd)\"\n"
    );
    rendered.push_str("cd \"$repo_root\"\n\n");
    for (index, row) in rows.iter().enumerate() {
        rendered.push_str("# ");
        rendered.push_str(&row.stage_id);
        rendered.push_str(" / ");
        rendered.push_str(&row.tool_id);
        rendered.push('\n');
        for command in &row.script_commands {
            rendered.push_str(command);
            rendered.push('\n');
        }
        if index + 1 < rows.len() {
            rendered.push('\n');
        }
    }
    rendered
}

/// Project the established structured argv wire records.
///
/// # Errors
/// Returns a serialization error if a row cannot be encoded.
pub fn render_vcf_command_argv_jsonl(rows: &[VcfRenderedCommandRow]) -> Result<String> {
    let mut rendered = String::new();
    for row in rows {
        let payload = VcfRenderedCommandArgvRow {
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            readiness_kind: row.readiness_kind.clone(),
            benchmark_status: row.benchmark_status.clone(),
            command_source: row.command_source.clone(),
            command_steps: row
                .command_steps
                .iter()
                .map(|step| VcfRenderedCommandArgvStep {
                    step_id: step.step_id.clone(),
                    step_kind: step.step_kind.clone(),
                    consumes_previous_stdout: step.consumes_previous_stdout,
                    argv: step.argv.clone(),
                })
                .collect(),
        };
        let line =
            serde_json::to_string(&payload).context("serialize VCF rendered command argv row")?;
        rendered.push_str(&line);
        rendered.push('\n');
    }
    Ok(rendered)
}

/// Quote each argument independently, including empty strings and shell metacharacters.
#[must_use]
pub fn render_shell_command(argv: &[String]) -> String {
    argv.iter()
        .map(|arg| {
            if !arg.is_empty()
                && arg.chars().all(|ch| {
                    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':')
                })
            {
                arg.clone()
            } else {
                let escaped = arg.replace('\'', "'\"'\"'");
                format!("'{escaped}'")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
