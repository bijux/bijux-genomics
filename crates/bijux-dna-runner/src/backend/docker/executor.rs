use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use bijux_dna_environment::api::{
    docker_image_exists, resolve_image, PlatformSpec, ResolvedImage, RuntimeKind, ToolImageSpec,
};
use tracing::warn;

#[derive(Debug, Clone)]
pub struct StageExecutionPlan {
    pub tool: String,
    pub container_args: Vec<String>,
    pub expected_outputs: Vec<PathBuf>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct ExecutionAssessment {
    pub success: bool,
    pub missing_outputs: Vec<PathBuf>,
    pub reason: Option<String>,
}

/// Resolve a concrete image reference for execution and verify local availability.
///
/// # Errors
/// Returns an error if resolution fails or required local images are unavailable.
pub fn resolve_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage> {
    let image = resolve_image(spec, platform)?;
    match platform.runner {
        RuntimeKind::Docker => resolve_docker_image_for_run(spec, platform, image),
        RuntimeKind::Apptainer | RuntimeKind::Singularity => {
            resolve_apptainer_image_for_run(spec, platform, image)
        }
    }
}

fn resolve_docker_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
    image: ResolvedImage,
) -> Result<ResolvedImage> {
    if std::env::var("BIJUX_SKIP_IMAGE_CHECK").is_ok() {
        return Ok(image);
    }
    if docker_image_exists(&image) {
        return Ok(image);
    }
    if spec.digest.is_some() {
        let fallback = ResolvedImage {
            full_name: format!(
                "{}/{}:{}-{}",
                platform.image_prefix, spec.tool, spec.version, platform.arch
            ),
            arch: platform.arch.clone(),
            runner: platform.runner,
        };
        if docker_image_exists(&fallback) {
            warn!(
                "digest image missing locally; falling back to tag {}",
                fallback.full_name
            );
            return Ok(fallback);
        }
    }
    Err(anyhow!("docker image not found: {}", image.full_name))
}

fn resolve_apptainer_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
    image: ResolvedImage,
) -> Result<ResolvedImage> {
    let candidates = apptainer_image_candidates(spec, platform);
    let image_path = candidates
        .iter()
        .find(|path| path.is_file())
        .cloned()
        .unwrap_or_else(|| candidates[0].clone());
    let resolved = ResolvedImage {
        full_name: image_path.display().to_string(),
        arch: image.arch,
        runner: image.runner,
    };
    if std::env::var("BIJUX_SKIP_IMAGE_CHECK").is_ok() || image_path.is_file() {
        return Ok(resolved);
    }
    Err(anyhow!(
        "apptainer image not found for tool {}. checked: {}",
        spec.tool,
        candidates
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn apptainer_image_candidates(spec: &ToolImageSpec, platform: &PlatformSpec) -> Vec<PathBuf> {
    let registry_root = apptainer_registry_root(&platform.container_dir);
    let mut candidates = vec![platform.container_dir.join(format!("{}.sif", spec.tool))];
    if let Some(digest) = spec.digest.as_deref() {
        let normalized_digest = digest.strip_prefix("sha256:").unwrap_or(digest);
        candidates.push(
            registry_root
                .join(&spec.tool)
                .join(format!("{normalized_digest}.sif")),
        );
        candidates.push(registry_root.join(&spec.tool).join(format!("{digest}.sif")));
    } else if let Some(unique_sif) = unique_registry_sif(&registry_root, &spec.tool) {
        candidates.push(unique_sif);
    }
    candidates.dedup();
    candidates
}

fn unique_registry_sif(registry_root: &Path, tool: &str) -> Option<PathBuf> {
    let tool_dir = registry_root.join(tool);
    let entries = fs::read_dir(&tool_dir).ok()?;
    let mut sifs = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sif"))
        .collect::<Vec<_>>();
    sifs.sort();
    if sifs.len() == 1 {
        return sifs.into_iter().next();
    }
    None
}

fn apptainer_registry_root(container_dir: &Path) -> PathBuf {
    let parent = container_dir.parent();
    let grandparent = parent.and_then(Path::parent);
    let is_flat_lunarc_dir = container_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "sif")
        && parent
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "apptainer");
    if is_flat_lunarc_dir {
        return grandparent.unwrap_or(container_dir).to_path_buf();
    }
    container_dir.to_path_buf()
}

/// Execute a container plan and collect command output.
///
/// # Errors
/// Returns an error if Docker invocation fails or the container cannot be observed.
pub fn execute_plan(
    plan: &StageExecutionPlan,
    image: &ResolvedImage,
    input_mount: &Path,
    output_mount: &Path,
    container_name: &str,
) -> Result<ExecutionOutput> {
    let input_mount = format!("{}:/data/input:ro", input_mount.display());
    let output_mount = format!("{}:/data/output", output_mount.display());

    let mut cmd = Command::new("docker");
    let mut args: Vec<String> = Vec::new();
    push_arg(&mut cmd, &mut args, "run");
    push_arg(&mut cmd, &mut args, "-d");
    push_arg(&mut cmd, &mut args, "--rm=false");
    push_arg(&mut cmd, &mut args, "--name");
    push_arg(&mut cmd, &mut args, container_name);
    push_arg(&mut cmd, &mut args, "-v");
    push_arg(&mut cmd, &mut args, input_mount);
    push_arg(&mut cmd, &mut args, "-v");
    push_arg(&mut cmd, &mut args, output_mount);
    for (key, value) in &plan.env {
        push_arg(&mut cmd, &mut args, "-e");
        push_arg(&mut cmd, &mut args, format!("{key}={value}"));
    }
    push_arg(&mut cmd, &mut args, image.full_name.clone());
    for arg in &plan.container_args {
        push_arg(&mut cmd, &mut args, arg.clone());
    }

    let output = cmd.output().context("run docker")?;
    if !output.status.success() {
        return Err(anyhow!("docker run failed for {}", plan.tool));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        return Err(anyhow!("missing container id for {}", plan.tool));
    }
    let exit_code = docker_wait(&id)?;
    let stdout = docker_logs(&id)?;
    let stderr = String::new();
    let command = command_string(&args);
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        command,
    })
}

/// Execute a container plan with timeout enforcement.
///
/// # Errors
/// Returns an error if execution fails or timeout is reached.
pub fn execute_plan_with_timeout(
    plan: &StageExecutionPlan,
    image: &ResolvedImage,
    input_mount: &Path,
    output_mount: &Path,
    container_name: &str,
    timeout: std::time::Duration,
) -> Result<ExecutionOutput> {
    let input_mount = format!("{}:/data/input:ro", input_mount.display());
    let output_mount = format!("{}:/data/output", output_mount.display());

    let mut cmd = Command::new("docker");
    let mut args: Vec<String> = Vec::new();
    push_arg(&mut cmd, &mut args, "run");
    push_arg(&mut cmd, &mut args, "-d");
    push_arg(&mut cmd, &mut args, "--rm=false");
    push_arg(&mut cmd, &mut args, "--name");
    push_arg(&mut cmd, &mut args, container_name);
    push_arg(&mut cmd, &mut args, "-v");
    push_arg(&mut cmd, &mut args, input_mount);
    push_arg(&mut cmd, &mut args, "-v");
    push_arg(&mut cmd, &mut args, output_mount);
    for (key, value) in &plan.env {
        push_arg(&mut cmd, &mut args, "-e");
        push_arg(&mut cmd, &mut args, format!("{key}={value}"));
    }
    push_arg(&mut cmd, &mut args, image.full_name.clone());
    for arg in &plan.container_args {
        push_arg(&mut cmd, &mut args, arg.clone());
    }

    let output = cmd.output().context("run docker")?;
    if !output.status.success() {
        return Err(anyhow!("docker run failed for {}", plan.tool));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        return Err(anyhow!("missing container id for {}", plan.tool));
    }
    let exit_code = docker_wait_timeout(&id, timeout)?;
    let stdout = docker_logs(&id)?;
    let stderr = String::new();
    let command = command_string(&args);
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        command,
    })
}

#[must_use]
pub fn assess_execution(exit_code: i32, expected_outputs: &[PathBuf]) -> ExecutionAssessment {
    if exit_code != 0 {
        return ExecutionAssessment {
            success: false,
            missing_outputs: Vec::new(),
            reason: Some(format!("exit_code={exit_code}")),
        };
    }
    let missing: Vec<PathBuf> = expected_outputs
        .iter()
        .filter(|path| !path.exists())
        .cloned()
        .collect();
    if !missing.is_empty() {
        return ExecutionAssessment {
            success: false,
            missing_outputs: missing,
            reason: Some("missing_outputs".to_string()),
        };
    }
    ExecutionAssessment {
        success: true,
        missing_outputs: Vec::new(),
        reason: None,
    }
}

pub(crate) fn push_arg(cmd: &mut Command, args: &mut Vec<String>, value: impl Into<String>) {
    let value = value.into();
    cmd.arg(&value);
    args.push(value);
}

pub(crate) fn command_string(args: &[String]) -> String {
    format!("docker {}", args.join(" "))
}

/// Wait for container completion and parse its exit code.
///
/// # Errors
/// Returns an error if docker wait fails or output cannot be parsed.
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

/// Wait for completion up to a timeout and return the container exit code.
///
/// # Errors
/// Returns an error if timeout is reached or docker inspection/wait fails.
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

/// Fetch container logs from Docker.
///
/// # Errors
/// Returns an error if docker logs command fails.
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

/// Read current memory usage (MB) from docker stats.
///
/// # Errors
/// Returns an error if docker stats command fails or parsing is invalid.
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

/// Parse docker memory usage string (e.g. `123.4MiB / 4GiB`) into MB.
///
/// # Errors
/// Returns an error if the input format or unit is unsupported.
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

/// Remove a container forcefully.
///
/// # Errors
/// Returns an error if docker rm fails.
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
    use super::{assess_execution, resolve_image_for_run};
    use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
    use std::path::PathBuf;

    #[test]
    fn assess_execution_success() -> anyhow::Result<()> {
        let dir = bijux_dna_infra::temp_dir("bijux")?;
        let output = dir.path().join("out.data");
        bijux_dna_infra::atomic_write_bytes(&output, b"ok")?;
        let assessment = assess_execution(0, &[output]);
        assert!(assessment.success);
        Ok(())
    }

    #[test]
    fn assess_execution_missing_outputs() {
        let missing = PathBuf::from("/tmp/missing.data");
        let assessment = assess_execution(0, &[missing]);
        assert!(!assessment.success);
        assert_eq!(assessment.reason.as_deref(), Some("missing_outputs"));
    }

    #[test]
    fn assess_execution_partial_outputs() -> anyhow::Result<()> {
        let dir = bijux_dna_infra::temp_dir("bijux")?;
        let present = dir.path().join("present.data");
        bijux_dna_infra::atomic_write_bytes(&present, b"ok")?;
        let missing = dir.path().join("missing.data");
        let assessment = assess_execution(0, &[present, missing]);
        assert!(!assessment.success);
        assert_eq!(assessment.reason.as_deref(), Some("missing_outputs"));
        Ok(())
    }

    #[test]
    fn assess_execution_bad_exit_code() {
        let assessment = assess_execution(1, &[]);
        assert!(!assessment.success);
        assert_eq!(assessment.reason.as_deref(), Some("exit_code=1"));
    }

    #[test]
    fn resolve_image_for_run_uses_platform_sif_for_apptainer() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-runner-apptainer")?;
        let sif_path = temp.path().join("fastqc.sif");
        bijux_dna_infra::atomic_write_bytes(&sif_path, b"sif")?;
        let platform = PlatformSpec {
            name: "lunarc-apptainer".to_string(),
            runner: RuntimeKind::Apptainer,
            container_dir: temp.path().to_path_buf(),
            image_prefix: "bijuxdna".to_string(),
            arch: "amd64".to_string(),
        };
        let spec = ToolImageSpec {
            tool: "fastqc".to_string(),
            version: "latest-pinned".to_string(),
            digest: None,
            enabled: None,
            shipping_policy: None,
        };

        let image = resolve_image_for_run(&spec, &platform)?;

        assert_eq!(image.full_name, sif_path.display().to_string());
        assert_eq!(image.runner, RuntimeKind::Apptainer);
        Ok(())
    }

    #[test]
    fn resolve_image_for_run_rejects_missing_apptainer_sif() {
        let platform = PlatformSpec {
            name: "lunarc-apptainer".to_string(),
            runner: RuntimeKind::Apptainer,
            container_dir: PathBuf::from("/tmp/does-not-exist-bijux"),
            image_prefix: "bijuxdna".to_string(),
            arch: "amd64".to_string(),
        };
        let spec = ToolImageSpec {
            tool: "fastqc".to_string(),
            version: "latest-pinned".to_string(),
            digest: None,
            enabled: None,
            shipping_policy: None,
        };

        let error = resolve_image_for_run(&spec, &platform).expect_err("missing sif must fail");
        assert!(error.to_string().contains("apptainer image not found"));
    }

    #[test]
    fn resolve_image_for_run_uses_digest_pinned_apptainer_sif_when_flat_name_is_absent(
    ) -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-runner-apptainer-registry")?;
        let flat_dir = temp.path().join("apptainer").join("sif");
        let registry_dir = temp.path().join("fastqc");
        bijux_dna_infra::ensure_dir(&flat_dir)?;
        bijux_dna_infra::ensure_dir(&registry_dir)?;
        let sif_path = registry_dir.join("abc123.sif");
        bijux_dna_infra::atomic_write_bytes(&sif_path, b"sif")?;
        let platform = PlatformSpec {
            name: "lunarc-apptainer".to_string(),
            runner: RuntimeKind::Apptainer,
            container_dir: flat_dir,
            image_prefix: "bijuxdna".to_string(),
            arch: "amd64".to_string(),
        };
        let spec = ToolImageSpec {
            tool: "fastqc".to_string(),
            version: "latest-pinned".to_string(),
            digest: Some("sha256:abc123".to_string()),
            enabled: None,
            shipping_policy: None,
        };

        let image = resolve_image_for_run(&spec, &platform)?;

        assert_eq!(image.full_name, sif_path.display().to_string());
        assert_eq!(image.runner, RuntimeKind::Apptainer);
        Ok(())
    }

    #[test]
    fn resolve_image_for_run_uses_single_registry_sif_when_digest_is_missing() -> anyhow::Result<()>
    {
        let temp = bijux_dna_infra::temp_dir("bijux-runner-apptainer-unique-registry")?;
        let flat_dir = temp.path().join("apptainer").join("sif");
        let registry_dir = temp.path().join("seqkit");
        bijux_dna_infra::ensure_dir(&flat_dir)?;
        bijux_dna_infra::ensure_dir(&registry_dir)?;
        let sif_path = registry_dir.join("pending.sif");
        bijux_dna_infra::atomic_write_bytes(&sif_path, b"sif")?;
        let platform = PlatformSpec {
            name: "lunarc-apptainer".to_string(),
            runner: RuntimeKind::Apptainer,
            container_dir: flat_dir,
            image_prefix: "bijuxdna".to_string(),
            arch: "amd64".to_string(),
        };
        let spec = ToolImageSpec {
            tool: "seqkit".to_string(),
            version: "latest-pinned".to_string(),
            digest: None,
            enabled: None,
            shipping_policy: None,
        };

        let image = resolve_image_for_run(&spec, &platform)?;

        assert_eq!(image.full_name, sif_path.display().to_string());
        assert_eq!(image.runner, RuntimeKind::Apptainer);
        Ok(())
    }
}
