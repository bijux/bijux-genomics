use std::path::Path;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use anyhow::{Context, Result};

mod command_line;
mod command_output;
mod execution_binding;
mod invocation_identity;

use command_line::build_command_string;
pub use command_output::{CommandOutputV1, TimedCommandOutputV1};
pub use execution_binding::{
    BoundFileIdentityV1, CommandExecutionBindingV1, CommandExecutionBindingV2,
    ExecutionBindingManifestV1, ExecutionBindingManifestV2, RUNNER_EXECUTION_BINDINGS_ENV,
    RUNNER_EXECUTION_BINDINGS_SCHEMA_V1, RUNNER_EXECUTION_BINDINGS_SCHEMA_V2,
};
pub use invocation_identity::invocation_hash;

use std::collections::BTreeMap;

/// # Errors
/// Returns an error if the command cannot be executed.
pub fn run_command(command: &str, args: &[String]) -> Result<CommandOutputV1> {
    run_command_with_context_and_stdin(command, args, None, None, None)
}

/// # Errors
/// Returns an error if the command cannot be executed.
pub fn run_command_with_context(
    command: &str,
    args: &[String],
    current_dir: Option<&Path>,
    envs: Option<&BTreeMap<String, String>>,
) -> Result<CommandOutputV1> {
    run_command_with_context_and_stdin(command, args, current_dir, envs, None)
}

/// # Errors
/// Returns an error if the command cannot be executed.
pub fn run_command_with_context_and_timeout(
    command: &str,
    args: &[String],
    current_dir: Option<&Path>,
    envs: Option<&BTreeMap<String, String>>,
    timeout: Duration,
) -> Result<TimedCommandOutputV1> {
    let resolved = execution_binding::resolve_command(command, args, envs)?;

    let start = Instant::now();
    let mut child = Command::new(&resolved.program);
    child.args(&resolved.args);
    if let Some(current_dir) = current_dir {
        child.current_dir(current_dir);
    }
    if let Some(envs) = envs {
        child.envs(envs);
    }
    child.stdin(std::process::Stdio::null());
    child.stdout(std::process::Stdio::piped());
    child.stderr(std::process::Stdio::piped());
    crate::process_supervision::isolate_process_group(&mut child);

    let mut process = child.spawn().with_context(|| format!("run command {}", resolved.program))?;
    let supervised = crate::process_supervision::wait_for_output(
        &mut process,
        timeout,
        &format!("command {}", resolved.program),
    )?;
    let mut stderr = supervised.stderr;
    if supervised.timed_out {
        if !stderr.is_empty() {
            stderr.push(b'\n');
        }
        std::io::Write::write_fmt(
            &mut stderr,
            format_args!("local command exceeded timeout of {}s", timeout.as_secs()),
        )
        .context("append local command timeout")?;
    }
    Ok(TimedCommandOutputV1 {
        output: command_output_from_parts(
            &resolved.program,
            &resolved.args,
            start,
            supervised.status.code().unwrap_or(-1),
            &supervised.stdout,
            &stderr,
        ),
        timed_out: supervised.timed_out,
    })
}

/// # Errors
/// Returns an error if the command cannot be executed.
pub fn run_command_with_context_and_stdin(
    command: &str,
    args: &[String],
    current_dir: Option<&Path>,
    envs: Option<&BTreeMap<String, String>>,
    stdin_bytes: Option<&[u8]>,
) -> Result<CommandOutputV1> {
    let resolved = execution_binding::resolve_command(command, args, envs)?;
    let start = Instant::now();
    let mut child = Command::new(&resolved.program);
    child.args(&resolved.args);
    if let Some(current_dir) = current_dir {
        child.current_dir(current_dir);
    }
    if let Some(envs) = envs {
        child.envs(envs);
    }
    if stdin_bytes.is_some() {
        use std::io::Write;
        use std::process::Stdio;

        child.stdin(Stdio::piped());
        child.stdout(Stdio::piped());
        child.stderr(Stdio::piped());

        let mut process =
            child.spawn().with_context(|| format!("run command {}", resolved.program))?;
        if let Some(input) = stdin_bytes {
            let stdin = process
                .stdin
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("open stdin for command {}", resolved.program))?;
            stdin
                .write_all(input)
                .with_context(|| format!("write stdin for command {}", resolved.program))?;
        }
        let output = process
            .wait_with_output()
            .with_context(|| format!("collect output for command {}", resolved.program))?;
        return Ok(command_output(&resolved.program, &resolved.args, start, &output));
    }
    let output = child.output().with_context(|| format!("run command {}", resolved.program))?;
    Ok(command_output(&resolved.program, &resolved.args, start, &output))
}

fn command_output(
    command: &str,
    args: &[String],
    start: Instant,
    output: &std::process::Output,
) -> CommandOutputV1 {
    command_output_from_parts(
        command,
        args,
        start,
        output.status.code().unwrap_or(-1),
        &output.stdout,
        &output.stderr,
    )
}

fn command_output_from_parts(
    command: &str,
    args: &[String],
    start: Instant,
    exit_code: i32,
    stdout: &[u8],
    stderr: &[u8],
) -> CommandOutputV1 {
    let runtime_s = start.elapsed().as_secs_f64();
    let stdout = String::from_utf8_lossy(stdout).to_string();
    let stderr = String::from_utf8_lossy(stderr).to_string();
    CommandOutputV1 {
        stdout,
        stderr,
        exit_code,
        runtime_s,
        command: build_command_string(command, args),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use super::{
        run_command_with_context, run_command_with_context_and_stdin,
        run_command_with_context_and_timeout, BoundFileIdentityV1, CommandExecutionBindingV1,
        ExecutionBindingManifestV1, RUNNER_EXECUTION_BINDINGS_ENV,
        RUNNER_EXECUTION_BINDINGS_SCHEMA_V1,
    };

    #[test]
    fn run_command_rejects_empty_executable() {
        let Err(err) = run_command_with_context("  ", &[], None, None) else {
            panic!("empty executable should be rejected before spawn");
        };

        assert_eq!(err.to_string(), "command executable must not be empty");
    }

    #[cfg(unix)]
    #[test]
    fn run_command_supports_stdin_payload() {
        let args = Vec::new();
        let output = run_command_with_context_and_stdin("cat", &args, None, None, Some(b"bijux"))
            .expect("run cat");
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout, "bijux");
    }

    #[cfg(unix)]
    #[test]
    fn configured_binding_runs_verified_program_and_records_resolved_invocation(
    ) -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt as _;

        let temporary = tempfile::tempdir()?;
        let program = temporary.path().join("bound-tool");
        std::fs::write(&program, b"#!/bin/sh\nprintf 'bound:%s' \"$1\"\n")?;
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700))?;
        let manifest_path = temporary.path().join("execution-bindings.json");
        let manifest = ExecutionBindingManifestV1 {
            schema_version: RUNNER_EXECUTION_BINDINGS_SCHEMA_V1.to_string(),
            bindings: vec![CommandExecutionBindingV1 {
                command: "bijux-tool".to_string(),
                program: program.display().to_string(),
                prefix_args: Vec::new(),
                required_files: vec![BoundFileIdentityV1 {
                    path: program.display().to_string(),
                    sha256: bijux_dna_infra::hash_file_sha256(&program)?,
                    size_bytes: program.metadata()?.len(),
                }],
            }],
        };
        std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
        let envs = BTreeMap::from([(
            RUNNER_EXECUTION_BINDINGS_ENV.to_string(),
            manifest_path.display().to_string(),
        )]);

        let output =
            run_command_with_context("bijux-tool", &["payload".to_string()], None, Some(&envs))?;

        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout, "bound:payload");
        assert_eq!(output.command, format!("{} payload", program.display()));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn timed_command_drains_output_larger_than_a_pipe() -> anyhow::Result<()> {
        let args = vec![
            "-c".to_string(),
            "i=0; while [ \"$i\" -lt 20000 ]; do printf 1234567890; i=$((i+1)); done".to_string(),
        ];
        let output =
            run_command_with_context_and_timeout("sh", &args, None, None, Duration::from_secs(5))?;

        assert!(!output.timed_out);
        assert_eq!(output.output.exit_code, 0);
        assert_eq!(output.output.stdout.len(), 200_000);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn timed_command_kills_and_reaps_at_deadline() -> anyhow::Result<()> {
        let args = vec!["-c".to_string(), "printf ready; tail -f /dev/null".to_string()];
        let output = run_command_with_context_and_timeout(
            "sh",
            &args,
            None,
            None,
            Duration::from_millis(250),
        )?;

        assert!(output.timed_out);
        assert_eq!(output.output.stdout, "ready");
        assert!(output.output.stderr.contains("local command exceeded timeout"));
        Ok(())
    }
}
