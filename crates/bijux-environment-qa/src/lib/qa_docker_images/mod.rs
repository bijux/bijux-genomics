use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_environment::build::{
    default_docker_tools, extract_version_from_dockerfile, DockerToolSpec,
};
use bijux_environment::resolve::{load_platform, ImageRef, PlatformSpec, RunnerKind};

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
    UnexpectedExitCode(i32),
}

impl std::fmt::Display for ImageFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFailureReason::ImageNotFound => write!(f, "image not found"),
            ImageFailureReason::ExecutableMissing => write!(f, "executable missing"),
            ImageFailureReason::ProbeFailed => write!(f, "probe failed"),
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
    let container_dir = PathBuf::from(&platform_spec.container_dir);
    let image_plans = build_image_plans(&platform_spec, &container_dir, &tools)?;

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
    log_discovered_images(&mut logger, &image_plans)?;

    let runner = RealRunner;
    let summary = run_image_tests(&runner, &mut logger, &image_plans)?;
    log_summary(&mut logger, &summary);

    if summary.fail > 0 {
        return Err(format!("image tests failed: {}", summary.fail).into());
    }

    Ok(())
}

fn load_platform_spec(platform: Option<&str>) -> Result<PlatformSpec, Box<dyn std::error::Error>> {
    let platform_spec = load_platform(platform)?;
    if platform_spec.runner != RunnerKind::Docker {
        return Err(format!(
            "platform runner must be docker, got {}",
            platform_spec.runner
        )
        .into());
    }
    Ok(platform_spec)
}

fn filter_tools(
    tools_filter: Option<String>,
) -> Result<Vec<DockerToolSpec>, Box<dyn std::error::Error>> {
    let mut tools = default_docker_tools();
    if let Some(filter) = tools_filter {
        let wanted: Vec<String> = filter
            .split(',')
            .map(|item| item.trim().to_lowercase())
            .filter(|item| !item.is_empty())
            .collect();
        if wanted.is_empty() {
            return Err("empty --tools filter".into());
        }
        tools.retain(|tool| wanted.contains(&tool.name.to_lowercase()));
        if tools.is_empty() {
            return Err("no matching tools for --tools filter".into());
        }
    }
    Ok(tools)
}

fn build_image_plans(
    platform_spec: &PlatformSpec,
    container_dir: &std::path::Path,
    tools: &[DockerToolSpec],
) -> Result<Vec<ImagePlan>, Box<dyn std::error::Error>> {
    let mut plans = Vec::new();
    for tool in tools {
        let dockerfile = container_dir.join(format!("{}.Dockerfile", tool.name));
        if !dockerfile.exists() {
            return Err(format!("Dockerfile not found: {}", dockerfile.display()).into());
        }
        let expected_version = extract_version_from_dockerfile(&dockerfile, &tool.name)?;
        let image = ImageRef {
            tool: tool.name.clone(),
            version: expected_version.clone(),
            arch: platform_spec.arch.clone(),
        };
        let image_name = image.to_full_name(&platform_spec.image_prefix);
        plans.push(ImagePlan {
            image_name,
            expected_version,
            probe_cmd: tool.probe_cmd.clone(),
            probe_expected_exit: tool.probe_expected_exit.clone(),
            executable: tool.executable.clone(),
        });
    }
    if plans.is_empty() {
        return Err("no images discovered".into());
    }
    Ok(plans)
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

fn run_image_test(
    runner: &dyn CommandRunner,
    logger: &mut dyn Logger,
    plan: &ImagePlan,
) -> Result<ImageTestOutcome, Box<dyn std::error::Error>> {
    log_debug(logger, &format!("checking {}", plan.image_name));
    if !image_exists(runner, &plan.image_name)? {
        return Ok(ImageTestOutcome::Fail(ImageFailureReason::ImageNotFound));
    }
    if let Some(executable) = plan.executable.as_ref() {
        let exec_result = run_container_command(runner, &plan.image_name, executable)?;
        if exec_result.exit_code != 0 {
            return Ok(ImageTestOutcome::Fail(
                ImageFailureReason::ExecutableMissing,
            ));
        }
    }
    if let Some(probe_cmd) = plan.probe_cmd.as_ref() {
        let probe_result = run_container_command(runner, &plan.image_name, probe_cmd)?;
        if !plan.probe_expected_exit.contains(&probe_result.exit_code) {
            return Ok(ImageTestOutcome::Fail(
                ImageFailureReason::UnexpectedExitCode(probe_result.exit_code),
            ));
        }
        if !probe_result.output.contains(&plan.expected_version) {
            return Ok(ImageTestOutcome::Fail(ImageFailureReason::ProbeFailed));
        }
        return Ok(ImageTestOutcome::Pass(ImageProbeKind::Version));
    }
    Ok(ImageTestOutcome::Pass(ImageProbeKind::Exec))
}

fn image_exists(
    runner: &dyn CommandRunner,
    image: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let output = runner.run(&["docker", "image", "inspect", image])?;
    Ok(output.status.success())
}

fn run_container_command(
    runner: &dyn CommandRunner,
    image: &str,
    cmd: &str,
) -> Result<ProbeResult, Box<dyn std::error::Error>> {
    let args = ["docker", "run", "--rm", image, "sh", "-c", cmd];
    let output = runner.run(&args)?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(ProbeResult {
        exit_code: output.status.code().unwrap_or(1),
        output: stdout,
    })
}

fn log_header(logger: &mut dyn Logger, platform: Option<&str>, runner: RunnerKind, total: usize) {
    let platform = platform.unwrap_or("unknown");
    logger.log(
        LogLevel::Info,
        &format!("Platform {platform} ({runner}) - {total} images"),
    );
}

fn log_discovered_images(
    logger: &mut dyn Logger,
    plans: &[ImagePlan],
) -> Result<(), Box<dyn std::error::Error>> {
    if logger.is_quiet() {
        return Ok(());
    }
    for plan in plans {
        logger.log(LogLevel::Info, &format!("image: {}", plan.image_name));
    }
    Ok(())
}

fn log_summary(logger: &mut dyn Logger, summary: &Summary) {
    logger.log(
        LogLevel::Info,
        &format!("Summary: {} pass / {} fail", summary.pass, summary.fail),
    );
}

fn parse_arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|idx| args.get(idx + 1))
        .map(|value| value.to_string())
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
        matches!(
            outcome,
            ImageTestOutcome::Fail(ImageFailureReason::ImageNotFound)
        );
        Ok(())
    }
}
