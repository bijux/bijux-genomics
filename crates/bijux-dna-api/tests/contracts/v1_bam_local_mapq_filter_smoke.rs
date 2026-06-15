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

#[test]
fn write_local_mapq_filter_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("runs/bench/local-smoke/bam.mapq_filter");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_mapq_filter_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("runs/bench/local-smoke/bam.mapq_filter/mapq_filter.json")
    );
    assert!(report_path.is_file(), "local-smoke BAM MAPQ filter report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.mapq_filter"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.mapq_filter.local_smoke.report.v1")
    );
    assert_eq!(payload["sample_id"], serde_json::json!("human_like_mapq_threshold_ladder"));
    assert_eq!(payload["expectation_matched"], serde_json::json!(true));
    assert_eq!(payload["mapq_threshold"], serde_json::json!(30));
    assert_eq!(payload["input_reads"], serde_json::json!(4));
    assert_eq!(payload["kept_reads"], serde_json::json!(3));
    assert_eq!(payload["removed_reads"], serde_json::json!(1));
    assert_eq!(payload["mapped_reads_removed"], serde_json::json!(1));
    assert_eq!(payload["mapped_fraction_retained"], serde_json::json!(0.6666666666666666));

    let filtered_bam = repo_root.join(
        payload["filtered_bam"].as_str().unwrap_or_else(|| panic!("filtered_bam path missing")),
    );
    let filtered_index = repo_root.join(
        payload["filtered_bai"].as_str().unwrap_or_else(|| panic!("filtered_bai path missing")),
    );
    let mapq_filter_summary = repo_root.join(
        payload["mapq_filter_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("mapq_filter_summary path missing")),
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
        &filtered_bam,
        &filtered_index,
        &mapq_filter_summary,
        &flagstat_before,
        &flagstat_after,
        &idxstats_before,
        &idxstats_after,
        &stage_metrics,
    ] {
        assert!(path.is_file(), "governed BAM MAPQ filter artifact must exist: {}", path.display());
    }

    let filtered_body = std::fs::read_to_string(&filtered_bam)?;
    assert!(filtered_body.contains("mapq60"));
    assert!(filtered_body.contains("mapq30"));
    assert!(filtered_body.contains("unmapped"));
    assert!(!filtered_body.contains("mapq10"));

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&mapq_filter_summary)?)?;
    assert_eq!(summary_json["schema_version"], serde_json::json!("bijux.bam.mapq_filter.v1"));
    assert_eq!(summary_json["mapq_threshold"], serde_json::json!(30));
    assert_eq!(summary_json["input_reads"], serde_json::json!(4));
    assert_eq!(summary_json["kept_reads"], serde_json::json!(3));
    assert_eq!(summary_json["removed_reads"], serde_json::json!(1));
    assert_eq!(summary_json["mapped_reads_removed"], serde_json::json!(1));

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.mapq_filter.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["mapq_threshold"], serde_json::json!(30));
    assert_eq!(stage_metrics_json["input_reads"], serde_json::json!(4));
    assert_eq!(stage_metrics_json["kept_reads"], serde_json::json!(3));
    assert_eq!(stage_metrics_json["removed_reads"], serde_json::json!(1));
    assert_eq!(stage_metrics_json["mapped_reads_removed"], serde_json::json!(1));
    assert_eq!(
        stage_metrics_json["mapped_fraction_retained"],
        serde_json::json!(0.6666666666666666)
    );
    assert_eq!(stage_metrics_json["expectation_matched"], serde_json::json!(true));

    Ok(())
}
