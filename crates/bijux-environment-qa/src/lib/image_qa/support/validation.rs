#[allow(dead_code)]
#[derive(Debug)]
pub struct MergeExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub merged_fastq: std::path::PathBuf,
    pub unmerged_r1: std::path::PathBuf,
    pub unmerged_r2: std::path::PathBuf,
    pub command: String,
}

#[allow(clippy::too_many_lines)]
pub fn run_tool_container_with_timeout(
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
    let out_fastq = match tool {
        "fastp" => {
            let out_name = "fastp.fastq.gz";
            push_arg(&mut cmd, &mut args, "fastp");
            push_arg(&mut cmd, &mut args, "-i");
            push_arg(&mut cmd, &mut args, input_path.clone());
            push_arg(&mut cmd, &mut args, "-o");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        "cutadapt" => {
            let out_name = "cutadapt.fastq.gz";
            push_arg(&mut cmd, &mut args, "cutadapt");
            push_arg(&mut cmd, &mut args, "-o");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out_name}"));
            push_arg(&mut cmd, &mut args, input_path.clone());
            Some(out_dir.join(out_name))
        }
        "atropos" => {
            let out_name = "atropos.fastq.gz";
            push_arg(&mut cmd, &mut args, "atropos");
            push_arg(&mut cmd, &mut args, "trim");
            push_arg(&mut cmd, &mut args, "-a");
            push_arg(&mut cmd, &mut args, "AGATCGGAAGAGC");
            push_arg(&mut cmd, &mut args, "-se");
            push_arg(&mut cmd, &mut args, input_path.clone());
            push_arg(&mut cmd, &mut args, "-o");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        "bbduk" => {
            let out_name = "bbduk.fastq.gz";
            push_arg(&mut cmd, &mut args, format!("in={input_path}"));
            push_arg(&mut cmd, &mut args, format!("out=/data/output/{out_name}"));
            push_arg(&mut cmd, &mut args, "ref=adapters");
            Some(out_dir.join(out_name))
        }
        "adapterremoval" => {
            let out_name = "adapterremoval.fastq.gz";
            push_arg(&mut cmd, &mut args, "adapterremoval");
            push_arg(&mut cmd, &mut args, "--file1");
            push_arg(&mut cmd, &mut args, input_path.clone());
            push_arg(&mut cmd, &mut args, "--output1");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        "trimmomatic" => {
            let out_name = "trimmomatic.fastq.gz";
            push_arg(&mut cmd, &mut args, "trimmomatic");
            push_arg(&mut cmd, &mut args, "SE");
            push_arg(&mut cmd, &mut args, "-phred33");
            push_arg(&mut cmd, &mut args, input_path.clone());
            push_arg(&mut cmd, &mut args, format!("/data/output/{out_name}"));
            push_arg(&mut cmd, &mut args, "SLIDINGWINDOW:4:20");
            push_arg(&mut cmd, &mut args, "MINLEN:30");
            Some(out_dir.join(out_name))
        }
        "trim_galore" => {
            let basename = "trimmed";
            push_arg(&mut cmd, &mut args, "trim_galore");
            push_arg(&mut cmd, &mut args, "--gzip");
            push_arg(&mut cmd, &mut args, "--output_dir");
            push_arg(&mut cmd, &mut args, "/data/output");
            push_arg(&mut cmd, &mut args, "--basename");
            push_arg(&mut cmd, &mut args, basename);
            push_arg(&mut cmd, &mut args, input_path.clone());
            Some(out_dir.join(format!("{basename}_trimmed.fq.gz")))
        }
        "seqpurge" => {
            let out_name = "seqpurge.fastq.gz";
            push_arg(&mut cmd, &mut args, "seqpurge");
            push_arg(&mut cmd, &mut args, "-in1");
            push_arg(&mut cmd, &mut args, input_path.clone());
            push_arg(&mut cmd, &mut args, "-out1");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        "prinseq" => {
            let prefix = "prinseq_good";
            push_arg(&mut cmd, &mut args, "prinseq++");
            push_arg(&mut cmd, &mut args, "-fastq");
            push_arg(&mut cmd, &mut args, input_path.clone());
            push_arg(&mut cmd, &mut args, "-out_good");
            push_arg(&mut cmd, &mut args, format!("/data/output/{prefix}"));
            push_arg(&mut cmd, &mut args, "-out_bad");
            push_arg(&mut cmd, &mut args, "/data/output/prinseq_bad");
            Some(out_dir.join(format!("{prefix}.fastq")))
        }
        "seqkit" => {
            let out_name = "seqkit.fastq.gz";
            push_arg(&mut cmd, &mut args, "seqkit");
            push_arg(&mut cmd, &mut args, "seq");
            push_arg(&mut cmd, &mut args, "-m");
            push_arg(&mut cmd, &mut args, "1");
            push_arg(&mut cmd, &mut args, "-o");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out_name}"));
            push_arg(&mut cmd, &mut args, input_path.clone());
            Some(out_dir.join(out_name))
        }
        "rcorrector" => {
            push_arg(&mut cmd, &mut args, "sh");
            push_arg(&mut cmd, &mut args, "-lc");
            push_arg(
                &mut cmd,
                &mut args,
                format!("rcorrector -s {input_path} -od /data/output"),
            );
            None
        }
        "umi_tools" => {
            let out_name = "umi_tools.fastq.gz";
            push_arg(&mut cmd, &mut args, "extract");
            push_arg(&mut cmd, &mut args, "--bc-pattern=NNNN");
            push_arg(&mut cmd, &mut args, "-I");
            push_arg(&mut cmd, &mut args, input_path.clone());
            push_arg(&mut cmd, &mut args, "-S");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out_name}"));
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
    let command = command_string(&args);
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        output_fastq: out_fastq,
        command,
    })
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn run_trim_container_with_timeout(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    container_name: &str,
    timeout: Duration,
) -> Result<TrimExecutionOutput> {
    if r2.is_none() {
        let execution = run_tool_container_with_timeout(
            tool,
            image,
            r1_dir,
            r1,
            out_dir,
            container_name,
            timeout,
        )?;
        let output_r1 = execution
            .output_fastq
            .ok_or_else(|| anyhow!("output FASTQ missing for {tool}"))?;
        return Ok(TrimExecutionOutput {
            exit_code: execution.exit_code,
            stdout: execution.stdout,
            stderr: execution.stderr,
            output_r1,
            output_r2: None,
            command: execution.command,
        });
    }

    let r2 = r2.ok_or_else(|| anyhow!("r2 path missing"))?;
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

    let (out_r1, out_r2) = match tool {
        "fastp" => {
            let out1 = "fastp_R1.fastq.gz";
            let out2 = "fastp_R2.fastq.gz";
            push_arg(&mut cmd, &mut args, "fastp");
            push_arg(&mut cmd, &mut args, "-i");
            push_arg(&mut cmd, &mut args, input_r1.clone());
            push_arg(&mut cmd, &mut args, "-I");
            push_arg(&mut cmd, &mut args, input_r2.clone());
            push_arg(&mut cmd, &mut args, "-o");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out1}"));
            push_arg(&mut cmd, &mut args, "-O");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out2}"));
            (out_dir.join(out1), out_dir.join(out2))
        }
        "cutadapt" => {
            let out1 = "cutadapt_R1.fastq.gz";
            let out2 = "cutadapt_R2.fastq.gz";
            push_arg(&mut cmd, &mut args, "cutadapt");
            push_arg(&mut cmd, &mut args, "-o");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out1}"));
            push_arg(&mut cmd, &mut args, "-p");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out2}"));
            push_arg(&mut cmd, &mut args, input_r1.clone());
            push_arg(&mut cmd, &mut args, input_r2.clone());
            (out_dir.join(out1), out_dir.join(out2))
        }
        "atropos" => {
            let out1 = "atropos_R1.fastq.gz";
            let out2 = "atropos_R2.fastq.gz";
            push_arg(&mut cmd, &mut args, "atropos");
            push_arg(&mut cmd, &mut args, "trim");
            push_arg(&mut cmd, &mut args, "-a");
            push_arg(&mut cmd, &mut args, "AGATCGGAAGAGC");
            push_arg(&mut cmd, &mut args, "-A");
            push_arg(&mut cmd, &mut args, "AGATCGGAAGAGC");
            push_arg(&mut cmd, &mut args, "-pe1");
            push_arg(&mut cmd, &mut args, input_r1.clone());
            push_arg(&mut cmd, &mut args, "-pe2");
            push_arg(&mut cmd, &mut args, input_r2.clone());
            push_arg(&mut cmd, &mut args, "-o");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out1}"));
            push_arg(&mut cmd, &mut args, "-p");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out2}"));
            (out_dir.join(out1), out_dir.join(out2))
        }
        "bbduk" => {
            let out1 = "bbduk_R1.fastq.gz";
            let out2 = "bbduk_R2.fastq.gz";
            push_arg(&mut cmd, &mut args, format!("in1={input_r1}"));
            push_arg(&mut cmd, &mut args, format!("in2={input_r2}"));
            push_arg(&mut cmd, &mut args, format!("out1=/data/output/{out1}"));
            push_arg(&mut cmd, &mut args, format!("out2=/data/output/{out2}"));
            push_arg(&mut cmd, &mut args, "ref=adapters");
            (out_dir.join(out1), out_dir.join(out2))
        }
        "adapterremoval" => {
            let out1 = "adapterremoval_R1.fastq.gz";
            let out2 = "adapterremoval_R2.fastq.gz";
            push_arg(&mut cmd, &mut args, "adapterremoval");
            push_arg(&mut cmd, &mut args, "--file1");
            push_arg(&mut cmd, &mut args, input_r1.clone());
            push_arg(&mut cmd, &mut args, "--file2");
            push_arg(&mut cmd, &mut args, input_r2.clone());
            push_arg(&mut cmd, &mut args, "--output1");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out1}"));
            push_arg(&mut cmd, &mut args, "--output2");
            push_arg(&mut cmd, &mut args, format!("/data/output/{out2}"));
            (out_dir.join(out1), out_dir.join(out2))
        }
        "trimmomatic" => {
            let out1 = "trimmomatic_R1.fastq.gz";
            let out2 = "trimmomatic_R2.fastq.gz";
            let unpaired1 = "trimmomatic_R1_unpaired.fastq.gz";
            let unpaired2 = "trimmomatic_R2_unpaired.fastq.gz";
            push_arg(&mut cmd, &mut args, "trimmomatic");
            push_arg(&mut cmd, &mut args, "PE");
            push_arg(&mut cmd, &mut args, "-phred33");
            push_arg(&mut cmd, &mut args, input_r1.clone());
            push_arg(&mut cmd, &mut args, input_r2.clone());
            push_arg(&mut cmd, &mut args, format!("/data/output/{out1}"));
            push_arg(&mut cmd, &mut args, format!("/data/output/{unpaired1}"));
            push_arg(&mut cmd, &mut args, format!("/data/output/{out2}"));
            push_arg(&mut cmd, &mut args, format!("/data/output/{unpaired2}"));
            push_arg(&mut cmd, &mut args, "SLIDINGWINDOW:4:20");
            push_arg(&mut cmd, &mut args, "MINLEN:30");
            (out_dir.join(out1), out_dir.join(out2))
        }
        "trim_galore" => {
            let basename = "trimmed";
            let out1 = format!("{basename}_val_1.fq.gz");
            let out2 = format!("{basename}_val_2.fq.gz");
            push_arg(&mut cmd, &mut args, "trim_galore");
            push_arg(&mut cmd, &mut args, "--paired");
            push_arg(&mut cmd, &mut args, "--gzip");
            push_arg(&mut cmd, &mut args, "--output_dir");
            push_arg(&mut cmd, &mut args, "/data/output");
            push_arg(&mut cmd, &mut args, "--basename");
            push_arg(&mut cmd, &mut args, basename);
            push_arg(&mut cmd, &mut args, input_r1.clone());
            push_arg(&mut cmd, &mut args, input_r2.clone());
            (out_dir.join(out1), out_dir.join(out2))
        }
        _ => return Err(anyhow!("unsupported tool for paired trim: {tool}")),
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
    Ok(TrimExecutionOutput {
        exit_code,
        stdout,
        stderr,
        output_r1: out_r1,
        output_r2: Some(out_r2),
        command,
    })
}
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};

use super::core::{
    command_string, docker_logs, docker_wait_timeout, push_arg, ExecutionOutput,
    TrimExecutionOutput,
};
use super::ResolvedImage;
