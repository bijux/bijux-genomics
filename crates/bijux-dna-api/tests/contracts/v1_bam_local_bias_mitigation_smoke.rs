#![cfg(feature = "bam_downstream")]

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
fn write_local_bias_mitigation_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.bias_mitigation");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_bias_mitigation_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("target/local-smoke/bam.bias_mitigation/bias_mitigation.json")
    );
    assert!(report_path.is_file(), "local-smoke BAM bias-mitigation report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.bias_mitigation"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.bias_mitigation.local_smoke.report.v1")
    );
    assert_eq!(payload["sample_id"], serde_json::json!("human_like_gc_window_ladder"));
    assert_eq!(payload["expectation_matched"], serde_json::json!(true));
    assert_eq!(payload["method"], serde_json::json!("mapdamage2"));
    assert_eq!(payload["metric_name"], serde_json::json!("gc_bias_score"));
    assert_eq!(payload["pre_mitigation_metric"], serde_json::json!(0.25));
    assert_eq!(payload["post_mitigation_metric"], serde_json::json!(0.125));
    assert_eq!(payload["metric_delta"], serde_json::json!(0.125));
    assert_eq!(payload["mitigation_projection_basis"], serde_json::json!("policy_projection"));
    assert_eq!(payload["mitigation_actions"], serde_json::json!(["gc_bias_correction"]));
    assert_eq!(payload["consumed_metrics"], serde_json::json!(["gc_bias_score"]));

    let bias_report = repo_root.join(
        payload["bias_report"].as_str().unwrap_or_else(|| panic!("bias_report path missing")),
    );
    let bias_summary = repo_root.join(
        payload["bias_summary"].as_str().unwrap_or_else(|| panic!("bias_summary path missing")),
    );
    let bias_policy = repo_root.join(
        payload["bias_policy"].as_str().unwrap_or_else(|| panic!("bias_policy path missing")),
    );
    let stage_metrics = repo_root.join(
        payload["stage_metrics"].as_str().unwrap_or_else(|| panic!("stage_metrics path missing")),
    );
    for path in [&bias_report, &bias_summary, &bias_policy, &stage_metrics] {
        assert!(
            path.is_file(),
            "governed BAM bias-mitigation artifact must exist: {}",
            path.display()
        );
    }

    let report_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&bias_report)?)?;
    assert_eq!(report_json["schema_version"], serde_json::json!("bijux.bam.bias_mitigation.v1"));
    assert_eq!(report_json["method"], serde_json::json!("mapdamage2"));
    assert_eq!(report_json["metric_name"], serde_json::json!("gc_bias_score"));
    assert_eq!(report_json["pre_mitigation_metric"], serde_json::json!(0.25));
    assert_eq!(report_json["post_mitigation_metric"], serde_json::json!(0.125));

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&bias_summary)?)?;
    assert_eq!(
        summary_json["schema_version"],
        serde_json::json!("bijux.bam.bias_mitigation_summary.v1")
    );
    assert_eq!(summary_json["stage_id"], serde_json::json!("bam.bias_mitigation"));
    assert_eq!(summary_json["method"], serde_json::json!("mapdamage2"));
    assert_eq!(summary_json["metric_name"], serde_json::json!("gc_bias_score"));
    assert_eq!(summary_json["pre_mitigation_metric"], serde_json::json!(0.25));
    assert_eq!(summary_json["post_mitigation_metric"], serde_json::json!(0.125));
    assert_eq!(summary_json["metric_delta"], serde_json::json!(0.125));
    assert_eq!(
        summary_json["reference_fasta"],
        serde_json::json!(repo_root
            .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta")
            .display()
            .to_string())
    );

    let policy_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&bias_policy)?)?;
    assert_eq!(policy_json["gc_bias_correction"], serde_json::json!(true));
    assert_eq!(policy_json["map_bias_correction"], serde_json::json!(false));

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.bias_mitigation.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["expected_method"], serde_json::json!("mapdamage2"));
    assert_eq!(stage_metrics_json["method"], serde_json::json!("mapdamage2"));
    assert_eq!(stage_metrics_json["expected_metric_name"], serde_json::json!("gc_bias_score"));
    assert_eq!(stage_metrics_json["metric_name"], serde_json::json!("gc_bias_score"));
    assert_eq!(stage_metrics_json["expected_pre_mitigation_metric"], serde_json::json!(0.25));
    assert_eq!(stage_metrics_json["pre_mitigation_metric"], serde_json::json!(0.25));
    assert_eq!(stage_metrics_json["pre_mitigation_metric_delta"], serde_json::json!(0.0));
    assert_eq!(stage_metrics_json["expected_post_mitigation_metric"], serde_json::json!(0.125));
    assert_eq!(stage_metrics_json["post_mitigation_metric"], serde_json::json!(0.125));
    assert_eq!(stage_metrics_json["post_mitigation_metric_delta"], serde_json::json!(0.0));
    assert_eq!(stage_metrics_json["metric_delta"], serde_json::json!(0.125));
    assert_eq!(stage_metrics_json["mitigation_actions"], serde_json::json!(["gc_bias_correction"]));
    assert_eq!(stage_metrics_json["consumed_metrics"], serde_json::json!(["gc_bias_score"]));
    assert_eq!(
        stage_metrics_json["mitigation_projection_basis"],
        serde_json::json!("policy_projection")
    );
    assert_eq!(stage_metrics_json["expectation_matched"], serde_json::json!(true));

    Ok(())
}
