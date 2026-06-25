use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_runner::command_runner::{run_command_with_context, CommandOutputV1};

pub(crate) fn run_command_output<I, S>(
    command: &str,
    args: I,
    current_dir: Option<&Path>,
) -> Result<CommandOutputV1>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let argv = args.into_iter().map(|arg| arg.as_ref().to_string()).collect::<Vec<_>>();
    run_command_with_context(command, &argv, current_dir, None)
        .with_context(|| format!("run command {command} {}", argv.join(" ")))
}

pub(crate) fn run_checked_command<I, S>(
    command: &str,
    args: I,
    current_dir: Option<&Path>,
) -> Result<CommandOutputV1>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let output = run_command_output(command, args, current_dir)?;
    if output.exit_code != 0 {
        bail!("{command} failed: {}", failure_detail(&output));
    }
    Ok(output)
}

pub(crate) fn run_text_command<I, S>(
    command: &str,
    args: I,
    current_dir: Option<&Path>,
) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    Ok(run_checked_command(command, args, current_dir)?.stdout)
}

pub(crate) fn command_succeeds<I, S>(command: &str, args: I, current_dir: Option<&Path>) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    run_command_output(command, args, current_dir)
        .map(|output| output.exit_code == 0)
        .unwrap_or(false)
}

fn failure_detail(output: &CommandOutputV1) -> String {
    let stderr = output.stderr.trim();
    if !stderr.is_empty() {
        return stderr.to_string();
    }
    let stdout = output.stdout.trim();
    if !stdout.is_empty() {
        return format!("stdout: {stdout}");
    }
    format!("exit code {}", output.exit_code)
}
