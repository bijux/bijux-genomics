use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_env_builder::{default_docker_tools, extract_version_from_dockerfile, DockerToolSpec};
use bijux_env_runtime::{load_platform, ImageRef, PlatformSpec, RunnerKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        log_image_banner(logger, plan);
        let outcome = validate_plan(runner, logger, plan)?;
        log_image_result(logger, plan, &outcome);
        match outcome {
            ImageTestOutcome::Pass(_) => summary.pass += 1,
            ImageTestOutcome::Fail(_) => summary.fail += 1,
        }
    }
    Ok(summary)
}

fn validate_plan(
    runner: &dyn CommandRunner,
    logger: &mut dyn Logger,
    plan: &ImagePlan,
) -> Result<ImageTestOutcome, Box<dyn std::error::Error>> {
    if !image_present(runner, logger, &plan.image_name)? {
        return Ok(ImageTestOutcome::Fail(ImageFailureReason::ImageNotFound));
    }

    if let Some(executable) = plan.executable.as_deref() {
        if !executable_present(runner, logger, &plan.image_name, executable)? {
            return Ok(ImageTestOutcome::Fail(
                ImageFailureReason::ExecutableMissing,
            ));
        }
    }

    if let Some(probe_cmd) = plan.probe_cmd.as_deref() {
        let probe = run_probe(runner, logger, &plan.image_name, probe_cmd)?;
        if !plan.probe_expected_exit.contains(&probe.exit_code) {
            let reason = classify_failure(&probe);
            return Ok(ImageTestOutcome::Fail(reason));
        }
        let expected = plan.expected_version.trim_start_matches('v').to_lowercase();
        if !probe.output.to_lowercase().contains(&expected) {
            return Ok(ImageTestOutcome::Fail(ImageFailureReason::ProbeFailed));
        }
        return Ok(ImageTestOutcome::Pass(ImageProbeKind::Version));
    }

    Ok(ImageTestOutcome::Pass(ImageProbeKind::Exec))
}

fn classify_failure(probe: &ProbeResult) -> ImageFailureReason {
    let combined = probe.output.to_lowercase();
    if combined.contains("error while loading shared libraries")
        || combined.contains("libgomp.so.1")
    {
        return ImageFailureReason::RuntimeDependencyMissing("libgomp.so.1".to_string());
    }
    ImageFailureReason::UnexpectedExitCode(probe.exit_code)
}

fn image_present(
    runner: &dyn CommandRunner,
    logger: &mut dyn Logger,
    image_name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    log_debug(
        logger,
        &format!("[bijux][image][check] docker image inspect {image_name}"),
    );
    let output = runner.run(&["docker", "image", "inspect", image_name])?;
    if output.status.success() {
        log_debug(logger, "[bijux][image][ok] image present locally");
        return Ok(true);
    }
    log_debug(logger, "[bijux][image][warn] image not found locally");
    log_debug(
        logger,
        &format!("[bijux][image][policy] pull disabled ({image_name})"),
    );
    Ok(false)
}

fn executable_present(
    runner: &dyn CommandRunner,
    logger: &mut dyn Logger,
    image_name: &str,
    executable: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    log_debug(
        logger,
        &format!("[bijux][image][check] docker run --entrypoint which {image_name} {executable}"),
    );
    let output = runner.run(&[
        "docker",
        "run",
        "--rm",
        "--entrypoint",
        "which",
        image_name,
        executable,
    ])?;
    if output.status.success() && !output_stdout(&output).trim().is_empty() {
        log_debug(logger, "[bijux][image][ok] executable present");
        return Ok(true);
    }
    log_debug(logger, "[bijux][image][warn] executable not found");
    Ok(false)
}

fn run_probe(
    runner: &dyn CommandRunner,
    logger: &mut dyn Logger,
    image_name: &str,
    probe_cmd: &str,
) -> Result<ProbeResult, Box<dyn std::error::Error>> {
    log_debug(logger, &format!("[bijux][image][run] {probe_cmd}"));
    let output = runner.run(&[
        "docker",
        "run",
        "--rm",
        "--entrypoint",
        "bash",
        image_name,
        "-c",
        probe_cmd,
    ])?;
    let combined = format!("{}{}", output_stdout(&output), output_stderr(&output));
    log_debug(
        logger,
        &format!("[bijux][image][run] exit {}", exit_code(&output)),
    );
    log_debug_output(logger, "stdout", output_stdout(&output).trim());
    log_debug_output(logger, "stderr", output_stderr(&output).trim());
    Ok(ProbeResult {
        exit_code: exit_code(&output),
        output: combined,
    })
}

fn log_header(
    logger: &mut dyn Logger,
    platform: Option<&str>,
    runner: RunnerKind,
    image_count: usize,
) {
    if logger.is_quiet() {
        return;
    }
    log_info(logger, &format!("[bijux] Image test started ({runner})"));
    log_info(
        logger,
        &format!(
            "Platform   : {}",
            platform.unwrap_or("default (platform.yaml)")
        ),
    );
    log_info(logger, &format!("Image count: {image_count}"));
}

fn log_discovered_images(
    logger: &mut dyn Logger,
    plans: &[ImagePlan],
) -> Result<(), Box<dyn std::error::Error>> {
    if logger.is_quiet() {
        return Ok(());
    }
    log_info(logger, "[bijux] Discovered images:");
    if plans.is_empty() {
        return Err("no images discovered".into());
    }
    for plan in plans {
        log_info(logger, &format!("  - {}", plan.image_name));
    }
    Ok(())
}

fn log_image_banner(logger: &mut dyn Logger, plan: &ImagePlan) {
    log_debug(logger, &format!("[bijux][image] {}", plan.image_name));
}

fn log_image_result(logger: &mut dyn Logger, plan: &ImagePlan, outcome: &ImageTestOutcome) {
    if logger.is_quiet() {
        return;
    }
    let line = match outcome {
        ImageTestOutcome::Pass(kind) => format!(
            "[bijux][image] {:<28} PASS (probe: {})",
            plan.image_name, kind
        ),
        ImageTestOutcome::Fail(reason) => {
            format!("[bijux][image] {:<28} FAIL ({})", plan.image_name, reason)
        }
    };
    log_info(logger, &line);
}

fn log_summary(logger: &mut dyn Logger, summary: &Summary) {
    if logger.is_quiet() {
        log_info(
            logger,
            &format!("PASS={} FAIL={}", summary.pass, summary.fail),
        );
        return;
    }
    log_info(
        logger,
        &format!(
            "[bijux] Summary: PASS={} FAIL={}",
            summary.pass, summary.fail
        ),
    );
}

fn log_debug_output(logger: &mut dyn Logger, label: &str, text: &str) {
    let trimmed = trim_output(text);
    if trimmed.is_empty() {
        return;
    }
    log_debug(logger, &format!("[bijux][image][{label}] {trimmed}"));
}

fn trim_output(text: &str) -> String {
    const LIMIT: usize = 200;
    if text.len() <= LIMIT {
        return text.to_string();
    }
    let mut trimmed = text[..LIMIT].to_string();
    trimmed.push_str("...");
    trimmed
}

fn exit_code(output: &Output) -> i32 {
    output.status.code().unwrap_or(-1)
}

fn output_stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn output_stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn parse_arg_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|idx| args.get(idx + 1))
        .cloned()
}

fn log_info(logger: &mut dyn Logger, line: &str) {
    logger.log(LogLevel::Info, line);
}

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
