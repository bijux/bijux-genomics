pub fn run_validate_container_with_timeout(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    out_dir: &Path,
    container_name: &str,
    timeout: Duration,
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
        "seqkit_stats" => {
            push_arg(&mut cmd, &mut args, "seqkit");
            push_arg(&mut cmd, &mut args, "stats");
            push_arg(&mut cmd, &mut args, "-a");
            push_arg(&mut cmd, &mut args, "-T");
            push_arg(&mut cmd, &mut args, input_path.clone());
        }
        "multiqc" => {
            push_arg(&mut cmd, &mut args, "-o");
            push_arg(&mut cmd, &mut args, "/data/output");
            push_arg(&mut cmd, &mut args, "/data/input");
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

pub fn run_multiqc_container_with_timeout(
    image: &ResolvedImage,
    input_dir: &Path,
    out_dir: &Path,
    container_name: &str,
    timeout: Duration,
) -> Result<ExecutionOutput> {
    let input_mount = format!("{}:/data/input:ro", input_dir.display());
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
    push_arg(&mut cmd, &mut args, "-o");
    push_arg(&mut cmd, &mut args, "/data/output");
    push_arg(&mut cmd, &mut args, "/data/input");

    let output = cmd.output().context("run docker")?;
    if !output.status.success() {
        return Err(anyhow!("docker run failed for multiqc"));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        return Err(anyhow!("missing container id for multiqc"));
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

#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
pub fn run_merge_container_with_timeout(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    container_name: &str,
    timeout: Duration,
) -> Result<MergeExecutionOutput> {
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
    let r2_name = r2
        .file_name()
        .ok_or_else(|| anyhow!("r2 filename missing"))?
        .to_string_lossy()
        .to_string();
    let input_r1 = format!("/data/input/{r1_name}");
    let input_r2 = format!("/data/input/{r2_name}");

    let (merged, unmerged_r1, unmerged_r2) = match tool {
        "pear" => {
            let prefix = "pear";
            push_arg(&mut cmd, &mut args, "pear");
            push_arg(&mut cmd, &mut args, "-f");
            push_arg(&mut cmd, &mut args, input_r1.clone());
            push_arg(&mut cmd, &mut args, "-r");
            push_arg(&mut cmd, &mut args, input_r2.clone());
            push_arg(&mut cmd, &mut args, "-o");
            push_arg(&mut cmd, &mut args, format!("/data/output/{prefix}"));
            (
                out_dir.join(format!("{prefix}.assembled.fastq")),
                out_dir.join(format!("{prefix}.unassembled.forward.fastq")),
                out_dir.join(format!("{prefix}.unassembled.reverse.fastq")),
            )
        }
        "vsearch" => {
            let prefix = "vsearch";
            push_arg(&mut cmd, &mut args, "vsearch");
            push_arg(&mut cmd, &mut args, "--fastq_mergepairs");
            push_arg(&mut cmd, &mut args, input_r1.clone());
            push_arg(&mut cmd, &mut args, "--reverse");
            push_arg(&mut cmd, &mut args, input_r2.clone());
            push_arg(&mut cmd, &mut args, "--fastqout");
            push_arg(
                &mut cmd,
                &mut args,
                format!("/data/output/{prefix}.merged.fastq"),
            );
            push_arg(&mut cmd, &mut args, "--fastqout_notmerged_fwd");
            push_arg(
                &mut cmd,
                &mut args,
                format!("/data/output/{prefix}.unmerged_r1.fastq"),
            );
            push_arg(&mut cmd, &mut args, "--fastqout_notmerged_rev");
            push_arg(
                &mut cmd,
                &mut args,
                format!("/data/output/{prefix}.unmerged_r2.fastq"),
            );
            (
                out_dir.join(format!("{prefix}.merged.fastq")),
                out_dir.join(format!("{prefix}.unmerged_r1.fastq")),
                out_dir.join(format!("{prefix}.unmerged_r2.fastq")),
            )
        }
        "bbmerge" => {
            let prefix = "bbmerge";
            push_arg(&mut cmd, &mut args, format!("in1={input_r1}"));
            push_arg(&mut cmd, &mut args, format!("in2={input_r2}"));
            push_arg(
                &mut cmd,
                &mut args,
                format!("out=/data/output/{prefix}.merged.fastq"),
            );
            push_arg(
                &mut cmd,
                &mut args,
                format!("outu1=/data/output/{prefix}.unmerged_r1.fastq"),
            );
            push_arg(
                &mut cmd,
                &mut args,
                format!("outu2=/data/output/{prefix}.unmerged_r2.fastq"),
            );
            (
                out_dir.join(format!("{prefix}.merged.fastq")),
                out_dir.join(format!("{prefix}.unmerged_r1.fastq")),
                out_dir.join(format!("{prefix}.unmerged_r2.fastq")),
            )
        }
        "flash2" => {
            let prefix = "flash2";
            push_arg(&mut cmd, &mut args, "flash2");
            push_arg(&mut cmd, &mut args, "-o");
            push_arg(&mut cmd, &mut args, prefix);
            push_arg(&mut cmd, &mut args, "-d");
            push_arg(&mut cmd, &mut args, "/data/output");
            push_arg(&mut cmd, &mut args, input_r1.clone());
            push_arg(&mut cmd, &mut args, input_r2.clone());
            (
                out_dir.join(format!("{prefix}.extendedFrags.fastq")),
                out_dir.join(format!("{prefix}.notCombined_1.fastq")),
                out_dir.join(format!("{prefix}.notCombined_2.fastq")),
            )
        }
        _ => return Err(anyhow!("unsupported tool: {tool}")),
    };

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
    Ok(MergeExecutionOutput {
        exit_code,
        stdout,
        stderr,
        merged_fastq: merged,
        unmerged_r1,
        unmerged_r2,
        command,
    })
}
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};

use super::checks::{command_string, docker_logs, docker_wait_timeout, push_arg, ExecutionOutput};
use super::validation::MergeExecutionOutput;
use super::ResolvedImage;
