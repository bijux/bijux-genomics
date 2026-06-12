#![allow(clippy::too_many_lines)]

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
fn write_local_recalibration_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("runs/bench/local-smoke/bam.recalibration");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_recalibration_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("runs/bench/local-smoke/bam.recalibration/recalibration.json")
    );
    assert!(report_path.is_file(), "local-smoke BAM recalibration report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.recalibration"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.recalibration.local_smoke.report.v1")
    );
    assert_eq!(payload["sample_id"], serde_json::json!("human_like_recalibration_low_coverage"));
    assert_eq!(payload["expectation_matched"], serde_json::json!(true));
    assert_eq!(payload["requested_mode"], serde_json::json!("standard"));
    assert_eq!(payload["effective_mode"], serde_json::json!("skip"));
    assert_eq!(payload["status"], serde_json::json!("skipped"));
    assert_eq!(payload["reason"], serde_json::json!("coverage_below_gate"));
    assert_eq!(
        payload["known_sites"],
        serde_json::json!(["benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf"])
    );
    assert_eq!(
        payload["coverage_gate"],
        serde_json::json!({
            "min_mean_coverage": 0.2,
            "min_breadth_1x": 0.2
        })
    );
    assert_eq!(payload["observed_mean_coverage"], serde_json::json!(0.192));
    assert_eq!(payload["observed_breadth_1x"], serde_json::json!(0.192));
    assert_eq!(payload["output_bam_present"], serde_json::json!(true));
    assert_eq!(payload["recalibration_report_present"], serde_json::json!(true));

    let recalibrated_bam = repo_root.join(
        payload["recalibrated_bam"]
            .as_str()
            .unwrap_or_else(|| panic!("recalibrated_bam path missing")),
    );
    let recalibration_report = repo_root.join(
        payload["recalibration_report"]
            .as_str()
            .unwrap_or_else(|| panic!("recalibration_report path missing")),
    );
    let recalibration_summary = repo_root.join(
        payload["recalibration_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("recalibration_summary path missing")),
    );
    let stage_metrics = repo_root.join(
        payload["stage_metrics"].as_str().unwrap_or_else(|| panic!("stage_metrics path missing")),
    );
    for path in [&recalibrated_bam, &recalibration_report, &recalibration_summary, &stage_metrics] {
        assert!(
            path.is_file(),
            "governed BAM recalibration artifact must exist: {}",
            path.display()
        );
    }

    let raw_report = std::fs::read_to_string(&recalibration_report)?;
    assert!(raw_report.contains("status=skipped"));
    assert!(raw_report.contains("reason=coverage_below_gate"));

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&recalibration_summary)?)?;
    assert_eq!(summary_json["schema_version"], serde_json::json!("bijux.bam.recalibration.v1"));
    assert_eq!(summary_json["stage_id"], serde_json::json!("bam.recalibration"));
    assert_eq!(summary_json["requested_mode"], serde_json::json!("standard"));
    assert_eq!(summary_json["effective_mode"], serde_json::json!("skip"));
    assert_eq!(summary_json["status"], serde_json::json!("skipped"));
    assert_eq!(summary_json["reason"], serde_json::json!("coverage_below_gate"));
    assert_eq!(
        summary_json["known_sites"],
        serde_json::json!([repo_root
            .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf")
            .display()
            .to_string()])
    );
    assert_eq!(
        summary_json["reference_fasta"],
        serde_json::json!(repo_root
            .join(
                "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
            )
            .display()
            .to_string())
    );
    assert_eq!(summary_json["observed_mean_coverage"], serde_json::json!(0.192));
    assert_eq!(summary_json["observed_breadth_1x"], serde_json::json!(0.192));
    assert_eq!(summary_json["output_bam_present"], serde_json::json!(true));
    assert_eq!(summary_json["recalibration_report_present"], serde_json::json!(true));

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.recalibration.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["expected_requested_mode"], serde_json::json!("standard"));
    assert_eq!(stage_metrics_json["requested_mode"], serde_json::json!("standard"));
    assert_eq!(stage_metrics_json["expected_effective_mode"], serde_json::json!("skip"));
    assert_eq!(stage_metrics_json["effective_mode"], serde_json::json!("skip"));
    assert_eq!(stage_metrics_json["expected_status"], serde_json::json!("skipped"));
    assert_eq!(stage_metrics_json["status"], serde_json::json!("skipped"));
    assert_eq!(stage_metrics_json["expected_reason"], serde_json::json!("coverage_below_gate"));
    assert_eq!(stage_metrics_json["reason"], serde_json::json!("coverage_below_gate"));
    assert_eq!(
        stage_metrics_json["expected_known_sites"],
        serde_json::json!(["benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf"])
    );
    assert_eq!(
        stage_metrics_json["known_sites"],
        serde_json::json!(["benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf"])
    );
    assert_eq!(
        stage_metrics_json["expected_coverage_gate"],
        serde_json::json!({
            "min_mean_coverage": 0.2,
            "min_breadth_1x": 0.2
        })
    );
    assert_eq!(
        stage_metrics_json["coverage_gate"],
        serde_json::json!({
            "min_mean_coverage": 0.2,
            "min_breadth_1x": 0.2
        })
    );
    assert_eq!(stage_metrics_json["observed_mean_coverage"], serde_json::json!(0.192));
    assert!(
        (stage_metrics_json["mean_coverage_margin"]
            .as_f64()
            .unwrap_or_else(|| panic!("mean_coverage_margin missing"))
            + 0.008)
            .abs()
            <= 1e-12
    );
    assert_eq!(stage_metrics_json["observed_breadth_1x"], serde_json::json!(0.192));
    assert!(
        (stage_metrics_json["breadth_1x_margin"]
            .as_f64()
            .unwrap_or_else(|| panic!("breadth_1x_margin missing"))
            + 0.008)
            .abs()
            <= 1e-12
    );
    assert_eq!(stage_metrics_json["output_bam_present"], serde_json::json!(true));
    assert_eq!(stage_metrics_json["recalibration_report_present"], serde_json::json!(true));
    assert_eq!(stage_metrics_json["expectation_matched"], serde_json::json!(true));

    Ok(())
}
