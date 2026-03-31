mod models;
mod planning;
mod probe;
mod reporting;

use std::process::{Command, Output};

use models::{ImagePlan, ImageTestOutcome, Summary};
use planning::{build_image_plans, filter_tools, load_platform_spec};
use probe::run_image_test;
use reporting::{log_discovered_images, log_header, log_summary};

#[derive(Copy, Clone, Eq, PartialEq)]
enum LogLevel {
    Info,
    Debug,
}

trait Logger {
    fn log(&mut self, level: LogLevel, line: &str);
    fn is_quiet(&self) -> bool;
}

struct StdoutLogger {
    level: LogLevel,
    quiet: bool,
}

impl StdoutLogger {
    fn new(level: LogLevel, quiet: bool) -> Self {
        Self { level, quiet }
    }
}

impl Logger for StdoutLogger {
    fn log(&mut self, level: LogLevel, line: &str) {
        if level == LogLevel::Info || self.level == LogLevel::Debug {
            println!("{line}");
        }
    }

    fn is_quiet(&self) -> bool {
        self.quiet
    }
}

trait CommandRunner {
    fn run(&self, args: &[&str]) -> Result<Output, std::io::Error>;
}

struct RealRunner;

impl CommandRunner for RealRunner {
    fn run(&self, args: &[&str]) -> Result<Output, std::io::Error> {
        let mut cmd = Command::new(args[0]);
        if args.len() > 1 {
            cmd.args(&args[1..]);
        }
        cmd.output()
    }
}

fn log_debug(logger: &mut dyn Logger, line: &str) {
    logger.log(LogLevel::Debug, line);
}

/// # Errors
/// Returns an error if loading specs, building plans, or executing image checks fails.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let platform = parse_arg_value(&args, "--platform");
    let tools_filter = parse_arg_value(&args, "--tools");
    let debug = args.iter().any(|arg| arg == "--debug")
        || std::env::var("DEBUG")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    let quiet = args.iter().any(|arg| arg == "--quiet")
        || std::env::var("QUIET")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    let platform_spec = load_platform_spec(platform.as_deref())?;
    let tools = filter_tools(tools_filter)?;
    let image_plans = build_image_plans(&platform_spec, &tools)?;

    let mut logger = StdoutLogger::new(
        if debug {
            LogLevel::Debug
        } else {
            LogLevel::Info
        },
        quiet,
    );
    log_header(
        &mut logger,
        Some(platform_spec.name.as_str()),
        platform_spec.runner,
        image_plans.len(),
    );
    log_discovered_images(&mut logger, &image_plans);

    let runner = RealRunner;
    let summary = run_image_tests(&runner, &mut logger, &image_plans)?;
    log_summary(&mut logger, &summary);

    if summary.fail > 0 {
        return Err(format!("image tests failed: {}", summary.fail).into());
    }

    Ok(())
}

fn run_image_tests(
    runner: &dyn CommandRunner,
    logger: &mut dyn Logger,
    plans: &[ImagePlan],
) -> Result<Summary, Box<dyn std::error::Error>> {
    let mut summary = Summary::default();
    for plan in plans {
        let outcome = run_image_test(runner, logger, plan)?;
        match outcome {
            ImageTestOutcome::Pass(kind) => {
                summary.pass += 1;
                if !logger.is_quiet() {
                    logger.log(
                        LogLevel::Info,
                        &format!("PASS [{}] {}", kind, plan.image_name),
                    );
                }
            }
            ImageTestOutcome::Fail(reason) => {
                summary.fail += 1;
                logger.log(
                    LogLevel::Info,
                    &format!("FAIL [{}] {}", reason, plan.image_name),
                );
            }
        }
    }
    Ok(summary)
}

fn parse_arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|idx| args.get(idx + 1))
        .cloned()
}

#[cfg(test)]
mod tests {
    use std::process::ExitStatus;

    use super::models::{ImageFailureReason, ImagePlan, ImageTestOutcome};
    use super::probe::{image_exists, run_container_command, run_image_test};
    use super::{CommandRunner, LogLevel, Logger};
    use std::process::Output;

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
}
