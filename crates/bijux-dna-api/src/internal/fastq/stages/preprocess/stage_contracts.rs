use super::{Context, ExecutionStep, Result};

pub(super) fn write_stage_path_contract(
    stage_root: &std::path::Path,
    stage_id: &str,
    planned: &ExecutionStep,
    is_paired: bool,
) -> Result<()> {
    bijux_dna_infra::ensure_dir(stage_root).context("create stage root for path contract")?;
    let outputs = planned
        .io
        .outputs
        .iter()
        .map(|x| {
            serde_json::json!({
                "name": x.name,
                "role": x.role.as_str(),
                "path": x.path
            })
        })
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.path_contract.v1",
        "stage_id": stage_id,
        "layout": if is_paired { "pe" } else { "se" },
        "deterministic_root": stage_root,
        "intermediate_root": stage_root.join("tmp"),
        "intermediate_paths": {
            "stdout_log": stage_root.join("stdout.log"),
            "stderr_log": stage_root.join("stderr.log"),
            "runtime_provenance": stage_root.join("runtime_provenance.json"),
            "resume_contract": stage_root.join("stage.resume_contract.json"),
        },
        "outputs": outputs,
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.path_contract.json"), &payload)
        .context("write stage.path_contract.json")
}

pub(super) fn capture_tool_version(
    stage_root: &std::path::Path,
    tool_bin: Option<&str>,
) -> Result<()> {
    let (declared_tool, ok, raw) =
        if let Some(tool_bin) = tool_bin.filter(|value| !value.trim().is_empty()) {
            let args = vec!["--version".to_string()];
            let output = bijux_dna_runner::command_runner::run_command(tool_bin, &args);
            let (ok, raw) = match output {
                Ok(out) => {
                    let raw = if out.stdout.is_empty() { out.stderr } else { out.stdout };
                    (out.exit_code == 0, raw)
                }
                Err(err) => (false, format!("failed to execute --version: {err}")),
            };
            (tool_bin, ok, raw)
        } else {
            ("", false, "tool command not declared in execution template".to_string())
        };
    let line = raw.lines().find(|x| !x.trim().is_empty()).unwrap_or("").trim();
    let tokenized = line
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '(' || c == ')')
        .filter(|x| !x.trim().is_empty())
        .collect::<Vec<_>>();
    let version = tokenized.iter().find_map(|tok| {
        let t = tok.trim_start_matches('v').trim_start_matches('V');
        if t.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            Some(t.to_string())
        } else {
            None
        }
    });
    let payload = serde_json::json!({
        "schema_version": "bijux.tool_version_capture.v1",
        "tool": declared_tool,
        "ok": ok,
        "raw": raw,
        "parsed": {
            "first_line": line,
            "version": version
        }
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.tool_version.json"), &payload)
        .context("write stage.tool_version.json")
}
