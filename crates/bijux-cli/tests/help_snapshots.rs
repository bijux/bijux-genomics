use std::fs;

use anyhow::Result;

fn assert_help_contains(stage: &str, snapshot_path: &str, required_flags: &[&str]) -> Result<()> {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("bijux");
    let output = cmd.args(["fastq", stage, "--help"]).output()?;
    assert!(output.status.success(), "help command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in required_flags {
        assert!(
            stdout.contains(flag),
            "missing flag {flag} in help for {stage}"
        );
    }
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let snapshot_path = std::path::Path::new(&manifest_dir).join(snapshot_path);
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert!(
        stdout.contains(snapshot.trim()),
        "help output for {stage} missing snapshot content"
    );
    Ok(())
}

#[test]
fn fastq_trim_help_snapshot() -> Result<()> {
    assert_help_contains(
        "trim",
        "tests/snapshots/fastq_trim_help.txt",
        &[
            "--r1",
            "--out",
            "--sample-id",
            "--tools",
            "--list-tools",
            "--list-adapter-presets",
            "--list-adapters",
            "--adapter-bank-preset",
            "--adapter-bank",
            "--adapter-bank-file",
            "--enable-adapter",
            "--disable-adapter",
            "--polyx-preset",
            "--contaminant-preset",
        ],
    )?;
    Ok(())
}

#[test]
fn fastq_validate_help_snapshot() -> Result<()> {
    assert_help_contains(
        "validate-pre",
        "tests/snapshots/fastq_validate_help.txt",
        &[
            "--r1",
            "--out",
            "--sample-id",
            "--tools",
            "--strict",
            "--list-tools",
        ],
    )?;
    Ok(())
}

#[test]
fn fastq_filter_help_snapshot() -> Result<()> {
    assert_help_contains(
        "filter",
        "tests/snapshots/fastq_filter_help.txt",
        &["--list-tools", "--dry-run"],
    )?;
    Ok(())
}

#[test]
fn fastq_merge_help_snapshot() -> Result<()> {
    assert_help_contains(
        "merge",
        "tests/snapshots/fastq_merge_help.txt",
        &["--r1", "--r2", "--out", "--sample-id", "--tools"],
    )?;
    Ok(())
}

#[test]
fn fastq_stats_help_snapshot() -> Result<()> {
    assert_help_contains(
        "stats-neutral",
        "tests/snapshots/fastq_stats_help.txt",
        &["--r1", "--out", "--sample-id", "--tools", "--list-tools"],
    )?;
    Ok(())
}

#[test]
fn fastq_preprocess_help_snapshot() -> Result<()> {
    assert_help_contains(
        "preprocess",
        "tests/snapshots/fastq_preprocess_help.txt",
        &[
            "--r1",
            "--out",
            "--sample-id",
            "--auto",
            "--objective",
            "--bench-corpus",
            "--list-adapter-presets",
            "--list-adapters",
            "--adapter-bank-preset",
            "--adapter-bank",
            "--adapter-bank-file",
            "--enable-adapter",
            "--disable-adapter",
            "--polyx-preset",
            "--contaminant-preset",
        ],
    )?;
    Ok(())
}
