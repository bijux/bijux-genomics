#![allow(clippy::unreadable_literal)]

use anyhow::Result;
use std::path::{Path, PathBuf};

struct RepoRootOverrideGuard {
    previous: Option<std::ffi::OsString>,
}

impl RepoRootOverrideGuard {
    fn install(root: &Path) -> Self {
        let previous = std::env::var_os("BIJUX_REPO_ROOT");
        std::env::set_var("BIJUX_REPO_ROOT", root);
        Self { previous }
    }
}

impl Drop for RepoRootOverrideGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            std::env::set_var("BIJUX_REPO_ROOT", previous);
        } else {
            std::env::remove_var("BIJUX_REPO_ROOT");
        }
    }
}

fn repo_root() -> Result<PathBuf> {
    crate::support::repo_root()
}

fn assert_json_f64(value: &serde_json::Value, expected: f64, label: &str) {
    let observed = value.as_f64().unwrap_or_else(|| panic!("{label} must serialize as f64"));
    assert!(
        (observed - expected).abs() <= 1e-12,
        "{label} drifted: observed {observed}, expected {expected}"
    );
}

#[test]
fn write_local_insert_size_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("runs/bench/local-smoke/bam.insert_size");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_insert_size_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("runs/bench/local-smoke/bam.insert_size/insert_size.json")
    );
    assert!(report_path.is_file(), "local-smoke BAM insert-size report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.insert_size"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.insert_size.local_smoke.report.v1")
    );
    assert_eq!(payload["sample_id"], serde_json::json!("human_like_insert_size_triplet"));
    assert_eq!(payload["expectation_matched"], serde_json::json!(true));
    assert_eq!(payload["method"], serde_json::json!("picard"));
    assert_eq!(payload["read_pairs"], serde_json::json!(3));
    assert_eq!(payload["median_insert_size"], serde_json::json!(20.0));
    assert_eq!(payload["mean_insert_size"], serde_json::json!(21.666666666666668));
    assert_json_f64(&payload["standard_deviation"], 6.236095644623236, "report standard_deviation");
    assert_eq!(payload["min_insert_size"], serde_json::json!(15));
    assert_eq!(payload["max_insert_size"], serde_json::json!(30));
    assert_eq!(payload["insufficient_pairs_reason"], serde_json::Value::Null);

    let insert_size_report = repo_root.join(
        payload["insert_size_report"]
            .as_str()
            .unwrap_or_else(|| panic!("insert_size_report path missing")),
    );
    let insert_size_histogram = repo_root.join(
        payload["insert_size_histogram"]
            .as_str()
            .unwrap_or_else(|| panic!("insert_size_histogram path missing")),
    );
    let insert_size_summary = repo_root.join(
        payload["insert_size_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("insert_size_summary path missing")),
    );
    let stage_metrics = repo_root.join(
        payload["stage_metrics"].as_str().unwrap_or_else(|| panic!("stage_metrics path missing")),
    );
    for path in [&insert_size_report, &insert_size_histogram, &insert_size_summary, &stage_metrics]
    {
        assert!(path.is_file(), "governed BAM insert-size artifact must exist: {}", path.display());
    }

    let raw_report = std::fs::read_to_string(&insert_size_report)?;
    assert!(raw_report.contains("MEDIAN_INSERT_SIZE"));
    assert!(raw_report.contains("READ_PAIRS"));
    assert!(raw_report.contains("STANDARD_DEVIATION"));

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&insert_size_summary)?)?;
    assert_eq!(summary_json["schema_version"], serde_json::json!("bijux.bam.insert_size.v1"));
    assert_eq!(summary_json["stage_id"], serde_json::json!("bam.insert_size"));
    assert_eq!(summary_json["report_present"], serde_json::json!(true));
    assert_eq!(summary_json["histogram_present"], serde_json::json!(true));
    assert_eq!(summary_json["read_pairs"], serde_json::json!(3));
    assert_eq!(summary_json["median_insert_size"], serde_json::json!(20.0));
    assert_eq!(summary_json["mean_insert_size"], serde_json::json!(21.666666666666668));
    assert_json_f64(
        &summary_json["standard_deviation"],
        6.236095644623236,
        "summary standard_deviation",
    );
    assert_eq!(summary_json["min_insert_size"], serde_json::json!(15));
    assert_eq!(summary_json["max_insert_size"], serde_json::json!(30));
    assert_eq!(summary_json["insufficient_pairs_reason"], serde_json::Value::Null);

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.insert_size.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["method"], serde_json::json!("picard"));
    assert_eq!(stage_metrics_json["read_pairs"], serde_json::json!(3));
    assert_eq!(stage_metrics_json["median_insert_size"], serde_json::json!(20.0));
    assert_eq!(stage_metrics_json["mean_insert_size"], serde_json::json!(21.666666666666668));
    assert_json_f64(
        &stage_metrics_json["standard_deviation"],
        6.236095644623236,
        "stage_metrics standard_deviation",
    );
    assert_eq!(stage_metrics_json["min_insert_size"], serde_json::json!(15));
    assert_eq!(stage_metrics_json["max_insert_size"], serde_json::json!(30));
    assert_eq!(stage_metrics_json["insufficient_pairs_reason"], serde_json::Value::Null);
    assert_eq!(stage_metrics_json["expectation_matched"], serde_json::json!(true));

    Ok(())
}
