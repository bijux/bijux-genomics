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
