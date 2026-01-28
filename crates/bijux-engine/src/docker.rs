use std::process::Command;

use anyhow::{anyhow, Context, Result};

pub(crate) fn push_arg(cmd: &mut Command, args: &mut Vec<String>, value: impl Into<String>) {
    let value = value.into();
    cmd.arg(&value);
    args.push(value);
}

pub(crate) fn command_string(args: &[String]) -> String {
    format!("docker {}", args.join(" "))
}

pub fn docker_wait(container_id: &str) -> Result<i32> {
    let output = Command::new("docker")
        .arg("wait")
        .arg(container_id)
        .output()
        .context("docker wait")?;
    if !output.status.success() {
        return Err(anyhow!("docker wait failed for {container_id}"));
    }
    let code = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<i32>()
        .context("parse docker wait output")?;
    Ok(code)
}

pub fn docker_wait_timeout(container_id: &str, timeout: std::time::Duration) -> Result<i32> {
    let start = std::time::Instant::now();
    loop {
        let output = Command::new("docker")
            .arg("inspect")
            .arg(container_id)
            .arg("--format")
            .arg("{{.State.Status}}")
            .output()
            .context("docker inspect")?;
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if status == "exited" {
                return docker_wait(container_id);
            }
        }
        if start.elapsed() >= timeout {
            return Err(anyhow!("timeout"));
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

pub fn docker_logs(container_id: &str) -> Result<String> {
    let output = Command::new("docker")
        .arg("logs")
        .arg(container_id)
        .output()
        .context("docker logs")?;
    if !output.status.success() {
        return Err(anyhow!("docker logs failed for {container_id}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn docker_stats_mb(container_id: &str) -> Result<f64> {
    let output = Command::new("docker")
        .arg("stats")
        .arg("--no-stream")
        .arg("--format")
        .arg("{{.MemUsage}}")
        .arg(container_id)
        .output()
        .context("docker stats")?;
    if !output.status.success() {
        return Err(anyhow!("docker stats failed for {container_id}"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mem = stdout
        .lines()
        .next()
        .ok_or_else(|| anyhow!("missing docker stats output"))?;
    parse_mem_to_mb(mem)
}

pub fn parse_mem_to_mb(value: &str) -> Result<f64> {
    let parts: Vec<&str> = value.split('/').collect();
    let value = parts
        .first()
        .ok_or_else(|| anyhow!("invalid memory format"))?
        .trim();
    let mut number = String::new();
    let mut unit = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            number.push(ch);
        } else {
            unit.push(ch);
        }
    }
    let num: f64 = number.parse().context("parse memory value")?;
    let mb = match unit.as_str() {
        "B" => num / 1024.0 / 1024.0,
        "KiB" => num / 1024.0,
        "MiB" => num,
        "GiB" => num * 1024.0,
        _ => return Err(anyhow!("unknown memory unit: {unit}")),
    };
    Ok(mb)
}

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

#[cfg(test)]
mod tests {
    use super::parse_mem_to_mb;

    #[test]
    fn parse_mem_to_mb_handles_units() {
        let value = parse_mem_to_mb("1024MiB / 2GiB");
        assert!(matches!(value, Ok(v) if (v - 1024.0).abs() < 1e-6));
        let value = parse_mem_to_mb("1GiB / 2GiB");
        assert!(matches!(value, Ok(v) if (v - 1024.0).abs() < 1e-6));
        let value = parse_mem_to_mb("512KiB / 1GiB");
        assert!(matches!(value, Ok(v) if (v - 0.5).abs() < 1e-6));
    }

    #[test]
    fn parse_mem_to_mb_rejects_unknown_units() {
        match parse_mem_to_mb("10MB / 1GB") {
            Ok(_) => panic!("expected unit error"),
            Err(err) => assert!(err.to_string().contains("unknown memory unit")),
        }
    }
}
