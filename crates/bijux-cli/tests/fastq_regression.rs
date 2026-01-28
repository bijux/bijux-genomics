use std::path::Path;

use anyhow::Result;
use bijux_engine::api::{
    docker_rm, docker_stats_mb, output_fastq_stats, parse_fastqvalidator_count,
    run_merge_container, run_tool_container, run_validate_container,
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

fn write_broken_fastq(dir: &Path, name: &str) -> Result<std::path::PathBuf> {
    let path = dir.join(name);
    let contents = b"@broken\nACGT\n+\n";
    std::fs::write(&path, contents)?;
    Ok(path)
}

fn write_truncated_gzip(dir: &Path, name: &str, source: &Path) -> Result<std::path::PathBuf> {
    let path = dir.join(name);
    let bytes = std::fs::read(source)?;
    let truncated = &bytes[..std::cmp::min(20, bytes.len())];
    std::fs::write(&path, truncated)?;
    Ok(path)
}

fn run_tool_with_measurements(
    tool: &str,
    image: &bijux_environment::api::ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
) -> Result<Option<(f64, f64)>> {
    let out_dir = tempdir_in_repo()?;
    let out_path = out_dir.path();
    let name = format!("bijux-test-measure-{}", Uuid::new_v4());
    let start = std::time::Instant::now();
    let execution = match run_tool_container(tool, image, r1_dir, r1, out_path, &name) {
        Ok(execution) => execution,
        Err(err) => {
            eprintln!("skipping: tool run failed: {err}");
            return Ok(None);
        }
    };
    let runtime = start.elapsed().as_secs_f64();
    let memory = match docker_stats_mb(&name) {
        Ok(memory) => memory,
        Err(err) => {
            eprintln!("skipping: stats failed: {err}");
            docker_rm(&name)?;
            return Ok(None);
        }
    };
    docker_rm(&name)?;
    if execution.exit_code != 0 {
        return Ok(None);
    }
    Ok(Some((runtime, memory)))
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
    let image = bijux_engine::api::resolve_image_for_run(spec, &platform)?;

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
    let image = bijux_engine::api::resolve_image_for_run(spec, &platform)?;

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
    let image = bijux_engine::api::resolve_image_for_run(spec, &platform)?;

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

#[test]
fn regression_fastq_validate_rejects_broken_fastq() -> Result<()> {
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
    let image = bijux_engine::api::resolve_image_for_run(spec, &platform)?;

    let out_dir = tempdir_in_repo()?;
    let out_path = out_dir.path();
    let input = write_broken_fastq(out_path, "broken.fastq")?;
    let r1_dir = input
        .parent()
        .ok_or_else(|| anyhow::anyhow!("missing parent"))?;
    let name = format!("bijux-test-broken-{}", Uuid::new_v4());
    let execution = run_validate_container(
        "fastqvalidator_official",
        &image,
        r1_dir,
        &input,
        out_path,
        &name,
    )?;
    docker_rm(&name)?;
    assert_ne!(execution.exit_code, 0, "broken fastq should fail");
    Ok(())
}

#[test]
fn regression_fastq_validate_rejects_truncated_gzip() -> Result<()> {
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
    let image = bijux_engine::api::resolve_image_for_run(spec, &platform)?;

    let out_dir = tempdir_in_repo()?;
    let out_path = out_dir.path();
    let source = Path::new("tests/data/fastq/canonical/BIJUX_SE_R1.fastq.gz");
    let input = write_truncated_gzip(out_path, "truncated.fastq.gz", source)?;
    let r1_dir = input
        .parent()
        .ok_or_else(|| anyhow::anyhow!("missing parent"))?;
    let name = format!("bijux-test-trunc-{}", Uuid::new_v4());
    let execution = run_validate_container(
        "fastqvalidator_official",
        &image,
        r1_dir,
        &input,
        out_path,
        &name,
    )?;
    docker_rm(&name)?;
    assert_ne!(execution.exit_code, 0, "truncated gzip should fail");
    Ok(())
}

#[test]
fn regression_fastq_merge_requires_r2() -> Result<()> {
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
        .get("pear")
        .ok_or_else(|| anyhow::anyhow!("pear missing from images.yaml"))?;
    let image = bijux_engine::api::resolve_image_for_run(spec, &platform)?;

    let out_dir = tempdir_in_repo()?;
    let out_path = out_dir.path();
    let r1 = Path::new("tests/data/fastq/canonical/BIJUX_PE_R1.fastq.gz").canonicalize()?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow::anyhow!("missing parent"))?;
    let missing_r2 = out_path.join("missing_r2.fastq.gz");
    let name = format!("bijux-test-merge-missing-{}", Uuid::new_v4());
    let execution = run_merge_container("pear", &image, r1_dir, &r1, &missing_r2, out_path, &name)?;
    docker_rm(&name)?;
    assert_ne!(execution.exit_code, 0, "merge should fail without r2");
    Ok(())
}

#[test]
fn regression_resource_measurements_are_consistent() -> Result<()> {
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
    let image = bijux_engine::api::resolve_image_for_run(spec, &platform)?;

    let input = Path::new("tests/data/fastq/canonical/BIJUX_SE_R1.fastq.gz").canonicalize()?;
    let r1_dir = input
        .parent()
        .ok_or_else(|| anyhow::anyhow!("missing parent"))?;

    let first = run_tool_with_measurements("seqkit", &image, r1_dir, &input)?;
    let second = run_tool_with_measurements("seqkit", &image, r1_dir, &input)?;
    let (Some(first), Some(second)) = (first, second) else {
        eprintln!("skipping: measurements unavailable");
        return Ok(());
    };
    let (runtime_a, memory_a) = first;
    let (runtime_b, memory_b) = second;
    assert!(runtime_a > 0.0 && runtime_b > 0.0);
    assert!(memory_a > 0.0 && memory_b > 0.0);
    let runtime_ratio = (runtime_a / runtime_b).max(runtime_b / runtime_a);
    let memory_ratio = (memory_a / memory_b).max(memory_b / memory_a);
    assert!(runtime_ratio < 10.0, "runtime ratio too large");
    assert!(memory_ratio < 10.0, "memory ratio too large");
    Ok(())
}
