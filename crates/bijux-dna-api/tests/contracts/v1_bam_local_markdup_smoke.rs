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
fn write_local_markdup_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("runs/bench/local-smoke/bam.markdup");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_markdup_smoke_report()?;
    assert_eq!(report_path, repo_root.join("runs/bench/local-smoke/bam.markdup/duplicates.json"));
    assert!(report_path.is_file(), "local-smoke BAM markdup report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.markdup"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.markdup.local_smoke.report.v1")
    );
    assert_eq!(payload["sample_id"], serde_json::json!("human_like_duplicate_cluster"));
    assert_eq!(payload["expectation_matched"], serde_json::json!(true));
    assert_eq!(payload["duplicate_action"], serde_json::json!("mark"));
    assert_eq!(payload["input_reads"], serde_json::json!(4));
    assert_eq!(payload["output_reads"], serde_json::json!(4));
    assert_eq!(payload["removed_reads"], serde_json::json!(0));
    assert_eq!(payload["duplicate_count"], serde_json::json!(1));
    assert_eq!(payload["duplicate_fraction"], serde_json::json!(0.25));
    assert_eq!(payload["duplicate_reads_before"], serde_json::json!(0));
    assert_eq!(payload["duplicate_reads_after"], serde_json::json!(1));
    assert_eq!(payload["newly_marked_reads"], serde_json::json!(1));

    let marked_bam = repo_root
        .join(payload["marked_bam"].as_str().unwrap_or_else(|| panic!("marked_bam path missing")));
    let marked_bai = repo_root
        .join(payload["marked_bai"].as_str().unwrap_or_else(|| panic!("marked_bai path missing")));
    let markdup_summary = repo_root.join(
        payload["markdup_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("markdup_summary path missing")),
    );
    let duplicate_metrics = repo_root.join(
        payload["duplicate_metrics"]
            .as_str()
            .unwrap_or_else(|| panic!("duplicate_metrics path missing")),
    );
    let markdup_policy = repo_root.join(
        payload["markdup_policy"].as_str().unwrap_or_else(|| panic!("markdup_policy path missing")),
    );
    let markdup_comparison = repo_root.join(
        payload["markdup_comparison"]
            .as_str()
            .unwrap_or_else(|| panic!("markdup_comparison path missing")),
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
        &marked_bam,
        &marked_bai,
        &markdup_summary,
        &duplicate_metrics,
        &markdup_policy,
        &markdup_comparison,
        &flagstat_before,
        &flagstat_after,
        &idxstats_before,
        &idxstats_after,
        &stage_metrics,
    ] {
        assert!(path.is_file(), "governed BAM markdup artifact must exist: {}", path.display());
    }

    let marked_body = std::fs::read_to_string(&marked_bam)?;
    assert!(marked_body.contains("dup_primary"));
    assert!(marked_body.contains("dup_copy\t1024\tchr1\t5"));
    assert!(marked_body.contains("unique_support"));
    assert!(marked_body.contains("unmapped_support"));

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&markdup_summary)?)?;
    assert_eq!(summary_json["schema_version"], serde_json::json!("bijux.bam.markdup.v1"));
    assert_eq!(summary_json["duplicate_action"], serde_json::json!("mark"));
    assert_eq!(summary_json["input_reads"], serde_json::json!(4));
    assert_eq!(summary_json["output_reads"], serde_json::json!(4));
    assert_eq!(summary_json["removed_reads"], serde_json::json!(0));
    assert_eq!(summary_json["duplicate_count"], serde_json::json!(1));
    assert_eq!(summary_json["duplicate_fraction"], serde_json::json!(0.25));
    assert_eq!(summary_json["duplicate_reads_after"], serde_json::json!(1));
    assert_eq!(summary_json["newly_marked_reads"], serde_json::json!(1));

    let policy_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&markdup_policy)?)?;
    assert_eq!(policy_json["schema_version"], serde_json::json!("bijux.bam.duplicate_policy.v1"));
    assert_eq!(policy_json["duplicate_action"], serde_json::json!("mark"));
    assert_eq!(policy_json["optical_duplicates"], serde_json::json!("mark_only"));
    assert_eq!(policy_json["umi_policy"], serde_json::json!("ignore"));

    let comparison_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&markdup_comparison)?)?;
    assert_eq!(
        comparison_json["schema_version"],
        serde_json::json!("bijux.bam.duplicate_comparison.v1")
    );
    assert_eq!(comparison_json["comparable"], serde_json::json!(true));

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.markdup.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["duplicate_action"], serde_json::json!("mark"));
    assert_eq!(stage_metrics_json["input_reads"], serde_json::json!(4));
    assert_eq!(stage_metrics_json["output_reads"], serde_json::json!(4));
    assert_eq!(stage_metrics_json["removed_reads"], serde_json::json!(0));
    assert_eq!(stage_metrics_json["duplicate_count"], serde_json::json!(1));
    assert_eq!(stage_metrics_json["duplicate_fraction"], serde_json::json!(0.25));
    assert_eq!(stage_metrics_json["duplicate_reads_before"], serde_json::json!(0));
    assert_eq!(stage_metrics_json["duplicate_reads_after"], serde_json::json!(1));
    assert_eq!(stage_metrics_json["newly_marked_reads"], serde_json::json!(1));
    assert_eq!(stage_metrics_json["expectation_matched"], serde_json::json!(true));

    Ok(())
}
