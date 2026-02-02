fn log_debug(logger: &mut dyn Logger, line: &str) {
    logger.log(LogLevel::Debug, line);
}

#[derive(Clone, Debug)]
struct ImagePlan {
    image_name: String,
    expected_version: String,
    probe_cmd: Option<String>,
    probe_expected_exit: Vec<i32>,
    executable: Option<String>,
}

#[derive(Default)]
struct Summary {
    pass: usize,
    fail: usize,
}

enum ImageTestOutcome {
    Pass(ImageProbeKind),
    Fail(ImageFailureReason),
}

enum ImageFailureReason {
    ImageNotFound,
    ExecutableMissing,
    ProbeFailed,
    RuntimeDependencyMissing(String),
    UnexpectedExitCode(i32),
}

impl std::fmt::Display for ImageFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFailureReason::ImageNotFound => write!(f, "image not found"),
            ImageFailureReason::ExecutableMissing => write!(f, "executable missing"),
            ImageFailureReason::ProbeFailed => write!(f, "probe failed"),
            ImageFailureReason::RuntimeDependencyMissing(dep) => {
                write!(f, "missing runtime dependency: {dep}")
            }
            ImageFailureReason::UnexpectedExitCode(code) => {
                write!(f, "unexpected exit code {code}")
            }
        }
    }
}

enum ImageProbeKind {
    Version,
    Exec,
}

impl std::fmt::Display for ImageProbeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageProbeKind::Version => write!(f, "version"),
            ImageProbeKind::Exec => write!(f, "exec"),
        }
    }
}

struct ProbeResult {
    exit_code: i32,
    output: String,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::ExitStatus;

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
        inspect_present: bool,
    }

    impl CommandRunner for FakeRunner {
        fn run(&self, args: &[&str]) -> Result<Output, std::io::Error> {
            let joined = args.join(" ");
            let (status, stdout, stderr) = if joined.contains("image inspect") {
                if self.inspect_present {
                    (exit_status(0), "ok".to_string(), String::new())
                } else {
                    (exit_status(1), String::new(), "not found".to_string())
                }
            } else {
                (exit_status(0), "v1.0.0".to_string(), String::new())
            };
            Ok(Output {
                status,
                stdout: stdout.into_bytes(),
                stderr: stderr.into_bytes(),
            })
        }
    }

    #[test]
    fn test_info_output_is_summary_only() -> Result<(), Box<dyn std::error::Error>> {
        let plan = ImagePlan {
            image_name: "bijuxdna/fake:1.0.0-arm64".to_string(),
            expected_version: "1.0.0".to_string(),
            probe_cmd: Some("fake --version".to_string()),
            probe_expected_exit: vec![0],
            executable: None,
        };
        let runner = FakeRunner {
            inspect_present: false,
        };
        let mut logger = BufferLogger::new(LogLevel::Info, false);

        let summary = run_image_tests(&runner, &mut logger, &[plan])?;

        assert_eq!(summary.fail, 1);
        assert!(logger.lines.iter().any(|line| line.contains("FAIL")));
        assert!(logger
            .lines
            .iter()
            .all(|line| !line.contains("stdout") && !line.contains("stderr")));
        Ok(())
    }

    #[test]
    fn test_info_snapshot_contains_results_and_summary() -> Result<(), Box<dyn std::error::Error>> {
        let pass_plan = ImagePlan {
            image_name: "bijuxdna/pass:1.0.0-arm64".to_string(),
            expected_version: "1.0.0".to_string(),
            probe_cmd: Some("pass --version".to_string()),
            probe_expected_exit: vec![0],
            executable: None,
        };
        let fail_plan = ImagePlan {
            image_name: "bijuxdna/fail:1.0.0-arm64".to_string(),
            expected_version: "1.0.0".to_string(),
            probe_cmd: Some("fail --version".to_string()),
            probe_expected_exit: vec![0],
            executable: None,
        };
        let runner_pass = FakeRunner {
            inspect_present: true,
        };
        let runner_fail = FakeRunner {
            inspect_present: false,
        };
        let mut logger = BufferLogger::new(LogLevel::Info, false);

        let summary_pass = run_image_tests(&runner_pass, &mut logger, &[pass_plan])?;
        let summary_fail = run_image_tests(&runner_fail, &mut logger, &[fail_plan])?;

        assert_eq!(summary_pass.pass, 1);
        assert_eq!(summary_fail.fail, 1);
        assert!(logger
            .lines
            .iter()
            .any(|line| line.contains("PASS (probe:")));
        assert!(logger
            .lines
            .iter()
            .any(|line| line.contains("FAIL (image not found)")));
        Ok(())
    }
}
