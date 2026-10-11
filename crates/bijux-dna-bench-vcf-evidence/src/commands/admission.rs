use crate::model::commands::{VcfRenderedCommandRow, VcfRenderedCommandStep};
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;

/// Project ordered commands and their explicit stdout dependencies.
///
/// # Errors
/// Rejects empty command lists or a pipe without a preceding command.
pub fn render_script_commands(steps: &[VcfRenderedCommandStep]) -> Result<Vec<String>> {
    let mut commands = Vec::new();
    let mut current = String::new();
    let mut have_current = false;

    for step in steps {
        if step.argv.is_empty() {
            return Err(anyhow!(
                "VCF rendered command step `{}` has an empty argv vector",
                step.step_id
            ));
        }
        if step.consumes_previous_stdout {
            if !have_current {
                return Err(anyhow!(
                    "VCF rendered command step `{}` consumes previous stdout but no source command is open",
                    step.step_id
                ));
            }
            current.push_str(" | ");
            current.push_str(&step.command);
            continue;
        }
        if have_current {
            commands.push(current);
            current = String::new();
        }
        current.push_str(&step.command);
        have_current = true;
    }

    if have_current {
        commands.push(current);
    }
    if commands.is_empty() {
        return Err(anyhow!("VCF rendered command row produced no shell commands"));
    }
    Ok(commands)
}

/// Admit matching structured argv, shell commands and pipeline topology.
///
/// # Errors
/// Rejects ambiguous steps, invalid executables, placeholders and inconsistent projections.
pub fn validate_vcf_rendered_command_row(row: &VcfRenderedCommandRow) -> Result<()> {
    if row.command_steps.is_empty() {
        return Err(anyhow!(
            "VCF rendered command row `{}` / `{}` has no command steps",
            row.stage_id,
            row.tool_id
        ));
    }
    let mut step_ids = BTreeSet::new();
    for step in &row.command_steps {
        if step.step_id.is_empty()
            || step.step_id.trim() != step.step_id
            || !step_ids.insert(&step.step_id)
        {
            return Err(anyhow!(
                "VCF command has an empty, padded or duplicate step identity: `{}`",
                step.step_id
            ));
        }
        if step.argv.iter().any(|arg| arg.contains('\0')) {
            return Err(anyhow!("VCF command argv contains a NUL byte"));
        }
        if step.command != super::projection::render_shell_command(&step.argv) {
            return Err(anyhow!(
                "VCF command step `{}` shell projection disagrees with structured argv",
                step.step_id
            ));
        }

        let executable = step
            .argv
            .first()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                anyhow!(
                    "VCF rendered command row `{}` / `{}` step `{}` has an empty executable",
                    row.stage_id,
                    row.tool_id,
                    step.step_id
                )
            })?;
        let command_lower = step.command.to_ascii_lowercase();
        if command_lower.contains("todo") {
            return Err(anyhow!(
                "VCF rendered command row `{}` / `{}` step `{}` still contains a to-do marker",
                row.stage_id,
                row.tool_id,
                step.step_id
            ));
        }
        if command_lower.contains("placeholder") {
            return Err(anyhow!(
                "VCF rendered command row `{}` / `{}` step `{}` still contains placeholder text",
                row.stage_id,
                row.tool_id,
                step.step_id
            ));
        }
        if executable.eq_ignore_ascii_case("echo") && command_lower.contains("echo execute") {
            return Err(anyhow!(
                "VCF rendered command row `{}` / `{}` step `{}` still contains `echo execute`",
                row.stage_id,
                row.tool_id,
                step.step_id
            ));
        }
    }
    if render_script_commands(&row.command_steps)? != row.script_commands {
        return Err(anyhow!("VCF script commands disagree with structured step dependencies"));
    }
    Ok(())
}

/// Require one command authority per stage/tool pair.
///
/// # Errors
/// Rejects duplicate stage/tool command rows.
pub fn ensure_unique_vcf_rendered_command_rows(rows: &[VcfRenderedCommandRow]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for row in rows {
        let key = (row.stage_id.as_str(), row.tool_id.as_str());
        if !seen.insert(key) {
            return Err(anyhow!(
                "VCF rendered command rows encountered duplicate pair `{}` / `{}`",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}
