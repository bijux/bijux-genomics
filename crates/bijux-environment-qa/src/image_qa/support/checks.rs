use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use bijux_core::contract::ExecutionContract;
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

pub(crate) fn push_arg(cmd: &mut Command, args: &mut Vec<String>, value: impl Into<String>) {
    let value = value.into();
    cmd.arg(&value);
    args.push(value);
}

pub(crate) fn command_string(args: &[String]) -> String {
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

pub(crate) fn docker_wait_timeout(container_id: &str, timeout: Duration) -> Result<i32> {
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

pub(crate) fn docker_logs(container_id: &str) -> Result<String> {
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

pub(crate) struct ExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub output_fastq: Option<std::path::PathBuf>,
    pub command: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct TrimExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub output_r1: std::path::PathBuf,
    pub output_r2: Option<std::path::PathBuf>,
    pub command: String,
}
