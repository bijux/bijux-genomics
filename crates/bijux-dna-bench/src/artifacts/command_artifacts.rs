//! Owner: pure shell and argv JSONL projections of typed benchmark commands.
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    AllDomainRenderedCommandArgvRow, AllDomainRenderedCommandArgvStep, AllDomainRenderedCommandRow,
};

pub(crate) fn render_all_domain_commands_shell_script(
    rows: &[AllDomainRenderedCommandRow],
    repo_root_ascent: &str,
) -> String {
    let mut rendered = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    rendered.push_str(&format!(
        "repo_root=\"$(cd \"$(dirname \"${{BASH_SOURCE[0]}}\")/{repo_root_ascent}\" && pwd)\"\n"
    ));
    rendered.push_str("cd \"$repo_root\"\n\n");
    for (index, row) in rows.iter().enumerate() {
        rendered.push_str(&format!(
            "# {} / {} / {} / {}\n",
            row.result_id, row.domain, row.stage_id, row.tool_id
        ));
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
pub(crate) fn render_all_domain_command_argv_jsonl(
    rows: &[AllDomainRenderedCommandRow],
) -> Result<String> {
    let mut rendered = String::new();
    for row in rows {
        let line = serde_json::to_string(&AllDomainRenderedCommandArgvRow {
            result_id: row.result_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: row.corpus_id.clone(),
            asset_profile_id: row.asset_profile_id.clone(),
            readiness_kind: row.readiness_kind.clone(),
            benchmark_status: row.benchmark_status.clone(),
            command_source: row.command_source.clone(),
            command_steps: row
                .command_steps
                .iter()
                .map(|step| AllDomainRenderedCommandArgvStep {
                    step_id: step.step_id.clone(),
                    step_kind: step.step_kind.clone(),
                    consumes_previous_stdout: step.consumes_previous_stdout,
                    argv: step.argv.clone(),
                })
                .collect(),
        })
        .context("serialize all-domain rendered command argv row")?;
        rendered.push_str(&line);
        rendered.push('\n');
    }
    Ok(rendered)
}
