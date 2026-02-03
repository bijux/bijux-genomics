use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use bijux_environment::api::ResolvedImage;

use super::docker::{command_string, docker_logs, docker_wait, docker_wait_timeout, push_arg};
use super::run_tool::ExecutionOutput;

pub fn run_validate_container(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    out_dir: &Path,
    container_name: &str,
) -> Result<ExecutionOutput> {
    let input_mount = format!("{}:/data/input:ro", r1_dir.display());
    let output_mount = format!("{}:/data/output", out_dir.display());

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
    push_arg(&mut cmd, &mut args, image.full_name.clone());

    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let input_path = format!("/data/input/{r1_name}");

    match tool {
        "seqtk" => {
            push_arg(&mut cmd, &mut args, "seqtk");
            push_arg(&mut cmd, &mut args, "fqchk");
            push_arg(&mut cmd, &mut args, input_path.clone());
        }
        "fastqc" => {
            push_arg(&mut cmd, &mut args, "fastqc");
            push_arg(&mut cmd, &mut args, "-v");
            push_arg(&mut cmd, &mut args, "--extract");
            push_arg(&mut cmd, &mut args, "-f");
            push_arg(&mut cmd, &mut args, "fastq");
            push_arg(&mut cmd, &mut args, "-o");
            push_arg(&mut cmd, &mut args, "/data/output");
            push_arg(&mut cmd, &mut args, input_path.clone());
        }
        "fastqvalidator" => {
            push_arg(&mut cmd, &mut args, "fastq-validator");
            push_arg(&mut cmd, &mut args, "--file");
            push_arg(&mut cmd, &mut args, input_path.clone());
            push_arg(&mut cmd, &mut args, "--printCount");
        }
        "fqtools" => {
            push_arg(&mut cmd, &mut args, "fqtools");
            push_arg(&mut cmd, &mut args, "count");
            push_arg(&mut cmd, &mut args, input_path.clone());
        }
        _ => return Err(anyhow!("unsupported tool: {tool}")),
    }

    let output = cmd.output().context("run docker")?;
    if !output.status.success() {
        return Err(anyhow!("docker run failed for {tool}"));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        return Err(anyhow!("missing container id for {tool}"));
    }
    let exit_code = docker_wait(&id)?;
    let stdout = docker_logs(&id)?;
    let stderr = String::new();
    let command = command_string(&args);
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        output_fastq: None,
        command,
    })
}

pub fn run_validate_container_with_timeout(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    out_dir: &Path,
    container_name: &str,
    timeout: std::time::Duration,
) -> Result<ExecutionOutput> {
    let input_mount = format!("{}:/data/input:ro", r1_dir.display());
    let output_mount = format!("{}:/data/output", out_dir.display());

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
    push_arg(&mut cmd, &mut args, image.full_name.clone());

    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let input_path = format!("/data/input/{r1_name}");

    match tool {
        "seqtk" => {
            push_arg(&mut cmd, &mut args, "seqtk");
            push_arg(&mut cmd, &mut args, "fqchk");
            push_arg(&mut cmd, &mut args, input_path.clone());
        }
        "fastqc" => {
            push_arg(&mut cmd, &mut args, "fastqc");
            push_arg(&mut cmd, &mut args, "-v");
            push_arg(&mut cmd, &mut args, "--extract");
            push_arg(&mut cmd, &mut args, "-f");
            push_arg(&mut cmd, &mut args, "fastq");
            push_arg(&mut cmd, &mut args, "-o");
            push_arg(&mut cmd, &mut args, "/data/output");
            push_arg(&mut cmd, &mut args, input_path.clone());
        }
        "fastqvalidator" => {
            push_arg(&mut cmd, &mut args, "fastq-validator");
            push_arg(&mut cmd, &mut args, "--file");
            push_arg(&mut cmd, &mut args, input_path.clone());
            push_arg(&mut cmd, &mut args, "--printCount");
        }
        "fqtools" => {
            push_arg(&mut cmd, &mut args, "fqtools");
            push_arg(&mut cmd, &mut args, "count");
            push_arg(&mut cmd, &mut args, input_path.clone());
        }
        _ => return Err(anyhow!("unsupported tool: {tool}")),
    }

    let output = cmd.output().context("run docker")?;
    if !output.status.success() {
        return Err(anyhow!("docker run failed for {tool}"));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        return Err(anyhow!("missing container id for {tool}"));
    }
    let exit_code = docker_wait_timeout(&id, timeout)?;
    let stdout = docker_logs(&id)?;
    let stderr = String::new();
    let command = command_string(&args);
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        output_fastq: None,
        command,
    })
}
