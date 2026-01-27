use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent().and_then(|p| p.parent()) {
        Some(root) => root.to_path_buf(),
        None => panic!("repo root not found"),
    }
}

fn read_json(path: &PathBuf) -> Value {
    let data = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => panic!("read json failed: {err}"),
    };
    match serde_json::from_str(&data) {
        Ok(value) => value,
        Err(err) => panic!("parse json failed: {err}"),
    }
}

#[test]
fn fastq_preprocess_end_to_end() {
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
        "ERR769587",
        "--r1",
        se_r1
            .to_str()
            .unwrap_or_else(|| panic!("invalid utf-8 path: {se_r1:?}")),
        "--out",
        artifacts
            .to_str()
            .unwrap_or_else(|| panic!("invalid utf-8 path: {artifacts:?}")),
        "--strict",
    ]);
    cmd.assert().success();

    let delta_path = root.join("artifacts/bench/preprocess/ERR769587/delta_metrics.json");
    assert!(delta_path.is_file(), "delta_metrics.json missing");
    let delta = read_json(&delta_path);
    let Some(read_retention) = delta["read_retention"].as_f64() else {
        panic!("read_retention missing or not a number");
    };
    assert!((0.0..=1.0).contains(&read_retention));

    let pe_r1 = root.join("tests/data/fastq/ERR2112797/ERR2112797_1.fastq.gz");
    let pe_r2 = root.join("tests/data/fastq/ERR2112797/ERR2112797_2.fastq.gz");
    let mut cmd = assert_cmd::cargo_bin_cmd!("bijux");
    cmd.current_dir(&root).args([
        "bench",
        "fastq",
        "preprocess",
        "--sample-id",
        "ERR2112797",
        "--r1",
        pe_r1
            .to_str()
            .unwrap_or_else(|| panic!("invalid utf-8 path: {pe_r1:?}")),
        "--r2",
        pe_r2
            .to_str()
            .unwrap_or_else(|| panic!("invalid utf-8 path: {pe_r2:?}")),
        "--out",
        artifacts
            .to_str()
            .unwrap_or_else(|| panic!("invalid utf-8 path: {artifacts:?}")),
        "--strict",
    ]);
    cmd.assert().success();

    let delta_path = root.join("artifacts/bench/preprocess/ERR2112797/delta_metrics.json");
    assert!(delta_path.is_file(), "delta_metrics.json missing (PE)");
    let delta = read_json(&delta_path);
    let Some(base_retention) = delta["base_retention"].as_f64() else {
        panic!("base_retention missing or not a number");
    };
    assert!((0.0..=1.0).contains(&base_retention));
}
