use std::process::{ExitStatus, Output};
use std::sync::{Mutex, MutexGuard};

use bijux_dna_environment::build::DockerToolSpec;
use bijux_dna_environment::resolve::{PlatformSpec, RuntimeKind};

use super::args::parse_run_options;
use super::models::{ImageFailureReason, ImagePlan, ImageTestOutcome};
use super::planning::{build_image_plans, filter_tools};
use super::probe::{image_exists, run_container_command, run_image_test};
use super::runtime::{CommandRunner, LogLevel, Logger, RealRunner};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|err| panic!("environment lock poisoned: {err}"))
}

struct EnvVarGuard {
    key: &'static str,
    value: Option<std::ffi::OsString>,
}

impl EnvVarGuard {
    fn capture(key: &'static str) -> Self {
        Self { key, value: std::env::var_os(key) }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

#[cfg(unix)]
fn exit_status(code: i32) -> ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    ExitStatus::from_raw(code)
}

#[cfg(windows)]
fn exit_status(code: i32) -> ExitStatus {
    use std::os::windows::process::ExitStatusExt;
    ExitStatus::from_raw(code as u32)
}

struct BufferLogger {
    level: LogLevel,
    quiet: bool,
    lines: Vec<String>,
}

impl BufferLogger {
    fn new(level: LogLevel, quiet: bool) -> Self {
        Self { level, quiet, lines: Vec::new() }
    }
}

impl Logger for BufferLogger {
    fn log(&mut self, level: LogLevel, line: &str) {
        if level == LogLevel::Info || self.level == LogLevel::Debug {
            self.lines.push(line.to_string());
        }
    }

    fn is_quiet(&self) -> bool {
        self.quiet
    }
}

struct FakeRunner {
    output: Output,
}

impl CommandRunner for FakeRunner {
    fn run(&self, _args: &[&str]) -> Result<Output, std::io::Error> {
        Ok(self.output.clone())
    }
}

#[test]
fn image_exists_false_when_docker_fails() -> Result<(), Box<dyn std::error::Error>> {
    let runner = FakeRunner {
        output: Output { status: exit_status(1), stdout: Vec::new(), stderr: Vec::new() },
    };
    assert!(!image_exists(&runner, "missing")?);
    Ok(())
}

#[test]
fn image_exists_rejects_empty_image_names() {
    let runner = FakeRunner {
        output: Output { status: exit_status(0), stdout: Vec::new(), stderr: Vec::new() },
    };
    let error = image_exists(&runner, " ")
        .err()
        .unwrap_or_else(|| panic!("expected empty image rejection"));
    assert!(error.to_string().contains("image name is empty"));
}

#[test]
fn run_container_command_captures_stdout() -> Result<(), Box<dyn std::error::Error>> {
    let runner = FakeRunner {
        output: Output {
            status: exit_status(0),
            stdout: b"version 1.2.3".to_vec(),
            stderr: Vec::new(),
        },
    };
    let result = run_container_command(&runner, "image", "echo")?;
    assert_eq!(result.exit_code, 0);
    assert!(result.output.contains("version"));
    Ok(())
}

#[test]
fn run_container_command_captures_stderr() -> Result<(), Box<dyn std::error::Error>> {
    let runner = FakeRunner {
        output: Output {
            status: exit_status(0),
            stdout: Vec::new(),
            stderr: b"version 1.2.3".to_vec(),
        },
    };
    let result = run_container_command(&runner, "image", "tool --version")?;
    assert_eq!(result.exit_code, 0);
    assert!(result.output.contains("version 1.2.3"));
    Ok(())
}

#[test]
fn run_container_command_rejects_empty_probe_commands() {
    let runner = FakeRunner {
        output: Output { status: exit_status(0), stdout: Vec::new(), stderr: Vec::new() },
    };
    let error = run_container_command(&runner, "image", " ")
        .err()
        .unwrap_or_else(|| panic!("expected empty command rejection"));
    assert!(error.to_string().contains("probe command is empty"));
}

#[test]
fn run_image_test_reports_missing_image() -> Result<(), Box<dyn std::error::Error>> {
    let runner = FakeRunner {
        output: Output { status: exit_status(1), stdout: Vec::new(), stderr: Vec::new() },
    };
    let mut logger = BufferLogger::new(LogLevel::Info, true);
    let plan = ImagePlan {
        image_name: "missing".to_string(),
        expected_version: "1.0.0".to_string(),
        probe_cmd: None,
        probe_expected_exit: vec![0],
        executable: None,
    };
    let outcome = run_image_test(&runner, &mut logger, &plan)?;
    assert!(matches!(outcome, ImageTestOutcome::Fail(ImageFailureReason::ImageNotFound)));
    Ok(())
}

#[test]
fn run_options_do_not_consume_neighbor_flags_as_values() {
    let args =
        vec!["qa_docker_images".to_string(), "--platform".to_string(), "--quiet".to_string()];
    let options = parse_run_options(&args);
    assert_eq!(options.platform, None);
    assert!(options.quiet);
}

#[test]
fn run_options_trim_environment_boolean_flags() {
    let _env = env_lock();
    let _debug = EnvVarGuard::capture("DEBUG");
    let _quiet = EnvVarGuard::capture("QUIET");
    std::env::set_var("DEBUG", " true ");
    std::env::set_var("QUIET", " 1 ");
    let options = parse_run_options(&["qa_docker_images".to_string()]);
    assert!(options.debug);
    assert!(options.quiet);
}

#[test]
fn tool_filter_rejects_unknown_entries() {
    let error = filter_tools(Some("fastp,definitely_missing".to_string()))
        .err()
        .unwrap_or_else(|| panic!("expected unknown tool filter rejection"));
    assert!(error.to_string().contains("definitely_missing"));
}

#[test]
fn image_plan_build_rejects_empty_probe_exit_contracts() {
    let platform = PlatformSpec {
        name: "docker-test".to_string(),
        runner: RuntimeKind::Docker,
        container_dir: std::path::PathBuf::from("containers"),
        image_prefix: "bijuxdna".to_string(),
        arch: "amd64".to_string(),
    };
    let tool = DockerToolSpec {
        name: "fastp".to_string(),
        executable: Some("fastp".to_string()),
        version_cmd: "fastp --version".to_string(),
        help_cmd: None,
        probe_cmd: Some("fastp --version".to_string()),
        probe_expected_exit: Vec::new(),
    };
    let error = build_image_plans(&platform, &[tool])
        .err()
        .unwrap_or_else(|| panic!("expected empty probe exit contract rejection"));
    assert!(error.to_string().contains("probe expected exits are empty"));
}

#[test]
fn real_runner_rejects_empty_command_vectors() {
    let runner = RealRunner;
    let error = runner.run(&[]).err().unwrap_or_else(|| panic!("expected empty command rejection"));
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
}
