use super::models::{ImageFailureReason, ImagePlan, ImageProbeKind, ImageTestOutcome, ProbeResult};
use super::runtime::{log_debug, CommandRunner, Logger};

pub(super) fn run_image_test(
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
            return Ok(ImageTestOutcome::Fail(ImageFailureReason::ExecutableMissing));
        }
    }
    if let Some(probe_cmd) = plan.probe_cmd.as_ref() {
        let probe_result = run_container_command(runner, &plan.image_name, probe_cmd)?;
        if !plan.probe_expected_exit.contains(&probe_result.exit_code) {
            return Ok(ImageTestOutcome::Fail(ImageFailureReason::UnexpectedExitCode(
                probe_result.exit_code,
            )));
        }
        if !probe_result.output.contains(&plan.expected_version) {
            return Ok(ImageTestOutcome::Fail(ImageFailureReason::ProbeFailed));
        }
        return Ok(ImageTestOutcome::Pass(ImageProbeKind::Version));
    }
    Ok(ImageTestOutcome::Pass(ImageProbeKind::Exec))
}

pub(super) fn image_exists(
    runner: &dyn CommandRunner,
    image: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    if image.trim().is_empty() {
        return Err("docker image name is empty".into());
    }
    let output = runner.run(&["docker", "image", "inspect", image])?;
    Ok(output.status.success())
}

pub(super) fn run_container_command(
    runner: &dyn CommandRunner,
    image: &str,
    cmd: &str,
) -> Result<ProbeResult, Box<dyn std::error::Error>> {
    if image.trim().is_empty() {
        return Err("docker image name is empty".into());
    }
    if cmd.trim().is_empty() {
        return Err("container probe command is empty".into());
    }
    let args = ["docker", "run", "--rm", image, "sh", "-c", cmd];
    let output = runner.run(&args)?;
    let mut probe_output = String::from_utf8_lossy(&output.stdout).to_string();
    probe_output.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(ProbeResult { exit_code: output.status.code().unwrap_or(1), output: probe_output })
}
