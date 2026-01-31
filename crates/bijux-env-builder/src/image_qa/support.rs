use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use bijux_core::ExecutionContract;
use sha2::Digest;
use tracing::warn;

pub use crate::api::ResolvedImage;
use crate::api::{docker_image_exists, resolve_image, PlatformSpec, ToolImageSpec};

pub struct StdoutLogger {
    debug: bool,
}

impl StdoutLogger {
    #[must_use]
    pub fn new() -> Self {
        Self { debug: false }
    }

    #[allow(clippy::unused_self)]
    pub fn info(&self, message: &str) {
        println!("{message}");
    }

    #[allow(clippy::unused_self)]
    pub fn debug(&self, message: &str) {
        if self.debug {
            println!("{message}");
        }
    }
}

#[must_use]
pub fn trace_enabled() -> bool {
    std::env::var("BIJUX_TRACE_ENGINE").is_ok()
}

#[must_use]
pub fn image_qa_base_dir(cwd: &Path, platform: &str) -> PathBuf {
    cwd.join("artifacts").join("image-qa").join(platform)
}

#[must_use]
pub fn image_qa_jsonl_path(cwd: &Path, platform: &str) -> PathBuf {
    image_qa_base_dir(cwd, platform).join("qa.jsonl")
}

#[must_use]
pub fn image_qa_sqlite_path(cwd: &Path, platform: &str) -> PathBuf {
    image_qa_base_dir(cwd, platform).join("qa.sqlite")
}

/// # Errors
/// Returns an error if the file cannot be read.
pub fn hash_file_sha256(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).context("read file for hash")?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct SeqkitMetrics {
    pub reads: u64,
    pub bases: u64,
    pub mean_q: f64,
    pub gc_percent: f64,
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

pub fn adapter_hit_reads(
    image: &ResolvedImage,
    mount_dir: &Path,
    fastq: &Path,
    adapter: &str,
) -> Result<u64> {
    let mount_dir = mount_dir
        .canonicalize()
        .context("resolve mount directory")?;
    let fastq = fastq.canonicalize().context("resolve fastq path")?;
    let fastq_name = fastq
        .file_name()
        .ok_or_else(|| anyhow!("fastq missing filename"))?
        .to_string_lossy()
        .to_string();
    let mut cmd = Command::new("docker");
    cmd.arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/data:ro", mount_dir.display()))
        .arg(&image.full_name)
        .arg("sh")
        .arg("-lc")
        .arg(format!(
            "seqkit grep -s -p {adapter} /data/{fastq_name} | seqkit stats -a -T -"
        ));
    let output = cmd.output().context("run seqkit grep stats")?;
    if !output.status.success() {
        return Err(anyhow!("seqkit adapter scan failed"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let metrics = parse_seqkit_stats(&stdout)?;
    Ok(metrics.reads)
}

pub fn ensure_gzip_integrity(path: &Path) -> Result<()> {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        let status = Command::new("gzip")
            .arg("-t")
            .arg(path)
            .status()
            .context("run gzip -t")?;
        if !status.success() {
            return Err(anyhow!("gzip integrity check failed: {}", path.display()));
        }
    }
    Ok(())
}

fn seqkit_stats(image: &ResolvedImage, mount_dir: &Path, fastq: &Path) -> Result<SeqkitMetrics> {
    let mount_dir = mount_dir
        .canonicalize()
        .context("resolve mount directory")?;
    let fastq = fastq.canonicalize().context("resolve fastq path")?;
    let fastq_name = fastq
        .file_name()
        .ok_or_else(|| anyhow!("fastq missing filename"))?
        .to_string_lossy()
        .to_string();

    let mut cmd = Command::new("docker");
    cmd.arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/data:ro", mount_dir.display()))
        .arg(&image.full_name)
        .arg("seqkit")
        .arg("stats")
        .arg("-a")
        .arg("-T")
        .arg(format!("/data/{fastq_name}"));

    let output = cmd.output().context("run seqkit stats")?;
    if !output.status.success() {
        return Err(anyhow!("seqkit stats failed"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_seqkit_stats(&stdout)
}

fn parse_seqkit_stats(output: &str) -> Result<SeqkitMetrics> {
    let mut lines = output.lines();
    let header = lines.next().ok_or_else(|| anyhow!("empty seqkit output"))?;
    let data = lines.next().ok_or_else(|| anyhow!("missing seqkit data"))?;
    let header_fields: Vec<&str> = header.split('\t').collect();
    let data_fields: Vec<&str> = data.split('\t').collect();
    if header_fields.len() != data_fields.len() {
        return Err(anyhow!("seqkit header/data column mismatch"));
    }
    let col = |name: &str| -> Result<&str> {
        let idx = header_fields
            .iter()
            .position(|field| field == &name)
            .ok_or_else(|| anyhow!("seqkit column missing: {name}"))?;
        data_fields
            .get(idx)
            .copied()
            .ok_or_else(|| anyhow!("seqkit data missing for {name}"))
    };
    let reads: u64 = col("num_seqs")?.parse().context("parse reads")?;
    let bases: u64 = col("sum_len")?.parse().context("parse bases")?;
    let mean_q = if header_fields.iter().any(|field| field == &"avg_qual") {
        col("avg_qual")?.parse().context("parse mean_q")?
    } else if header_fields.iter().any(|field| field == &"mean_qual") {
        col("mean_qual")?.parse().context("parse mean_q")?
    } else {
        warn!("seqkit avg_qual/mean_qual missing; defaulting mean_q to 0.0");
        0.0
    };
    let gc_percent = if let Some((idx, _)) = header_fields
        .iter()
        .enumerate()
        .find(|(_, field)| field.to_lowercase().starts_with("gc"))
    {
        data_fields
            .get(idx)
            .ok_or_else(|| anyhow!("seqkit data missing for gc"))?
            .parse()
            .context("parse gc_percent")?
    } else {
        warn!("seqkit gc column missing; defaulting gc_percent to 0.0");
        0.0
    };
    Ok(SeqkitMetrics {
        reads,
        bases,
        mean_q,
        gc_percent,
    })
}

/// # Errors
/// Returns an error when the output directory violates the contract.
pub fn validate_execution_outputs(contract: &ExecutionContract, out_dir: &Path) -> Result<()> {
    let outputs = collect_outputs(out_dir)?;

    for forbidden in &contract.forbidden_outputs {
        if outputs.iter().any(|path| matches_pattern(path, forbidden)) {
            return Err(anyhow!("forbidden output produced: {forbidden}"));
        }
    }

    for expected in &contract.expected_outputs {
        if !outputs.iter().any(|path| matches_pattern(path, expected)) {
            return Err(anyhow!("expected output missing: {expected}"));
        }
    }

    if contract.forbid_unexpected_outputs {
        for output in &outputs {
            if !contract
                .expected_outputs
                .iter()
                .any(|pattern| matches_pattern(output, pattern))
            {
                return Err(anyhow!("unexpected output produced: {output}"));
            }
        }
    }

    Ok(())
}

fn collect_outputs(root: &Path) -> Result<Vec<String>> {
    let mut results = Vec::new();
    walk_outputs(root, root, &mut results)?;
    Ok(results)
}

fn walk_outputs(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("read dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_dir() {
            walk_outputs(root, &path, out)?;
        } else if path.is_file() {
            out.push(rel);
        }
    }
    Ok(())
}

fn matches_pattern(value: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return value == pattern;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0usize;
    let starts_with_wildcard = pattern.starts_with('*');
    let ends_with_wildcard = pattern.ends_with('*');

    for (idx, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if let Some(found) = value[pos..].find(part) {
            if idx == 0 && !starts_with_wildcard && found != 0 {
                return false;
            }
            pos += found + part.len();
        } else {
            return false;
        }
    }

    if !ends_with_wildcard {
        if let Some(last) = parts.last() {
            if !last.is_empty() && !value.ends_with(last) {
                return false;
            }
        }
    }
    true
}

pub fn resolve_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage> {
    let image = resolve_image(spec, platform)?;
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

fn push_arg(cmd: &mut Command, args: &mut Vec<String>, value: impl Into<String>) {
    let value = value.into();
    cmd.arg(&value);
    args.push(value);
}

fn command_string(args: &[String]) -> String {
    format!("docker {}", args.join(" "))
}

fn docker_wait(container_id: &str) -> Result<i32> {
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

fn docker_wait_timeout(container_id: &str, timeout: Duration) -> Result<i32> {
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
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn docker_logs(container_id: &str) -> Result<String> {
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

#[allow(dead_code)]
#[derive(Debug)]
pub struct ExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub output_fastq: Option<std::path::PathBuf>,
    pub command: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct TrimExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub output_r1: std::path::PathBuf,
    pub output_r2: Option<std::path::PathBuf>,
    pub command: String,
}

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

#[allow(clippy::too_many_lines)]
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
        "fastqvalidator" | "fastqvalidator_official" => {
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
