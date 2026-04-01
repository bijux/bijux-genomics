use std::process::{ExitStatus, Output};

use super::models::{ImageFailureReason, ImagePlan, ImageTestOutcome};
use super::probe::{image_exists, run_container_command, run_image_test};
use super::runtime::{CommandRunner, LogLevel, Logger};

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
        Self {
            level,
            quiet,
            lines: Vec::new(),
        }
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
        output: Output {
            status: exit_status(1),
            stdout: Vec::new(),
            stderr: Vec::new(),
        },
    };
    assert!(!image_exists(&runner, "missing")?);
    Ok(())
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
fn run_image_test_reports_missing_image() -> Result<(), Box<dyn std::error::Error>> {
    let runner = FakeRunner {
        output: Output {
            status: exit_status(1),
            stdout: Vec::new(),
            stderr: Vec::new(),
        },
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
    assert!(matches!(
        outcome,
        ImageTestOutcome::Fail(ImageFailureReason::ImageNotFound)
    ));
    Ok(())
}
