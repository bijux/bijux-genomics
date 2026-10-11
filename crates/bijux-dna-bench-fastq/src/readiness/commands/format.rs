use super::models::FastqRenderedCommandArgvRow;
use super::models::FastqRenderedCommandRow;
use anyhow::Context;
use anyhow::Result;
use std::fmt::Write as _;
pub(super) fn render_fastq_commands_shell_script(
    rows: &[FastqRenderedCommandRow],
    repo_root_relative_to_output: &str,
) -> String {
    let mut rendered = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    let _ = writeln!(rendered,"repo_root=\"$(cd \"$(dirname \"${{BASH_SOURCE[0]}}\")/{repo_root_relative_to_output}\" && pwd)\"");
    rendered.push_str("cd \"$repo_root\"\n\n");
    for (index, row) in rows.iter().enumerate() {
        let _ = writeln!(rendered, "# {} / {}", row.stage_id, row.tool_id);
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

pub(super) fn render_fastq_command_argv_jsonl(rows: &[FastqRenderedCommandRow]) -> Result<String> {
    let mut rendered = String::new();
    for row in rows {
        let payload = FastqRenderedCommandArgvRow {
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: row.corpus_id.clone(),
            asset_profile_id: row.asset_profile_id.clone(),
            adapter_id: row.adapter_id.clone(),
            parser_id: row.parser_id.clone(),
            schema_id: row.schema_id.clone(),
            readiness_kind: row.readiness_kind.clone(),
            benchmark_status: row.benchmark_status.clone(),
            command_source: row.command_source.clone(),
            command_steps: row.command_steps.clone(),
        };
        let line =
            serde_json::to_string(&payload).context("serialize FASTQ rendered command argv row")?;
        rendered.push_str(&line);
        rendered.push('\n');
    }
    Ok(rendered)
}
