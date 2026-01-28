use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use bijux_environment::api::ResolvedImage;

use super::docker::{command_string, docker_logs, docker_wait, docker_wait_timeout, push_arg};

#[derive(Debug)]
pub struct ExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub output_fastq: Option<std::path::PathBuf>,
    pub command: String,
}

#[derive(Debug)]
struct ToolCommand {
    args: Vec<String>,
    output: Option<std::path::PathBuf>,
}

#[allow(clippy::too_many_lines)]
fn build_tool_command(tool: &str, input_path: &str, out_dir: &Path) -> Result<ToolCommand> {
    let mut args = Vec::new();
    let output = match tool {
        "fastp" => {
            let out_name = "fastp.fastq.gz";
            args.extend(["fastp", "-i", input_path, "-o"].map(str::to_string));
            args.push(format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        "cutadapt" => {
            let out_name = "cutadapt.fastq.gz";
            args.extend(["cutadapt", "-o"].map(str::to_string));
            args.push(format!("/data/output/{out_name}"));
            args.push(input_path.to_string());
            Some(out_dir.join(out_name))
        }
        "atropos" => {
            let out_name = "atropos.fastq.gz";
            args.extend(
                [
                    "atropos",
                    "trim",
                    "-a",
                    "AGATCGGAAGAGC",
                    "-se",
                    input_path,
                    "-o",
                ]
                .map(str::to_string),
            );
            args.push(format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        "bbduk" => {
            let out_name = "bbduk.fastq.gz";
            args.push(format!("in={input_path}"));
            args.push(format!("out=/data/output/{out_name}"));
            args.push("ref=adapters".to_string());
            Some(out_dir.join(out_name))
        }
        "adapterremoval" => {
            let out_name = "adapterremoval.fastq.gz";
            args.extend(["adapterremoval", "--file1", input_path, "--output1"].map(str::to_string));
            args.push(format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        "trimmomatic" => {
            let out_name = "trimmomatic.fastq.gz";
            args.extend(
                [
                    "trimmomatic",
                    "SE",
                    "-phred33",
                    input_path,
                    &format!("/data/output/{out_name}"),
                    "SLIDINGWINDOW:4:20",
                    "MINLEN:30",
                ]
                .map(str::to_string),
            );
            Some(out_dir.join(out_name))
        }
        "trim_galore" => {
            let basename = "trimmed";
            args.extend(
                [
                    "trim_galore",
                    "--gzip",
                    "--output_dir",
                    "/data/output",
                    "--basename",
                    basename,
                    input_path,
                ]
                .map(str::to_string),
            );
            Some(out_dir.join(format!("{basename}_trimmed.fq.gz")))
        }
        "seqpurge" => {
            let out_name = "seqpurge.fastq.gz";
            args.extend(["seqpurge", "-in1", input_path, "-out1"].map(str::to_string));
            args.push(format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        "prinseq" => {
            let prefix = "prinseq_good";
            args.extend(
                [
                    "prinseq++",
                    "-fastq",
                    input_path,
                    "-out_good",
                    &format!("/data/output/{prefix}"),
                    "-out_bad",
                    "/data/output/prinseq_bad",
                ]
                .map(str::to_string),
            );
            Some(out_dir.join(format!("{prefix}.fastq")))
        }
        "seqkit" => {
            let out_name = "seqkit.fastq.gz";
            args.extend(["seqkit", "seq", "-m", "1", "-o"].map(str::to_string));
            args.push(format!("/data/output/{out_name}"));
            args.push(input_path.to_string());
            Some(out_dir.join(out_name))
        }
        "rcorrector" => {
            args.extend(["sh", "-lc"].map(str::to_string));
            args.push(format!("rcorrector -s {input_path} -od /data/output"));
            None
        }
        "umi_tools" => {
            let out_name = "umi_tools.fastq.gz";
            args.extend(
                ["extract", "--bc-pattern=NNNN", "-I", input_path, "-S"].map(str::to_string),
            );
            args.push(format!("/data/output/{out_name}"));
            Some(out_dir.join(out_name))
        }
        _ => return Err(anyhow!("unsupported tool: {tool}")),
    };
    Ok(ToolCommand { args, output })
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
    let tool_command = build_tool_command(tool, &input_path, out_dir)?;
    for arg in tool_command.args {
        push_arg(&mut cmd, &mut args, arg);
    }
    let out_fastq = tool_command.output;

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
        output_fastq: out_fastq,
        command,
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

#[cfg(test)]
mod tests {
    use super::build_tool_command;
    use anyhow::Result;
    use std::path::Path;

    #[test]
    fn build_tool_command_fastp() -> Result<()> {
        let out_dir = Path::new("out");
        let cmd = build_tool_command("fastp", "/data/input/r1.fastq.gz", out_dir)?;
        assert!(cmd.args.iter().any(|arg| arg == "fastp"));
        let output = cmd
            .output
            .ok_or_else(|| anyhow::anyhow!("missing output"))?;
        assert_eq!(output.to_string_lossy(), "out/fastp.fastq.gz");
        Ok(())
    }

    #[test]
    fn build_tool_command_trim_galore() -> Result<()> {
        let out_dir = Path::new("out");
        let cmd = build_tool_command("trim_galore", "/data/input/r1.fastq.gz", out_dir)?;
        assert!(cmd.args.iter().any(|arg| arg == "trim_galore"));
        let output = cmd
            .output
            .ok_or_else(|| anyhow::anyhow!("missing output"))?;
        assert_eq!(output.to_string_lossy(), "out/trimmed_trimmed.fq.gz");
        Ok(())
    }

    #[test]
    fn build_tool_command_unknown_tool() {
        let out_dir = Path::new("out");
        match build_tool_command("mystery", "/data/input/r1.fastq.gz", out_dir) {
            Ok(_) => panic!("expected unsupported tool error"),
            Err(err) => assert!(err.to_string().contains("unsupported tool")),
        }
    }
}
