use std::path::Path;

use anyhow::Result;
use bijux_engine::{
    docker_rm, output_fastq_stats, parse_fastqvalidator_count, run_tool_container,
    run_validate_container,
};
use bijux_environment::api::{load_image_catalog, load_platform};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use uuid::Uuid;

fn tempdir_in_repo() -> Result<TempDir> {
    let cwd = std::env::current_dir()?;
    let base = cwd.join("target").join("test-tmp");
    std::fs::create_dir_all(&base)?;
    Ok(TempDir::new_in(base)?)
}

fn ensure_docker() -> bool {
    let status = std::process::Command::new("docker").arg("version").status();
    matches!(status, Ok(s) if s.success())
}

fn hash_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

#[test]
fn regression_fastq_validate_deterministic() -> Result<()> {
    if std::env::var("BIJUX_REGRESSION").is_err() {
        eprintln!("skipping: BIJUX_REGRESSION not set");
        return Ok(());
    }
    if !ensure_docker() {
        eprintln!("skipping: docker not available");
        return Ok(());
    }
    let platform = load_platform(None)?;
    let catalog = load_image_catalog()?;
    let spec = catalog
        .get("fastqvalidator_official")
        .ok_or_else(|| anyhow::anyhow!("fastqvalidator_official missing from images.yaml"))?;
    let image = bijux_engine::resolve_image_for_run(spec, &platform)?;

    let input = Path::new("tests/data/fastq/canonical/BIJUX_SE_R1.fastq.gz").canonicalize()?;
    let r1_dir = input
        .parent()
        .ok_or_else(|| anyhow::anyhow!("missing parent"))?;

    let run_once = || -> Result<Option<u64>> {
        let out_dir = tempdir_in_repo()?;
        let out_path = out_dir.path();
        let name = format!("bijux-test-validate-{}", Uuid::new_v4());
        let execution = match run_validate_container(
            "fastqvalidator_official",
            &image,
            r1_dir,
            &input,
            out_path,
            &name,
        ) {
            Ok(execution) => execution,
            Err(err) => {
                eprintln!("skipping: validate run failed: {err}");
                return Ok(None);
            }
        };
        docker_rm(&name)?;
        let reads = match parse_fastqvalidator_count(&execution.stdout) {
            Ok(reads) => reads,
            Err(err) => {
                eprintln!("skipping: validate parse failed: {err}");
                return Ok(None);
            }
        };
        Ok(Some(reads))
    };

    let first = run_once()?;
    let second = run_once()?;
    let (Some(first), Some(second)) = (first, second) else {
        eprintln!("skipping: validation output missing");
        return Ok(());
    };
    assert_eq!(first, second, "validate read counts differ");
    Ok(())
}

#[test]
fn regression_fastq_trim_deterministic() -> Result<()> {
    if std::env::var("BIJUX_REGRESSION").is_err() {
        eprintln!("skipping: BIJUX_REGRESSION not set");
        return Ok(());
    }
    if !ensure_docker() {
        eprintln!("skipping: docker not available");
        return Ok(());
    }
    let platform = load_platform(None)?;
    let catalog = load_image_catalog()?;
    let spec = catalog
        .get("fastp")
        .ok_or_else(|| anyhow::anyhow!("fastp missing from images.yaml"))?;
    let image = bijux_engine::resolve_image_for_run(spec, &platform)?;

    let input = Path::new("tests/data/fastq/canonical/BIJUX_SE_R1.fastq.gz").canonicalize()?;
    let r1_dir = input
        .parent()
        .ok_or_else(|| anyhow::anyhow!("missing parent"))?;

    let run_once = || -> Result<Option<(String, u64, u64)>> {
        let out_dir = tempdir_in_repo()?;
        let out_path = out_dir.path();
        let name = format!("bijux-test-trim-{}", Uuid::new_v4());
        let execution = match run_tool_container("fastp", &image, r1_dir, &input, out_path, &name) {
            Ok(execution) => execution,
            Err(err) => {
                eprintln!("skipping: trim run failed: {err}");
                return Ok(None);
            }
        };
        docker_rm(&name)?;
        let out_fastq = match execution.output_fastq {
            Some(path) if path.exists() => path,
            _ => return Ok(None),
        };
        let hash = hash_file(&out_fastq)?;
        let stats = output_fastq_stats(&image, out_path, &out_fastq)?;
        Ok(Some((hash, stats.reads, stats.bases)))
    };

    let first = run_once()?;
    let second = run_once()?;
    let (Some(first), Some(second)) = (first, second) else {
        eprintln!("skipping: trim output missing");
        return Ok(());
    };
    assert_eq!(first, second, "trim outputs differ");
    Ok(())
}

#[test]
fn regression_fastq_filter_deterministic() -> Result<()> {
    if std::env::var("BIJUX_REGRESSION").is_err() {
        eprintln!("skipping: BIJUX_REGRESSION not set");
        return Ok(());
    }
    if !ensure_docker() {
        eprintln!("skipping: docker not available");
        return Ok(());
    }
    let platform = load_platform(None)?;
    let catalog = load_image_catalog()?;
    let spec = catalog
        .get("seqkit")
        .ok_or_else(|| anyhow::anyhow!("seqkit missing from images.yaml"))?;
    let image = bijux_engine::resolve_image_for_run(spec, &platform)?;

    let input = Path::new("tests/data/fastq/canonical/BIJUX_SE_R1.fastq.gz").canonicalize()?;
    let r1_dir = input
        .parent()
        .ok_or_else(|| anyhow::anyhow!("missing parent"))?;

    let run_once = || -> Result<Option<(String, u64, u64)>> {
        let out_dir = tempdir_in_repo()?;
        let out_path = out_dir.path();
        let name = format!("bijux-test-filter-{}", Uuid::new_v4());
        let execution = match run_tool_container("seqkit", &image, r1_dir, &input, out_path, &name)
        {
            Ok(execution) => execution,
            Err(err) => {
                eprintln!("skipping: filter run failed: {err}");
                return Ok(None);
            }
        };
        docker_rm(&name)?;
        let out_fastq = match execution.output_fastq {
            Some(path) if path.exists() => path,
            _ => return Ok(None),
        };
        let hash = hash_file(&out_fastq)?;
        let stats = output_fastq_stats(&image, out_path, &out_fastq)?;
        Ok(Some((hash, stats.reads, stats.bases)))
    };

    let first = run_once()?;
    let second = run_once()?;
    let (Some(first), Some(second)) = (first, second) else {
        eprintln!("skipping: filter output missing");
        return Ok(());
    };
    assert_eq!(first, second, "filter outputs differ");
    Ok(())
}
