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
fn write_local_overlap_correction_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.overlap_correction");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_overlap_correction_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("target/local-smoke/bam.overlap_correction/overlap_correction.json")
    );
    assert!(report_path.is_file(), "local-smoke BAM overlap-correction report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.overlap_correction"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.overlap_correction.local_smoke.report.v1")
    );
    assert_eq!(payload["sample_id"], serde_json::json!("core-v1-paired-overlap"));
    assert_eq!(payload["expectation_matched"], serde_json::json!(true));
    assert_eq!(payload["method"], serde_json::json!("bamutil"));
    assert_eq!(payload["pair_count"], serde_json::json!(2));
    assert_eq!(payload["corrected_pairs"], serde_json::json!(1));
    assert_eq!(payload["corrected_overlap_bases"], serde_json::json!(7));
    assert_eq!(payload["insufficiency_reason"], serde_json::Value::Null);

    let corrected_bam = repo_root.join(
        payload["overlap_corrected_bam"]
            .as_str()
            .unwrap_or_else(|| panic!("overlap_corrected_bam path missing")),
    );
    let overlap_summary = repo_root.join(
        payload["overlap_correction_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("overlap_correction_summary path missing")),
    );
    let flagstat_before = repo_root.join(
        payload["flagstat_before"]
            .as_str()
            .unwrap_or_else(|| panic!("flagstat_before path missing")),
    );
    let flagstat_after = repo_root.join(
        payload["flagstat_after"].as_str().unwrap_or_else(|| panic!("flagstat_after path missing")),
    );
    let idxstats_before = repo_root.join(
        payload["idxstats_before"]
            .as_str()
            .unwrap_or_else(|| panic!("idxstats_before path missing")),
    );
    let idxstats_after = repo_root.join(
        payload["idxstats_after"].as_str().unwrap_or_else(|| panic!("idxstats_after path missing")),
    );
    let stage_metrics = repo_root.join(
        payload["stage_metrics"].as_str().unwrap_or_else(|| panic!("stage_metrics path missing")),
    );
    for path in [
        &corrected_bam,
        &overlap_summary,
        &flagstat_before,
        &flagstat_after,
        &idxstats_before,
        &idxstats_after,
        &stage_metrics,
    ] {
        assert!(
            path.is_file(),
            "governed BAM overlap-correction artifact must exist: {}",
            path.display()
        );
    }

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&overlap_summary)?)?;
    assert_eq!(
        summary_json["schema_version"],
        serde_json::json!("bijux.bam.overlap_correction.v1")
    );
    assert_eq!(summary_json["stage_id"], serde_json::json!("bam.overlap_correction"));
    assert_eq!(summary_json["method"], serde_json::json!("bamutil"));
    assert_eq!(summary_json["pair_count"], serde_json::json!(2));
    assert_eq!(summary_json["corrected_pairs"], serde_json::json!(1));
    assert_eq!(summary_json["corrected_overlap_bases"], serde_json::json!(7));
    assert_eq!(summary_json["insufficiency_reason"], serde_json::Value::Null);

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.overlap_correction.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["method"], serde_json::json!("bamutil"));
    assert_eq!(stage_metrics_json["expected_pair_count"], serde_json::json!(2));
    assert_eq!(stage_metrics_json["pair_count"], serde_json::json!(2));
    assert_eq!(stage_metrics_json["pair_count_delta"], serde_json::json!(0));
    assert_eq!(stage_metrics_json["expected_corrected_pairs"], serde_json::json!(1));
    assert_eq!(stage_metrics_json["corrected_pairs"], serde_json::json!(1));
    assert_eq!(stage_metrics_json["corrected_pair_delta"], serde_json::json!(0));
    assert_eq!(stage_metrics_json["expected_corrected_overlap_bases"], serde_json::json!(7));
    assert_eq!(stage_metrics_json["corrected_overlap_bases"], serde_json::json!(7));
    assert_eq!(stage_metrics_json["corrected_overlap_base_delta"], serde_json::json!(0));
    assert_eq!(stage_metrics_json["insufficiency_reason"], serde_json::Value::Null);
    assert_eq!(stage_metrics_json["expectation_matched"], serde_json::json!(true));

    Ok(())
}
