use anyhow::Context;

/// Execute container smoke contract script for a runtime/tool pair.
///
/// # Errors
/// Returns an error when the runtime is unsupported or smoke script exits non-zero.
pub(super) fn run_smoke_script(runtime: &str, tool: &str) -> anyhow::Result<()> {
    let command = smoke_command(runtime)?;
    let status = std::process::Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "bijux-dna-dev",
            "--",
            "containers",
            "run",
            command,
        ])
        .env("TOOLS", tool)
        .status()?;
    if !status.success() {
        anyhow::bail!("smoke failed for runtime={runtime} tool={tool} (exit={status})");
    }
    Ok(())
}

/// Execute smoke contract script for a runtime with multiple tools.
///
/// # Errors
/// Returns an error when runtime is unsupported or smoke script exits non-zero.
pub(super) fn run_smoke_script_batch(
    runtime: &str,
    tools: &[String],
    smoke_level: &str,
) -> anyhow::Result<()> {
    let command = smoke_command(runtime)?;
    let tools_csv = tools.join(",");
    let status = std::process::Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "bijux-dna-dev",
            "--",
            "containers",
            "run",
            command,
        ])
        .env("TOOLS", tools_csv)
        .env("SMOKE_LEVEL", smoke_level)
        .status()?;
    if !status.success() {
        anyhow::bail!("smoke failed for runtime={runtime} (exit={status})");
    }
    Ok(())
}

/// Execute a shell command and capture stdout/stderr.
///
/// # Errors
/// Returns an error when command execution fails or exits non-zero.
pub(super) fn run_shell_capture(cmd: &str) -> anyhow::Result<String> {
    if cmd.trim().is_empty() {
        anyhow::bail!("empty command");
    }
    let output = std::process::Command::new("sh")
        .arg("-lc")
        .arg(cmd)
        .output()
        .with_context(|| format!("execute `{cmd}`"))?;
    let merged = merge_command_output(&output.stdout, &output.stderr);
    if output.status.success() {
        Ok(merged)
    } else {
        Err(anyhow::anyhow!("{merged}"))
    }
}

fn smoke_command(runtime: &str) -> anyhow::Result<&'static str> {
    match runtime {
        "docker-arm64" => Ok("smoke-containers-docker-arm64"),
        "docker-amd64" => Ok("smoke-containers-docker-amd64"),
        "apptainer" => Ok("smoke-containers-apptainer"),
        other => {
            anyhow::bail!(
                "unsupported runtime `{other}`; expected docker-arm64 | docker-amd64 | apptainer"
            );
        }
    }
}

fn merge_command_output(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout).to_string();
    let stderr = String::from_utf8_lossy(stderr).to_string();
    if stdout.trim().is_empty() {
        return stderr;
    }
    if stderr.trim().is_empty() {
        return stdout;
    }
    if stdout.ends_with('\n') || stderr.starts_with('\n') {
        format!("{stdout}{stderr}")
    } else {
        format!("{stdout}\n{stderr}")
    }
}
