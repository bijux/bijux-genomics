use anyhow::{Context, Result};
use bijux_dna_bench_vcf_evidence::model::commands::VcfRenderedCommandRow;
pub(super) fn render_vcf_pca_commands_shell_script(
    rows: &[VcfRenderedCommandRow],
    repo_root_relative_to_output: &str,
) -> String {
    let mut rendered = format!("#!/usr/bin/env bash\nset -euo pipefail\nrepo_root=\"$(cd \"$(dirname \"${{BASH_SOURCE[0]}}\")/{repo_root_relative_to_output}\" && pwd)\"\n");
    rendered.push_str("cd \"$repo_root\"\n\n");
    for (index, row) in rows.iter().enumerate() {
        rendered.push_str("# ");
        rendered.push_str(&row.stage_id);
        rendered.push_str(" / ");
        rendered.push_str(&row.tool_id);
        rendered.push('\n');
        for step in &row.command_steps {
            rendered.push_str(&step.command);
            rendered.push('\n');
        }
        if index + 1 < rows.len() {
            rendered.push('\n');
        }
    }
    rendered
}
pub(super) fn render_vcf_pca_command_argv_jsonl(rows: &[VcfRenderedCommandRow]) -> Result<String> {
    let mut rendered = String::new();
    for row in rows {
        let payload = serde_json::json!({
            "stage_id": row.stage_id,
            "tool_id": row.tool_id,
            "readiness_kind": row.readiness_kind,
            "benchmark_status": row.benchmark_status,
            "command_source": row.command_source,
            "command_steps": row.command_steps.iter().map(|step| serde_json::json!({
                "step_id": step.step_id,
                "step_kind": step.step_kind,
                "consumes_previous_stdout": step.consumes_previous_stdout,
                "argv": step.argv,
            })).collect::<Vec<_>>(),
        });
        rendered.push_str(
            &serde_json::to_string(&payload).context("serialize VCF PCA command argv row")?,
        );
        rendered.push('\n');
    }
    Ok(rendered)
}
