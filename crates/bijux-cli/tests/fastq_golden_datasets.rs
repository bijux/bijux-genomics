#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent().and_then(|p| p.parent()) {
        Some(root) => root.to_path_buf(),
        None => panic!("repo root not found"),
    }
}

fn read_json(path: &PathBuf) -> Value {
    let data = fs::read_to_string(path).unwrap_or_else(|err| panic!("read json failed: {err}"));
    serde_json::from_str(&data).unwrap_or_else(|err| panic!("parse json failed: {err}"))
}

fn find_metrics_json(root: &Path, sample_id: &str) -> PathBuf {
    let run_dir = root.join("artifacts/bench/preprocess").join(sample_id);
    let mut stack = vec![run_dir];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.file_name().and_then(|s| s.to_str()) == Some("metrics.json") {
                    return path;
                }
            }
        }
    }
    panic!("metrics.json not found under preprocess run dir");
}

fn find_run_summary(root: &Path, sample_id: &str) -> PathBuf {
    let run_dir = root.join("artifacts/bench/preprocess").join(sample_id);
    let mut stack = vec![run_dir];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.file_name().and_then(|s| s.to_str()) == Some("run_summary.json") {
                    return path;
                }
            }
        }
    }
    panic!("run_summary.json not found under preprocess run dir");
}

#[test]
fn golden_datasets_science_checks() {
    if std::env::var("BIJUX_E2E").is_err() {
        return;
    }

    let root = repo_root();
    let artifacts = root.join("artifacts");

    let se_r1 = root.join("tests/data/fastq/ERR769587/ERR769587.fastq.gz");
    let mut cmd = assert_cmd::cargo_bin_cmd!("bijux");
    cmd.current_dir(&root).args([
        "bench",
        "fastq",
        "preprocess",
        "--sample-id",
        "golden-se",
        "--r1",
        se_r1.to_str().unwrap(),
        "--out",
        artifacts.to_str().unwrap(),
    ]);
    cmd.assert().success();

    let delta_path = find_metrics_json(&root, "golden-se");
    let delta = read_json(&delta_path);
    let read_retention = delta["delta_metrics"]["read_retention"]
        .as_f64()
        .unwrap_or(0.0);
    assert!((0.5..=1.0).contains(&read_retention));

    let pe_r1 = root.join("tests/data/fastq/ERR2112797/ERR2112797_1.fastq.gz");
    let pe_r2 = root.join("tests/data/fastq/ERR2112797/ERR2112797_2.fastq.gz");
    let mut cmd = assert_cmd::cargo_bin_cmd!("bijux");
    cmd.current_dir(&root).args([
        "bench",
        "fastq",
        "preprocess",
        "--sample-id",
        "golden-pe",
        "--r1",
        pe_r1.to_str().unwrap(),
        "--r2",
        pe_r2.to_str().unwrap(),
        "--out",
        artifacts.to_str().unwrap(),
        "--force-merge",
    ]);
    cmd.assert().success();

    let delta_path = find_metrics_json(&root, "golden-pe");
    let delta = read_json(&delta_path);
    let base_retention = delta["delta_metrics"]["base_retention"]
        .as_f64()
        .unwrap_or(0.0);
    assert!((0.5..=1.0).contains(&base_retention));

    let run_summary_path = find_run_summary(&root, "golden-pe");
    let summary = read_json(&run_summary_path);
    let merge_enabled = summary["pipeline_decisions"]["merge"]["enabled"]
        .as_bool()
        .unwrap_or(false);
    assert!(merge_enabled);
}
