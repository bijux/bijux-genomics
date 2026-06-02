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
fn write_local_endogenous_content_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.endogenous_content");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_endogenous_content_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("target/local-smoke/bam.endogenous_content/endogenous_content.json")
    );
    assert!(report_path.is_file(), "local-smoke BAM endogenous-content report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.endogenous_content"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.endogenous_content.local_smoke.report.v1")
    );
    assert_eq!(payload["sample_id"], serde_json::json!("core-v1-endogenous-partial-mapping"));
    assert_eq!(payload["expectation_matched"], serde_json::json!(true));
    assert_eq!(payload["method"], serde_json::json!("mapped_fraction_from_flagstat"));
    assert_eq!(payload["host_reference_scope"], serde_json::json!("human_host"));
    assert_eq!(payload["mapped_reads"], serde_json::json!(3));
    assert_eq!(payload["endogenous_reads"], serde_json::json!(3));
    assert_eq!(payload["total_reads"], serde_json::json!(5));
    assert_eq!(payload["endogenous_fraction"], serde_json::json!(0.6));
    assert_eq!(payload["prealignment_fraction"], serde_json::Value::Null);

    let endogenous_report = repo_root.join(
        payload["endogenous_report"]
            .as_str()
            .unwrap_or_else(|| panic!("endogenous_report path missing")),
    );
    let endogenous_summary = repo_root.join(
        payload["endogenous_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("endogenous_summary path missing")),
    );
    let stage_metrics = repo_root.join(
        payload["stage_metrics"].as_str().unwrap_or_else(|| panic!("stage_metrics path missing")),
    );
    for path in [&endogenous_report, &endogenous_summary, &stage_metrics] {
        assert!(
            path.is_file(),
            "governed BAM endogenous-content artifact must exist: {}",
            path.display()
        );
    }

    let report_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&endogenous_report)?)?;
    assert_eq!(report_json["schema_version"], serde_json::json!("bijux.bam.endogenous_content.v1"));
    assert_eq!(report_json["method"], serde_json::json!("mapped_fraction_from_flagstat"));
    assert_eq!(report_json["mapped_reads"], serde_json::json!(3));
    assert_eq!(report_json["endogenous_reads"], serde_json::json!(3));
    assert_eq!(report_json["total_reads"], serde_json::json!(5));
    assert_eq!(report_json["endogenous_fraction"], serde_json::json!(0.6));
    assert_eq!(report_json["host_reference_scope"], serde_json::json!("human_host"));

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&endogenous_summary)?)?;
    assert_eq!(summary_json, report_json);

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.endogenous_content.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["method"], serde_json::json!("mapped_fraction_from_flagstat"));
    assert_eq!(stage_metrics_json["host_reference_scope"], serde_json::json!("human_host"));
    assert_eq!(stage_metrics_json["mapped_reads"], serde_json::json!(3));
    assert_eq!(stage_metrics_json["endogenous_reads"], serde_json::json!(3));
    assert_eq!(stage_metrics_json["total_reads"], serde_json::json!(5));
    assert_eq!(stage_metrics_json["endogenous_fraction"], serde_json::json!(0.6));
    assert_eq!(stage_metrics_json["expectation_matched"], serde_json::json!(true));

    Ok(())
}
