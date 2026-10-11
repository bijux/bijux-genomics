use std::process::Command;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use wait_timeout::ChildExt as _;

pub fn docker_rm(container_id: &str) -> Result<()> {
    let output = Command::new("docker")
        .arg("rm")
        .arg("-f")
        .arg(container_id)
        .output()
        .context("docker rm")?;
    if !output.status.success() {
        return Err(anyhow!("docker rm failed for {container_id}"));
    }
    Ok(())
}

pub(crate) fn push_arg(cmd: &mut Command, args: &mut Vec<String>, value: impl Into<String>) {
    let value = value.into();
    cmd.arg(&value);
    args.push(value);
}

pub(crate) fn command_string(args: &[String]) -> String {
    format!("docker {}", args.join(" "))
}

pub(crate) fn docker_wait_timeout(container_id: &str, timeout: Duration) -> Result<i32> {
    let mut child = Command::new("docker")
        .arg("wait")
        .arg(container_id)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("docker wait")?;
    let Some(status) = child.wait_timeout(timeout).context("wait for docker")? else {
        child.kill().context("kill timed out docker wait")?;
        let _status = child.wait().context("reap timed out docker wait")?;
        return Err(anyhow!("docker wait timed out after {}s", timeout.as_secs()));
    };
    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stdout.take() {
        std::io::Read::read_to_string(&mut pipe, &mut stdout).context("read docker wait stdout")?;
    }
    if let Some(mut pipe) = child.stderr.take() {
        std::io::Read::read_to_string(&mut pipe, &mut stderr).context("read docker wait stderr")?;
    }
    if !status.success() {
        return Err(anyhow!("docker wait failed for {container_id}: {}", stderr.trim()));
    }
    stdout.trim().parse::<i32>().context("parse docker wait output")
}

pub(crate) fn docker_logs(container_id: &str) -> Result<String> {
    let output =
        Command::new("docker").arg("logs").arg(container_id).output().context("docker logs")?;
    if !output.status.success() {
        return Err(anyhow!("docker logs failed for {container_id}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
