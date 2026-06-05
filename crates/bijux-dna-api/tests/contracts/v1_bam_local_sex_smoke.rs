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
fn write_local_sex_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.sex");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_sex_smoke_report()?;
    assert_eq!(report_path, repo_root.join("target/local-smoke/bam.sex/sex.json"));
    assert!(report_path.is_file(), "local-smoke BAM sex report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.sex"));
    assert_eq!(payload["schema_version"], serde_json::json!("bijux.bam.sex.local_smoke.report.v1"));
    assert_eq!(payload["sample_id"], serde_json::json!("human_like_xy_autosome_coverage"));
    assert_eq!(payload["expectation_matched"], serde_json::json!(true));
    assert_eq!(payload["method"], serde_json::json!("rxy"));
    assert_eq!(payload["chromosome_system"], serde_json::json!("xy"));
    assert_eq!(payload["minimum_y_sites"], serde_json::json!(5));
    assert_eq!(payload["x_coverage"], serde_json::json!(0.5));
    assert_eq!(payload["y_coverage"], serde_json::json!(0.5));
    assert_eq!(payload["autosomal_coverage"], serde_json::json!(1.0));
    assert_eq!(payload["x_to_y_ratio"], serde_json::json!(1.0));
    assert_eq!(payload["call"], serde_json::json!("male"));
    assert_eq!(payload["confidence"], serde_json::json!(0.9));
    assert_eq!(payload["status"], serde_json::json!("ok"));
    assert_eq!(payload["insufficiency_reason"], serde_json::Value::Null);

    let sex_report = repo_root
        .join(payload["sex_report"].as_str().unwrap_or_else(|| panic!("sex_report path missing")));
    let sex_summary = repo_root.join(
        payload["sex_summary"].as_str().unwrap_or_else(|| panic!("sex_summary path missing")),
    );
    let stage_metrics = repo_root.join(
        payload["stage_metrics"].as_str().unwrap_or_else(|| panic!("stage_metrics path missing")),
    );
    for path in [&sex_report, &sex_summary, &stage_metrics] {
        assert!(path.is_file(), "governed BAM sex artifact must exist: {}", path.display());
    }

    let report_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&sex_report)?)?;
    assert_eq!(report_json["schema_version"], serde_json::json!("bijux.bam.sex.v1"));
    assert_eq!(report_json["method"], serde_json::json!("rxy"));
    assert_eq!(report_json["x_coverage"], serde_json::json!(0.5));
    assert_eq!(report_json["y_coverage"], serde_json::json!(0.5));
    assert_eq!(report_json["autosomal_coverage"], serde_json::json!(1.0));
    assert_eq!(report_json["call"], serde_json::json!("male"));
    assert_eq!(report_json["confidence"], serde_json::json!(0.9));
    assert_eq!(report_json["status"], serde_json::json!("ok"));

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&sex_summary)?)?;
    assert_eq!(summary_json["schema_version"], serde_json::json!("bijux.bam.sex_summary.v1"));
    assert_eq!(summary_json["stage_id"], serde_json::json!("bam.sex"));
    assert_eq!(summary_json["method"], serde_json::json!("rxy"));
    assert_eq!(
        summary_json["reference_fasta"],
        serde_json::json!(repo_root
            .join(
                "tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
            )
            .display()
            .to_string())
    );
    assert_eq!(summary_json["x_coverage"], serde_json::json!(0.5));
    assert_eq!(summary_json["y_coverage"], serde_json::json!(0.5));
    assert_eq!(summary_json["autosomal_coverage"], serde_json::json!(1.0));
    assert_eq!(summary_json["call"], serde_json::json!("male"));
    assert_eq!(summary_json["confidence"], serde_json::json!(0.9));
    assert_eq!(summary_json["status"], serde_json::json!("ok"));

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.sex.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["expected_method"], serde_json::json!("rxy"));
    assert_eq!(stage_metrics_json["method"], serde_json::json!("rxy"));
    assert_eq!(stage_metrics_json["expected_chromosome_system"], serde_json::json!("xy"));
    assert_eq!(stage_metrics_json["chromosome_system"], serde_json::json!("xy"));
    assert_eq!(stage_metrics_json["expected_minimum_y_sites"], serde_json::json!(5));
    assert_eq!(stage_metrics_json["minimum_y_sites"], serde_json::json!(5));
    assert_eq!(stage_metrics_json["expected_x_coverage"], serde_json::json!(0.5));
    assert_eq!(stage_metrics_json["x_coverage"], serde_json::json!(0.5));
    assert_eq!(stage_metrics_json["x_coverage_delta"], serde_json::json!(0.0));
    assert_eq!(stage_metrics_json["expected_y_coverage"], serde_json::json!(0.5));
    assert_eq!(stage_metrics_json["y_coverage"], serde_json::json!(0.5));
    assert_eq!(stage_metrics_json["y_coverage_delta"], serde_json::json!(0.0));
    assert_eq!(stage_metrics_json["expected_autosomal_coverage"], serde_json::json!(1.0));
    assert_eq!(stage_metrics_json["autosomal_coverage"], serde_json::json!(1.0));
    assert_eq!(stage_metrics_json["autosomal_coverage_delta"], serde_json::json!(0.0));
    assert_eq!(stage_metrics_json["expected_call"], serde_json::json!("male"));
    assert_eq!(stage_metrics_json["call"], serde_json::json!("male"));
    assert_eq!(stage_metrics_json["expected_confidence"], serde_json::json!(0.9));
    assert_eq!(stage_metrics_json["confidence"], serde_json::json!(0.9));
    assert_eq!(stage_metrics_json["confidence_delta"], serde_json::json!(0.0));
    assert_eq!(stage_metrics_json["expected_status"], serde_json::json!("ok"));
    assert_eq!(stage_metrics_json["status"], serde_json::json!("ok"));
    assert_eq!(stage_metrics_json["insufficiency_reason"], serde_json::Value::Null);
    assert_eq!(stage_metrics_json["expectation_matched"], serde_json::json!(true));

    Ok(())
}
