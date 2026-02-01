use anyhow::Result;
use std::path::PathBuf;

#[test]
#[ignore = "slow e2e (run with BIJUX_E2E=1 and make test-e2e)"]
#[allow(clippy::too_many_lines)]
fn fastq_e2e_pipeline_real_data() -> Result<()> {
    if std::env::var("BIJUX_E2E").is_err() {
        eprintln!("skipping e2e (set BIJUX_E2E=1 to run)");
        return Ok(());
    }

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let r1 = manifest_dir.join("tests/data/fastq/ERR2112797/ERR2112797_1.fastq.gz");
    let r2 = manifest_dir.join("tests/data/fastq/ERR2112797/ERR2112797_2.fastq.gz");
    if !r1.exists() || !r2.exists() {
        return Err(anyhow::anyhow!("missing test FASTQ inputs"));
    }

    let tmp = tempfile::tempdir()?;
    let out = tmp.path();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("bijux");
    cmd.args([
        "fastq",
        "validate",
        "--env",
        "docker",
        "--tools",
        "seqtk",
        "--sample-id",
        "e2e",
        "--r1",
        r1.to_string_lossy().as_ref(),
        "--out",
        out.to_string_lossy().as_ref(),
    ]);
    cmd.assert().success();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("bijux");
    cmd.args([
        "fastq",
        "trim",
        "--env",
        "docker",
        "--tools",
        "fastp",
        "--sample-id",
        "e2e",
        "--r1",
        r1.to_string_lossy().as_ref(),
        "--out",
        out.to_string_lossy().as_ref(),
    ]);
    cmd.assert().success();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("bijux");
    cmd.args([
        "fastq",
        "merge",
        "--env",
        "docker",
        "--tools",
        "vsearch",
        "--sample-id",
        "e2e",
        "--r1",
        r1.to_string_lossy().as_ref(),
        "--r2",
        r2.to_string_lossy().as_ref(),
        "--out",
        out.to_string_lossy().as_ref(),
    ]);
    cmd.assert().success();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("bijux");
    cmd.args([
        "fastq",
        "filter",
        "--env",
        "docker",
        "--tools",
        "seqkit",
        "--sample-id",
        "e2e",
        "--r1",
        r1.to_string_lossy().as_ref(),
        "--out",
        out.to_string_lossy().as_ref(),
    ]);
    cmd.assert().success();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("bijux");
    cmd.args([
        "fastq",
        "stats",
        "--env",
        "docker",
        "--tools",
        "seqkit_stats",
        "--sample-id",
        "e2e",
        "--r1",
        r1.to_string_lossy().as_ref(),
        "--out",
        out.to_string_lossy().as_ref(),
    ]);
    cmd.assert().success();

    let trim_metrics = out.join("artifacts/bench/trim/e2e/tools/fastp/metrics.json");
    let filter_metrics = out.join("artifacts/bench/filter/e2e/tools/seqkit/metrics.json");
    assert!(trim_metrics.exists(), "trim metrics missing");
    assert!(filter_metrics.exists(), "filter metrics missing");

    let trim_json = std::fs::read_to_string(trim_metrics)?;
    let filter_json = std::fs::read_to_string(filter_metrics)?;
    assert!(trim_json.contains("delta_metrics"));
    assert!(filter_json.contains("delta_metrics"));

    Ok(())
}
