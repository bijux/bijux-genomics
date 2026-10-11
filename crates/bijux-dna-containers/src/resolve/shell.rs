use anyhow::Context;

/// Execute a shell command and capture stdout/stderr.
///
/// # Errors
/// Returns an error when command execution fails or exits non-zero.
pub(super) fn run_shell_capture(cmd: &str) -> anyhow::Result<String> {
    if cmd.trim().is_empty() {
        anyhow::bail!("empty command");
    }
    let output = std::process::Command::new("/bin/sh")
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
