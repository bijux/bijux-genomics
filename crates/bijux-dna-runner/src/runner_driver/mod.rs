use std::io::Read as _;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{anyhow, bail, Context};
use bijux_dna_environment::api::RuntimeKind;
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
    let mut child = child.spawn().with_context(|| format!("run local command {command}"))?;
    let (status, stdout, stderr) = if let Some(timeout) = timeout {
        wait_with_timeout(&mut child, timeout)?
    } else {
        let output = child
            .wait_with_output()
            .with_context(|| format!("capture local command output {command}"))?;
        (
            output.status,
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )
    };
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

fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> anyhow::Result<(std::process::ExitStatus, String, String)> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().context("poll local command")? {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut handle) = child.stdout.take() {
                handle.read_to_end(&mut stdout).context("read local stdout")?;
            }
            if let Some(mut handle) = child.stderr.take() {
                handle.read_to_end(&mut stderr).context("read local stderr")?;
            }
            return Ok((
                status,
                String::from_utf8_lossy(&stdout).to_string(),
                String::from_utf8_lossy(&stderr).to_string(),
            ));
        }
        if std::time::Instant::now() >= deadline {
            child.kill().context("kill timed out local command")?;
            let status = child.wait().context("wait for killed local command")?;
            let mut stderr = Vec::new();
            if let Some(mut handle) = child.stderr.take() {
                handle.read_to_end(&mut stderr).context("read local stderr after timeout")?;
            }
            return Ok((
                status,
                String::new(),
                format!(
                    "{}\nlocal command exceeded timeout of {}s",
                    String::from_utf8_lossy(&stderr),
                    timeout.as_secs()
                )
                .trim()
                .to_string(),
            ));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[cfg(test)]
mod tests {
    use super::run_local_command;

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
}
