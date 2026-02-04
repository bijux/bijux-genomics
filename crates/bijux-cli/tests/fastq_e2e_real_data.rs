#![allow(clippy::unnecessary_wraps)]

use anyhow::Result;
use std::path::PathBuf;

const SAMPLE_ID: &str = "e2e";

fn e2e_inputs() -> Result<(PathBuf, PathBuf, PathBuf)> {
    if std::env::var("BIJUX_E2E").is_err() {
        return Err(anyhow::anyhow!("BIJUX_E2E not set"));
    }
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let r1 = manifest_dir.join("tests/data/fastq/ERR2112797/ERR2112797_1.fastq.gz");
    let r2 = manifest_dir.join("tests/data/fastq/ERR2112797/ERR2112797_2.fastq.gz");
    if !r1.exists() || !r2.exists() {
        return Err(anyhow::anyhow!("missing test FASTQ inputs"));
    }
    let tmp = bijux_infra::temp_dir("bijux")?;
    Ok((r1, r2, tmp.keep()))
}

fn run_stage(args: &[&str]) {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("bijux");
    cmd.args(args);
    cmd.assert().success();
}

#[test]
#[ignore = "e2e (run with BIJUX_E2E=1 and make test-e2e)"]
fn fastq_e2e_validate_trim_filter_e2e() -> Result<()> {
    let Ok((r1, _r2, out)) = e2e_inputs() else {
        return Ok(());
    };
    run_stage(&[
        "fastq",
        "validate",
        "--env",
        "docker",
        "--tools",
        "seqtk",
        "--sample-id",
        SAMPLE_ID,
        "--r1",
        r1.to_string_lossy().as_ref(),
        "--out",
        out.to_string_lossy().as_ref(),
    ]);
    run_stage(&[
        "fastq",
        "trim",
        "--env",
        "docker",
        "--tools",
        "fastp",
        "--sample-id",
        SAMPLE_ID,
        "--r1",
        r1.to_string_lossy().as_ref(),
        "--out",
        out.to_string_lossy().as_ref(),
    ]);
    run_stage(&[
        "fastq",
        "filter",
        "--env",
        "docker",
        "--tools",
        "seqkit",
        "--sample-id",
        SAMPLE_ID,
        "--r1",
        r1.to_string_lossy().as_ref(),
        "--out",
        out.to_string_lossy().as_ref(),
    ]);
    let trim_metrics = out.join("artifacts/bench/trim/e2e/tools/fastp/metrics.json");
    let filter_metrics = out.join("artifacts/bench/filter/e2e/tools/seqkit/metrics.json");
    assert!(trim_metrics.exists(), "missing trim metrics");
    assert!(filter_metrics.exists(), "missing filter metrics");
    Ok(())
}

#[test]
#[ignore = "e2e (run with BIJUX_E2E=1 and make test-e2e)"]
fn fastq_e2e_stats_qc_post_e2e() -> Result<()> {
    let Ok((r1, _r2, out)) = e2e_inputs() else {
        return Ok(());
    };
    run_stage(&[
        "fastq",
        "stats",
        "--env",
        "docker",
        "--tools",
        "seqkit_stats",
        "--sample-id",
        SAMPLE_ID,
        "--r1",
        r1.to_string_lossy().as_ref(),
        "--out",
        out.to_string_lossy().as_ref(),
    ]);
    run_stage(&[
        "fastq",
        "qc-post",
        "--env",
        "docker",
        "--tools",
        "multiqc",
        "--sample-id",
        SAMPLE_ID,
        "--r1",
        r1.to_string_lossy().as_ref(),
        "--out",
        out.to_string_lossy().as_ref(),
    ]);
    let stats_metrics = out.join("artifacts/bench/stats/e2e/tools/seqkit_stats/metrics.json");
    let qc_metrics = out.join("artifacts/bench/qc_post/e2e/tools/multiqc/metrics.json");
    assert!(stats_metrics.exists(), "missing stats metrics");
    assert!(qc_metrics.exists(), "missing qc_post metrics");
    Ok(())
}

#[test]
#[ignore = "e2e (run with BIJUX_E2E=1 and make test-e2e)"]
#[allow(clippy::too_many_lines)]
fn fastq_e2e_preprocess_pipeline_e2e() -> Result<()> {
    let Ok((r1, r2, out)) = e2e_inputs() else {
        return Ok(());
    };
    run_stage(&[
        "fastq",
        "preprocess",
        "--env",
        "docker",
        "--sample-id",
        SAMPLE_ID,
        "--r1",
        r1.to_string_lossy().as_ref(),
        "--r2",
        r2.to_string_lossy().as_ref(),
        "--out",
        out.to_string_lossy().as_ref(),
    ]);
    let manifest = out.join("artifacts/bench/preprocess/e2e/manifest.json");
    assert!(manifest.exists(), "missing preprocess manifest");
    Ok(())
}
