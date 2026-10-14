use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{anyhow, bail, Context};
use bijux_dna_containers::api::RuntimeKind;
use bijux_dna_runtime::{Invocation, Runner, RunnerResult};

mod artifact_collection;

use artifact_collection::collect_existing_artifacts;

#[derive(Debug, Clone, Copy)]
pub struct DockerRunner {
    pub timeout: Option<Duration>,
}

impl DockerRunner {
    #[must_use]
    pub fn new(timeout: Option<Duration>) -> Self {
        Self { timeout }
    }
}

impl Runner for DockerRunner {
    fn run(&self, invocation: &Invocation) -> anyhow::Result<RunnerResult> {
        let result =
            crate::step_runner::execute_step(&invocation.step, RuntimeKind::Docker, self.timeout)?;
        runner_result_from_stage_result(result)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LocalRunner {
    pub timeout: Option<Duration>,
}

impl LocalRunner {
    #[must_use]
    pub fn new(timeout: Option<Duration>) -> Self {
        Self { timeout }
    }
}

impl Runner for LocalRunner {
    fn run(&self, invocation: &Invocation) -> anyhow::Result<RunnerResult> {
        let result =
            crate::step_runner::execute_step(&invocation.step, RuntimeKind::Local, self.timeout)?;
        runner_result_from_stage_result(result)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ApptainerRunner {
    pub runtime: RuntimeKind,
    pub timeout: Option<Duration>,
}

impl ApptainerRunner {
    #[must_use]
    pub fn new(runtime: RuntimeKind, timeout: Option<Duration>) -> Self {
        debug_assert!(matches!(runtime, RuntimeKind::Apptainer | RuntimeKind::Singularity));
        Self { runtime, timeout }
    }
}

impl Runner for ApptainerRunner {
    fn run(&self, invocation: &Invocation) -> anyhow::Result<RunnerResult> {
        let result =
            crate::step_runner::execute_step(&invocation.step, self.runtime, self.timeout)?;
        runner_result_from_stage_result(result)
    }
}

fn runner_result_from_stage_result(
    result: crate::step_runner::StageResultV1,
) -> anyhow::Result<RunnerResult> {
    let crate::step_runner::StageResultV1 {
        exit_code,
        runtime_s,
        outputs,
        metrics_path,
        stdout,
        stderr,
        ..
    } = result;
    let mut paths = outputs;
    if let Some(metrics_path) = metrics_path {
        paths.push(metrics_path);
    }
    let artifacts = collect_existing_artifacts(paths)?;
    Ok(RunnerResult {
        exit_code,
        stdout,
        stderr,
        duration: Duration::from_secs_f64(runtime_s),
        artifacts,
    })
}

pub(crate) fn run_local_command(
    template: &[String],
    current_dir: &std::path::Path,
    timeout: Option<Duration>,
) -> anyhow::Result<crate::command_runner::CommandOutputV1> {
    run_local_command_with_context(template, current_dir, timeout, None)
}

pub(crate) fn run_local_command_with_context(
    template: &[String],
    current_dir: &std::path::Path,
    timeout: Option<Duration>,
    environment: Option<&BTreeMap<OsString, OsString>>,
) -> anyhow::Result<crate::command_runner::CommandOutputV1> {
    let Some((command, args)) = template.split_first() else {
        bail!("local command template must not be empty");
    };
    let start = std::time::Instant::now();
    let mut child = Command::new(command);
    child
        .args(args)
        .current_dir(current_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(environment) = environment {
        child.envs(environment);
    }
    crate::process_supervision::isolate_process_group(&mut child);
    let mut child = child.spawn().with_context(|| format!("run local command {command}"))?;
    let (status, stdout, stderr, timed_out) = if let Some(timeout) = timeout {
        let output = crate::process_supervision::wait_for_output(
            &mut child,
            timeout,
            &format!("local command {command}"),
        )?;
        let mut stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if output.timed_out {
            if !stderr.is_empty() {
                stderr.push('\n');
            }
            write!(stderr, "local command exceeded timeout of {}s", timeout.as_secs())
                .context("append local command timeout")?;
        }
        (
            output.status,
            String::from_utf8_lossy(&output.stdout).to_string(),
            stderr,
            output.timed_out,
        )
    } else {
        let output = child
            .wait_with_output()
            .with_context(|| format!("capture local command output {command}"))?;
        (
            output.status,
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
            false,
        )
    };
    if timed_out {
        return Err(anyhow!(stderr));
    }
    if status.code().is_none() {
        return Err(anyhow!("local command {command} terminated by signal"));
    }
    Ok(crate::command_runner::CommandOutputV1 {
        stdout,
        stderr,
        exit_code: status.code().unwrap_or(-1),
        runtime_s: start.elapsed().as_secs_f64(),
        command: if args.is_empty() {
            command.clone()
        } else {
            format!("{command} {}", args.join(" "))
        },
    })
}

#[cfg(test)]
mod tests {
    use super::run_local_command;
    use anyhow::anyhow;
    use std::time::Duration;

    #[test]
    fn local_command_runner_executes_host_processes() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let output = run_local_command(
            &["sh".to_string(), "-c".to_string(), "printf 'hello-local' > result.txt".to_string()],
            temp.path(),
            None,
        )?;

        assert_eq!(output.exit_code, 0);
        assert_eq!(std::fs::read_to_string(temp.path().join("result.txt"))?, "hello-local");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn local_command_timeout_is_classified_after_reaping() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let command =
            vec!["sh".to_string(), "-c".to_string(), "exec tail -f /dev/null".to_string()];
        let error = run_local_command(&command, temp.path(), Some(Duration::from_millis(20)))
            .err()
            .ok_or_else(|| anyhow!("timed local command must fail after it is reaped"))?;

        assert!(error.to_string().contains("local command exceeded timeout"));
        Ok(())
    }
}
