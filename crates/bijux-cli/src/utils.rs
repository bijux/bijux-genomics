use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use bijux_environment::api::ResolvedImage;
use sha2::{Digest, Sha256};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Debug)]
pub struct ExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub output_fastq: Option<PathBuf>,
}

#[derive(Debug)]
pub struct MergeExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub merged_fastq: PathBuf,
    pub unmerged_r1: PathBuf,
    pub unmerged_r2: PathBuf,
}

#[allow(clippy::too_many_lines)]
pub fn run_tool_container(
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
    cmd.arg("run")
        .arg("-d")
        .arg("--rm=false")
        .arg("--name")
        .arg(container_name)
        .arg("-v")
        .arg(input_mount)
        .arg("-v")
        .arg(output_mount)
        .arg(&image.full_name);

    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let input_path = format!("/data/input/{r1_name}");
    let out_fastq = match tool {
        "fastp" => {
            let out_name = "fastp.fastq.gz";
            cmd.arg("fastp")
                .arg("-i")
                .arg(&input_path)
                .arg("-o")
                .arg(format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        "cutadapt" => {
            let out_name = "cutadapt.fastq.gz";
            cmd.arg("cutadapt")
                .arg("-o")
                .arg(format!("/data/output/{out_name}"))
                .arg(&input_path);
            Some(out_dir.join(out_name))
        }
        "bbduk" => {
            let out_name = "bbduk.fastq.gz";
            cmd.arg(format!("in={input_path}"))
                .arg(format!("out=/data/output/{out_name}"))
                .arg("ref=adapters");
            Some(out_dir.join(out_name))
        }
        "adapterremoval" => {
            let out_name = "adapterremoval.fastq.gz";
            cmd.arg("adapterremoval")
                .arg("--file1")
                .arg(&input_path)
                .arg("--output1")
                .arg(format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        "trimmomatic" => {
            let out_name = "trimmomatic.fastq.gz";
            cmd.arg("trimmomatic")
                .arg("SE")
                .arg("-phred33")
                .arg(&input_path)
                .arg(format!("/data/output/{out_name}"))
                .arg("SLIDINGWINDOW:4:20")
                .arg("MINLEN:30");
            Some(out_dir.join(out_name))
        }
        "trim_galore" => {
            let basename = "trimmed";
            cmd.arg("trim_galore")
                .arg("--gzip")
                .arg("--output_dir")
                .arg("/data/output")
                .arg("--basename")
                .arg(basename)
                .arg(&input_path);
            Some(out_dir.join(format!("{basename}_trimmed.fq.gz")))
        }
        "prinseq" => {
            let prefix = "prinseq_good";
            cmd.arg("prinseq++")
                .arg("-fastq")
                .arg(&input_path)
                .arg("-out_good")
                .arg(format!("/data/output/{prefix}"))
                .arg("-out_bad")
                .arg("/data/output/prinseq_bad");
            Some(out_dir.join(format!("{prefix}.fastq")))
        }
        "seqkit" => {
            let out_name = "seqkit.fastq.gz";
            cmd.arg("seqkit")
                .arg("seq")
                .arg("-m")
                .arg("1")
                .arg("-o")
                .arg(format!("/data/output/{out_name}"))
                .arg(&input_path);
            Some(out_dir.join(out_name))
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
    let exit_code = docker_wait(&id)?;
    let stdout = docker_logs(&id)?;
    let stderr = String::new();
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        output_fastq: out_fastq,
    })
}

#[allow(clippy::too_many_lines)]
pub fn run_tool_container_with_timeout(
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
    cmd.arg("run")
        .arg("-d")
        .arg("--rm=false")
        .arg("--name")
        .arg(container_name)
        .arg("-v")
        .arg(input_mount)
        .arg("-v")
        .arg(output_mount)
        .arg(&image.full_name);

    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let input_path = format!("/data/input/{r1_name}");
    let out_fastq = match tool {
        "fastp" => {
            let out_name = "fastp.fastq.gz";
            cmd.arg("fastp")
                .arg("-i")
                .arg(&input_path)
                .arg("-o")
                .arg(format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        "cutadapt" => {
            let out_name = "cutadapt.fastq.gz";
            cmd.arg("cutadapt")
                .arg("-o")
                .arg(format!("/data/output/{out_name}"))
                .arg(&input_path);
            Some(out_dir.join(out_name))
        }
        "bbduk" => {
            let out_name = "bbduk.fastq.gz";
            cmd.arg(format!("in={input_path}"))
                .arg(format!("out=/data/output/{out_name}"))
                .arg("ref=adapters");
            Some(out_dir.join(out_name))
        }
        "adapterremoval" => {
            let out_name = "adapterremoval.fastq.gz";
            cmd.arg("adapterremoval")
                .arg("--file1")
                .arg(&input_path)
                .arg("--output1")
                .arg(format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        "trimmomatic" => {
            let out_name = "trimmomatic.fastq.gz";
            cmd.arg("trimmomatic")
                .arg("SE")
                .arg("-phred33")
                .arg(&input_path)
                .arg(format!("/data/output/{out_name}"))
                .arg("SLIDINGWINDOW:4:20")
                .arg("MINLEN:30");
            Some(out_dir.join(out_name))
        }
        "trim_galore" => {
            let basename = "trimmed";
            cmd.arg("trim_galore")
                .arg("--gzip")
                .arg("--output_dir")
                .arg("/data/output")
                .arg("--basename")
                .arg(basename)
                .arg(&input_path);
            Some(out_dir.join(format!("{basename}_trimmed.fq.gz")))
        }
        "prinseq" => {
            let prefix = "prinseq_good";
            cmd.arg("prinseq++")
                .arg("-fastq")
                .arg(&input_path)
                .arg("-out_good")
                .arg(format!("/data/output/{prefix}"))
                .arg("-out_bad")
                .arg("/data/output/prinseq_bad");
            Some(out_dir.join(format!("{prefix}.fastq")))
        }
        "seqkit" => {
            let out_name = "seqkit.fastq.gz";
            cmd.arg("seqkit")
                .arg("seq")
                .arg("-m")
                .arg("1")
                .arg("-o")
                .arg(format!("/data/output/{out_name}"))
                .arg(&input_path);
            Some(out_dir.join(out_name))
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
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        output_fastq: out_fastq,
    })
}

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
    cmd.arg("run")
        .arg("-d")
        .arg("--rm=false")
        .arg("--name")
        .arg(container_name)
        .arg("-v")
        .arg(input_mount)
        .arg("-v")
        .arg(output_mount)
        .arg(&image.full_name);

    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let input_path = format!("/data/input/{r1_name}");

    match tool {
        "seqtk" => {
            cmd.arg("seqtk").arg("fqchk").arg(&input_path);
        }
        "fastqc" => {
            cmd.arg("fastqc")
                .arg("-v")
                .arg("--extract")
                .arg("-f")
                .arg("fastq")
                .arg("-o")
                .arg("/data/output")
                .arg(&input_path);
        }
        "fastqvalidator" => {
            cmd.arg("fastq-validator")
                .arg("--file")
                .arg(&input_path)
                .arg("--printCount");
        }
        "fqtools" => {
            cmd.arg("fqtools").arg("count").arg(&input_path);
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
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        output_fastq: None,
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
    cmd.arg("run")
        .arg("-d")
        .arg("--rm=false")
        .arg("--name")
        .arg(container_name)
        .arg("-v")
        .arg(input_mount)
        .arg("-v")
        .arg(output_mount)
        .arg(&image.full_name);

    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let input_path = format!("/data/input/{r1_name}");

    match tool {
        "seqtk" => {
            cmd.arg("seqtk").arg("fqchk").arg(&input_path);
        }
        "fastqc" => {
            cmd.arg("fastqc")
                .arg("-v")
                .arg("--extract")
                .arg("-f")
                .arg("fastq")
                .arg("-o")
                .arg("/data/output")
                .arg(&input_path);
        }
        "fastqvalidator" => {
            cmd.arg("fastq-validator")
                .arg("--file")
                .arg(&input_path)
                .arg("--printCount");
        }
        "fqtools" => {
            cmd.arg("fqtools").arg("count").arg(&input_path);
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
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        output_fastq: None,
    })
}

#[allow(clippy::too_many_lines)]
pub fn run_merge_container(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    container_name: &str,
) -> Result<MergeExecutionOutput> {
    let input_mount = format!("{}:/data/input:ro", r1_dir.display());
    let output_mount = format!("{}:/data/output", out_dir.display());

    let mut cmd = Command::new("docker");
    cmd.arg("run")
        .arg("-d")
        .arg("--rm=false")
        .arg("--name")
        .arg(container_name)
        .arg("-v")
        .arg(input_mount)
        .arg("-v")
        .arg(output_mount)
        .arg(&image.full_name);

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
            cmd.arg("pear")
                .arg("-f")
                .arg(&input_r1)
                .arg("-r")
                .arg(&input_r2)
                .arg("-o")
                .arg(format!("/data/output/{prefix}"));
            (
                out_dir.join(format!("{prefix}.assembled.fastq")),
                out_dir.join(format!("{prefix}.unassembled.forward.fastq")),
                out_dir.join(format!("{prefix}.unassembled.reverse.fastq")),
            )
        }
        "vsearch" => {
            let prefix = "vsearch";
            cmd.arg("vsearch")
                .arg("--fastq_mergepairs")
                .arg(&input_r1)
                .arg("--reverse")
                .arg(&input_r2)
                .arg("--fastqout")
                .arg(format!("/data/output/{prefix}.merged.fastq"))
                .arg("--fastqout_notmerged_fwd")
                .arg(format!("/data/output/{prefix}.unmerged_r1.fastq"))
                .arg("--fastqout_notmerged_rev")
                .arg(format!("/data/output/{prefix}.unmerged_r2.fastq"));
            (
                out_dir.join(format!("{prefix}.merged.fastq")),
                out_dir.join(format!("{prefix}.unmerged_r1.fastq")),
                out_dir.join(format!("{prefix}.unmerged_r2.fastq")),
            )
        }
        "bbmerge" => {
            let prefix = "bbmerge";
            cmd.arg(format!("in1={input_r1}"))
                .arg(format!("in2={input_r2}"))
                .arg(format!("out=/data/output/{prefix}.merged.fastq"))
                .arg(format!("outu1=/data/output/{prefix}.unmerged_r1.fastq"))
                .arg(format!("outu2=/data/output/{prefix}.unmerged_r2.fastq"));
            (
                out_dir.join(format!("{prefix}.merged.fastq")),
                out_dir.join(format!("{prefix}.unmerged_r1.fastq")),
                out_dir.join(format!("{prefix}.unmerged_r2.fastq")),
            )
        }
        "flash2" => {
            let prefix = "flash2";
            cmd.arg("flash2")
                .arg("-o")
                .arg(prefix)
                .arg("-d")
                .arg("/data/output")
                .arg(&input_r1)
                .arg(&input_r2);
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
    let exit_code = docker_wait(&id)?;
    let stdout = docker_logs(&id)?;
    let stderr = String::new();
    Ok(MergeExecutionOutput {
        exit_code,
        stdout,
        stderr,
        merged_fastq: merged,
        unmerged_r1,
        unmerged_r2,
    })
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn run_merge_container_with_timeout(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    container_name: &str,
    timeout: std::time::Duration,
) -> Result<MergeExecutionOutput> {
    let input_mount = format!("{}:/data/input:ro", r1_dir.display());
    let output_mount = format!("{}:/data/output", out_dir.display());

    let mut cmd = Command::new("docker");
    cmd.arg("run")
        .arg("-d")
        .arg("--rm=false")
        .arg("--name")
        .arg(container_name)
        .arg("-v")
        .arg(input_mount)
        .arg("-v")
        .arg(output_mount)
        .arg(&image.full_name);

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
            cmd.arg("pear")
                .arg("-f")
                .arg(&input_r1)
                .arg("-r")
                .arg(&input_r2)
                .arg("-o")
                .arg(format!("/data/output/{prefix}"));
            (
                out_dir.join(format!("{prefix}.assembled.fastq")),
                out_dir.join(format!("{prefix}.unassembled.forward.fastq")),
                out_dir.join(format!("{prefix}.unassembled.reverse.fastq")),
            )
        }
        "vsearch" => {
            let prefix = "vsearch";
            cmd.arg("vsearch")
                .arg("--fastq_mergepairs")
                .arg(&input_r1)
                .arg("--reverse")
                .arg(&input_r2)
                .arg("--fastqout")
                .arg(format!("/data/output/{prefix}.merged.fastq"))
                .arg("--fastqout_notmerged_fwd")
                .arg(format!("/data/output/{prefix}.unmerged_r1.fastq"))
                .arg("--fastqout_notmerged_rev")
                .arg(format!("/data/output/{prefix}.unmerged_r2.fastq"));
            (
                out_dir.join(format!("{prefix}.merged.fastq")),
                out_dir.join(format!("{prefix}.unmerged_r1.fastq")),
                out_dir.join(format!("{prefix}.unmerged_r2.fastq")),
            )
        }
        "bbmerge" => {
            let prefix = "bbmerge";
            cmd.arg(format!("in1={input_r1}"))
                .arg(format!("in2={input_r2}"))
                .arg(format!("out=/data/output/{prefix}.merged.fastq"))
                .arg(format!("outu1=/data/output/{prefix}.unmerged_r1.fastq"))
                .arg(format!("outu2=/data/output/{prefix}.unmerged_r2.fastq"));
            (
                out_dir.join(format!("{prefix}.merged.fastq")),
                out_dir.join(format!("{prefix}.unmerged_r1.fastq")),
                out_dir.join(format!("{prefix}.unmerged_r2.fastq")),
            )
        }
        "flash2" => {
            let prefix = "flash2";
            cmd.arg("flash2")
                .arg("-o")
                .arg(prefix)
                .arg("-d")
                .arg("/data/output")
                .arg(&input_r1)
                .arg(&input_r2);
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
    Ok(MergeExecutionOutput {
        exit_code,
        stdout,
        stderr,
        merged_fastq: merged,
        unmerged_r1,
        unmerged_r2,
    })
}

pub fn docker_wait(container_id: &str) -> Result<i32> {
    let output = Command::new("docker")
        .arg("wait")
        .arg(container_id)
        .output()
        .context("docker wait")?;
    if !output.status.success() {
        return Err(anyhow!("docker wait failed"));
    }
    let code = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<i32>()
        .context("parse exit code")?;
    Ok(code)
}

pub fn docker_wait_timeout(container_id: &str, timeout: std::time::Duration) -> Result<i32> {
    let (tx, rx) = std::sync::mpsc::channel();
    let container_id = container_id.to_string();
    let container_id_thread = container_id.clone();
    std::thread::spawn(move || {
        let output = Command::new("docker")
            .arg("wait")
            .arg(&container_id_thread)
            .output()
            .context("docker wait");
        let _ = tx.send(output);
    });
    match rx.recv_timeout(timeout) {
        Ok(result) => {
            let output = result?;
            if !output.status.success() {
                return Err(anyhow!("docker wait failed"));
            }
            let code = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<i32>()
                .context("parse exit code")?;
            Ok(code)
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            let output = Command::new("docker")
                .arg("inspect")
                .arg("--format")
                .arg("{{.State.Status}} {{.State.ExitCode}}")
                .arg(&container_id)
                .output()
                .context("docker inspect")?;
            if output.status.success() {
                let state = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<_> = state.split_whitespace().collect();
                if parts.first().copied() == Some("exited") {
                    if let Some(code) = parts.get(1).and_then(|c| c.parse::<i32>().ok()) {
                        return Ok(code);
                    }
                }
            }
            Err(anyhow!("timeout"))
        }
        Err(err) => Err(anyhow!("docker wait channel failed: {err}")),
    }
}

pub fn docker_logs(container_id: &str) -> Result<String> {
    let output = Command::new("docker")
        .arg("logs")
        .arg(container_id)
        .output()
        .context("docker logs")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("docker logs failed: {stderr}"));
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
        return Err(anyhow!("docker stats failed"));
    }
    let usage = String::from_utf8_lossy(&output.stdout)
        .split('/')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    parse_mem_to_mb(&usage)
}

pub fn parse_mem_to_mb(value: &str) -> Result<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("empty memory value"));
    }
    let number: f64 = trimmed
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>()
        .parse()
        .context("parse memory number")?;
    let unit = trimmed
        .chars()
        .skip_while(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>()
        .to_ascii_lowercase();
    let mb = match unit.as_str() {
        "b" => number / 1_000_000.0,
        "kb" => number / 1000.0,
        "kib" => number * 1.024,
        "mb" => number,
        "mib" => number * 1.048_576,
        "gb" => number * 1000.0,
        "gib" => number * 1024.0,
        _ => return Err(anyhow!("unknown memory unit {unit}")),
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
        return Err(anyhow!("docker rm failed"));
    }
    Ok(())
}

#[allow(clippy::struct_field_names)]
#[derive(Clone, Copy, Debug)]
pub struct SeqkitMetrics {
    pub reads: u64,
    pub bases: u64,
    pub mean_q: f64,
}

pub fn input_fastq_stats(
    image: &ResolvedImage,
    mount_dir: &Path,
    fastq: &Path,
) -> Result<SeqkitMetrics> {
    seqkit_stats(image, mount_dir, fastq)
}

pub fn output_fastq_stats(
    image: &ResolvedImage,
    mount_dir: &Path,
    fastq: &Path,
) -> Result<SeqkitMetrics> {
    seqkit_stats(image, mount_dir, fastq)
}

fn seqkit_stats(image: &ResolvedImage, mount_dir: &Path, fastq: &Path) -> Result<SeqkitMetrics> {
    let output_mount = format!("{}:/data/input:ro", mount_dir.display());
    let out_name = fastq
        .file_name()
        .ok_or_else(|| anyhow!("missing input filename"))?
        .to_string_lossy()
        .to_string();
    let output_path = format!("/data/input/{out_name}");

    let output = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(output_mount)
        .arg(&image.full_name)
        .arg("seqkit")
        .arg("stats")
        .arg("-a")
        .arg("-T")
        .arg(&output_path)
        .output()
        .context("run seqkit stats")?;
    if !output.status.success() {
        return Err(anyhow!("seqkit stats failed"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_seqkit_stats(&stdout)
}

fn parse_seqkit_stats(output: &str) -> Result<SeqkitMetrics> {
    let mut lines = output.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow!("missing seqkit header"))?;
    let data = lines.next().ok_or_else(|| anyhow!("missing seqkit data"))?;
    let headers: Vec<_> = header.split('\t').collect();
    let values: Vec<_> = data.split('\t').collect();
    let column = |name: &str| -> Result<&str> {
        let idx = headers
            .iter()
            .position(|h| *h == name)
            .ok_or_else(|| anyhow!("missing {name} in seqkit output"))?;
        values
            .get(idx)
            .copied()
            .ok_or_else(|| anyhow!("missing {name} value in seqkit output"))
    };
    let reads = column("num_seqs")?.parse::<u64>()?;
    let bases = column("sum_len")?.parse::<u64>()?;
    let mean_q = column("AvgQual")?.parse::<f64>()?;
    Ok(SeqkitMetrics {
        reads,
        bases,
        mean_q,
    })
}

pub fn parse_fastqvalidator_count(stdout: &str) -> Result<u64> {
    let line = stdout
        .lines()
        .find(|line| line.contains("containing") && line.contains("sequences"))
        .ok_or_else(|| anyhow!("fastqvalidator count not found"))?;
    let count = line
        .split_whitespace()
        .find_map(|part| part.parse::<u64>().ok())
        .ok_or_else(|| anyhow!("fastqvalidator count parse failed"))?;
    Ok(count)
}

pub fn hash_file_sha256(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];
    loop {
        let read = std::io::Read::read(&mut file, &mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    let digest = hasher.finalize();
    Ok(format!("sha256:{digest:x}"))
}

pub fn normalize_run_base_dir(cwd: &Path, run_base: &Path) -> PathBuf {
    if run_base.is_absolute() {
        run_base.to_path_buf()
    } else {
        cwd.join(run_base)
    }
}

pub fn bench_base_dir(out: &Path, stage: &str, sample_id: &str) -> PathBuf {
    out.join("artifacts").join(stage).join(sample_id)
}

pub fn bench_tools_dir(out: &Path, stage: &str, sample_id: &str) -> PathBuf {
    bench_base_dir(out, stage, sample_id).join("tools")
}

pub fn image_qa_base_dir(cwd: &Path, platform: &str) -> PathBuf {
    cwd.join("artifacts").join("image-qa").join(platform)
}

pub fn image_qa_jsonl_path(cwd: &Path, platform: &str) -> PathBuf {
    image_qa_base_dir(cwd, platform).join("qa.jsonl")
}

pub fn image_qa_sqlite_path(cwd: &Path, platform: &str) -> PathBuf {
    image_qa_base_dir(cwd, platform).join("qa.sqlite")
}

pub fn init_logging(log_path: &Path) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let file_appender = tracing_appender::rolling::never(
        log_path
            .parent()
            .ok_or_else(|| anyhow!("log path missing parent"))?,
        log_path
            .file_name()
            .ok_or_else(|| anyhow!("log path missing filename"))?,
    );
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .compact(),
        )
        .with(EnvFilter::from_default_env())
        .init();
    Ok(guard)
}
