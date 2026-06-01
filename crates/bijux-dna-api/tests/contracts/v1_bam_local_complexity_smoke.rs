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

#[test]
fn write_local_complexity_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.complexity");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_complexity_smoke_report()?;
    assert_eq!(report_path, repo_root.join("target/local-smoke/bam.complexity/complexity.json"));
    assert!(report_path.is_file(), "local-smoke BAM complexity report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.complexity"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.complexity.local_smoke.report.v1")
    );
    assert_eq!(payload["sample_id"], serde_json::json!("core-v1-complexity-insufficient"));
    assert_eq!(payload["expectation_matched"], serde_json::json!(true));
    assert_eq!(payload["method"], serde_json::json!("preseq"));
    assert_eq!(payload["observed_total_reads"], serde_json::json!(3));
    assert_eq!(payload["observed_unique_reads"], serde_json::json!(2));
    assert_eq!(payload["estimated_unique_reads"], serde_json::Value::Null);
    assert_eq!(
        payload["insufficient_data_reason"],
        serde_json::json!("insufficient_observed_unique_reads_for_complexity_extrapolation")
    );

    let complexity_report = repo_root.join(
        payload["complexity_report"]
            .as_str()
            .unwrap_or_else(|| panic!("complexity_report path missing")),
    );
    let preseq =
        repo_root.join(payload["preseq"].as_str().unwrap_or_else(|| panic!("preseq path missing")));
    let complexity_summary = repo_root.join(
        payload["complexity_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("complexity_summary path missing")),
    );
    let stage_metrics = repo_root.join(
        payload["stage_metrics"].as_str().unwrap_or_else(|| panic!("stage_metrics path missing")),
    );
    for path in [&complexity_report, &preseq, &complexity_summary, &stage_metrics] {
        assert!(path.is_file(), "governed BAM complexity artifact must exist: {}", path.display());
    }

    let observation_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&complexity_report)?)?;
    assert_eq!(
        observation_json["schema_version"],
        serde_json::json!("bijux.bam.complexity.local_smoke.observation.v1")
    );
    assert_eq!(observation_json["method"], serde_json::json!("preseq"));
    assert_eq!(observation_json["observed_total_reads"], serde_json::json!(3));
    assert_eq!(observation_json["observed_unique_reads"], serde_json::json!(2));
    assert_eq!(observation_json["projected_unique_reads"], serde_json::json!([[3, 2]]));
    assert_eq!(
        observation_json["insufficient_data_reason"],
        serde_json::json!("insufficient_observed_unique_reads_for_complexity_extrapolation")
    );

    let preseq_body = std::fs::read_to_string(&preseq)?;
    assert_eq!(preseq_body, "3\t2\n");

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&complexity_summary)?)?;
    assert_eq!(summary_json["schema_version"], serde_json::json!("bijux.bam.complexity.v1"));
    assert_eq!(summary_json["method"], serde_json::json!("preseq"));
    assert_eq!(summary_json["observed_total_reads"], serde_json::json!(3));
    assert_eq!(summary_json["observed_unique_reads"], serde_json::json!(2));
    assert_eq!(summary_json["projected_unique_reads"], serde_json::json!([[3, 2]]));
    assert_eq!(summary_json["estimated_unique_reads"], serde_json::Value::Null);
    assert_eq!(summary_json["min_reads"], serde_json::json!(3));
    assert_eq!(
        summary_json["insufficient_data_reason"],
        serde_json::json!("insufficient_observed_unique_reads_for_complexity_extrapolation")
    );

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.complexity.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["method"], serde_json::json!("preseq"));
    assert_eq!(stage_metrics_json["observed_total_reads"], serde_json::json!(3));
    assert_eq!(stage_metrics_json["observed_unique_reads"], serde_json::json!(2));
    assert_eq!(
        stage_metrics_json["insufficient_data_reason"],
        serde_json::json!("insufficient_observed_unique_reads_for_complexity_extrapolation")
    );

    Ok(())
}
