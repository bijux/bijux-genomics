use std::path::Path;

use anyhow::Result;
use bijux_domain_fastq::analyze::report::{
    write_filter_report, write_trim_report, write_validate_report,
};
use bijux_domain_fastq::stages::{
    args as bench_args, bench_fastq_filter, bench_fastq_trim, bench_fastq_validate_pre,
};
use bijux_environment::api::{load_image_catalog, load_platform};
use tempfile::TempDir;

fn tempdir_in_repo() -> Result<TempDir> {
    let cwd = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let base = cwd.join("target").join("test-tmp");
    std::fs::create_dir_all(&base)?;
    Ok(TempDir::new_in(base)?)
}

fn ensure_docker() -> bool {
    let status = std::process::Command::new("docker").arg("version").status();
    matches!(status, Ok(s) if s.success())
}

fn assert_report_ok(path: &Path) -> Result<()> {
    let bytes = std::fs::read(path)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    let records = value
        .get("records")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("records missing"))?;
    let failures = value
        .get("failures")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("failures missing"))?;
    let rankings = value
        .get("rankings")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("rankings missing"))?;
    assert_eq!(failures.len(), 0, "benchmark failures present");
    assert_eq!(records.len(), 1, "expected single tool record");
    assert!(!rankings.is_empty(), "rankings should be present");
    Ok(())
}

#[test]
fn benchmark_gate_validate_trim_filter() -> Result<()> {
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
    let out_dir = tempdir_in_repo()?;
    let out_path = out_dir.path().to_path_buf();
    let sample_id = "bench-gate".to_string();
    let r1 = Path::new("tests/data/fastq/canonical/BIJUX_SE_R1.fastq.gz").canonicalize()?;

    let validate_args = bench_args::BenchFastqValidateArgs {
        sample_id: sample_id.clone(),
        r1: r1.clone(),
        out: out_path.clone(),
        tools: vec!["fastqvalidator_official".to_string()],
        explain: false,
        strict: false,
    };
    let validate_outcome = bench_fastq_validate_pre(&catalog, &platform, None, &validate_args)?;
    write_validate_report(
        &validate_outcome.bench_dir,
        &validate_outcome.records,
        &validate_outcome.failures,
        validate_outcome.explain,
    )?;

    let trim_args = bench_args::BenchFastqTrimArgs {
        sample_id: sample_id.clone(),
        r1: r1.clone(),
        out: out_path.clone(),
        tools: vec!["fastp".to_string()],
        explain: false,
    };
    let trim_outcome = bench_fastq_trim(&catalog, &platform, None, &trim_args)?;
    write_trim_report(
        &trim_outcome.bench_dir,
        &trim_outcome.records,
        &trim_outcome.failures,
        trim_outcome.explain,
    )?;

    let filter_args = bench_args::BenchFastqFilterArgs {
        sample_id: sample_id.clone(),
        r1,
        out: out_path.clone(),
        tools: vec!["seqkit".to_string()],
        explain: false,
    };
    let filter_outcome = bench_fastq_filter(&catalog, &platform, None, &filter_args)?;
    write_filter_report(
        &filter_outcome.bench_dir,
        &filter_outcome.records,
        &filter_outcome.failures,
        filter_outcome.explain,
    )?;

    let validate_report = out_path
        .join("artifacts")
        .join("bench")
        .join("validate")
        .join(&sample_id)
        .join("report.json");
    let trim_report = out_path
        .join("artifacts")
        .join("bench")
        .join("trim")
        .join(&sample_id)
        .join("report.json");
    let filter_report = out_path
        .join("artifacts")
        .join("bench")
        .join("filter")
        .join(&sample_id)
        .join("report.json");

    assert_report_ok(&validate_report)?;
    assert_report_ok(&trim_report)?;
    assert_report_ok(&filter_report)?;
    Ok(())
}
