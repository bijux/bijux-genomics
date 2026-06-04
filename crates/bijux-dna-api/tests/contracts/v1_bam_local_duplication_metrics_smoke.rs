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
fn write_local_duplication_metrics_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.duplication_metrics");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_duplication_metrics_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("target/local-smoke/bam.duplication_metrics/duplication_metrics.json")
    );
    assert!(report_path.is_file(), "local-smoke BAM duplication metrics report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.duplication_metrics"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.duplication_metrics.local_smoke.report.v1")
    );
    assert_eq!(payload["sample_id"], serde_json::json!("human_like_duplicate_cluster"));
    assert_eq!(payload["expectation_matched"], serde_json::json!(true));
    assert_eq!(payload["method"], serde_json::json!("samtools"));
    assert_eq!(payload["examined_reads"], serde_json::json!(3));
    assert_eq!(payload["duplicate_reads"], serde_json::json!(1));
    assert_eq!(payload["duplicate_count"], serde_json::json!(1));
    assert_eq!(payload["duplicate_fraction"], serde_json::json!(1.0 / 3.0));
    assert_eq!(payload["estimated_library_size"], serde_json::Value::Null);
    assert_eq!(
        payload["insufficient_library_size_reason"],
        serde_json::json!(
            "tiny_smoke_duplicate_observation_is_insufficient_for_library_size_estimate"
        )
    );

    let duplication_report = repo_root.join(
        payload["duplication_report"]
            .as_str()
            .unwrap_or_else(|| panic!("duplication_report path missing")),
    );
    let duplication_histogram = repo_root.join(
        payload["duplication_histogram"]
            .as_str()
            .unwrap_or_else(|| panic!("duplication_histogram path missing")),
    );
    let duplication_summary = repo_root.join(
        payload["duplication_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("duplication_summary path missing")),
    );
    let duplication_policy = repo_root.join(
        payload["duplication_policy"]
            .as_str()
            .unwrap_or_else(|| panic!("duplication_policy path missing")),
    );
    let stage_metrics = repo_root.join(
        payload["stage_metrics"].as_str().unwrap_or_else(|| panic!("stage_metrics path missing")),
    );
    for path in [
        &duplication_report,
        &duplication_histogram,
        &duplication_summary,
        &duplication_policy,
        &stage_metrics,
    ] {
        assert!(
            path.is_file(),
            "governed BAM duplication metrics artifact must exist: {}",
            path.display()
        );
    }

    let observation_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&duplication_report)?)?;
    assert_eq!(
        observation_json["schema_version"],
        serde_json::json!("bijux.bam.duplication_metrics.local_smoke.observation.v1")
    );
    assert_eq!(observation_json["method"], serde_json::json!("samtools"));
    assert_eq!(observation_json["examined_reads"], serde_json::json!(3));
    assert_eq!(observation_json["duplicate_reads"], serde_json::json!(1));
    assert_eq!(observation_json["duplicate_count"], serde_json::json!(1));
    assert_eq!(
        observation_json["insufficient_library_size_reason"],
        serde_json::json!(
            "tiny_smoke_duplicate_observation_is_insufficient_for_library_size_estimate"
        )
    );

    let histogram_body = std::fs::read_to_string(&duplication_histogram)?;
    assert_eq!(histogram_body, "family_size\tfamily_count\n1\t1\n2\t1\n");

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&duplication_summary)?)?;
    assert_eq!(
        summary_json["schema_version"],
        serde_json::json!("bijux.bam.duplication_metrics.v1")
    );
    assert_eq!(summary_json["method"], serde_json::json!("samtools"));
    assert_eq!(summary_json["examined_reads"], serde_json::json!(3));
    assert_eq!(summary_json["duplicate_reads"], serde_json::json!(1));
    assert_eq!(summary_json["duplicate_count"], serde_json::json!(1));
    assert_eq!(summary_json["duplicate_fraction"], serde_json::json!(1.0 / 3.0));
    assert_eq!(
        summary_json["insufficient_library_size_reason"],
        serde_json::json!(
            "tiny_smoke_duplicate_observation_is_insufficient_for_library_size_estimate"
        )
    );

    let policy_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&duplication_policy)?)?;
    assert_eq!(policy_json["schema_version"], serde_json::json!("bijux.bam.duplicate_policy.v1"));
    assert_eq!(policy_json["duplicate_action"], serde_json::json!("mark"));
    assert_eq!(policy_json["optical_duplicates"], serde_json::json!("mark_only"));
    assert_eq!(policy_json["umi_policy"], serde_json::json!("ignore"));
    assert_eq!(policy_json["policy_scope"], serde_json::json!("observation_only"));

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.duplication_metrics.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["method"], serde_json::json!("samtools"));
    assert_eq!(stage_metrics_json["examined_reads"], serde_json::json!(3));
    assert_eq!(stage_metrics_json["duplicate_reads"], serde_json::json!(1));
    assert_eq!(stage_metrics_json["duplicate_count"], serde_json::json!(1));
    assert_eq!(stage_metrics_json["duplicate_fraction"], serde_json::json!(1.0 / 3.0));
    assert_eq!(stage_metrics_json["estimated_library_size"], serde_json::Value::Null);
    assert_eq!(
        stage_metrics_json["insufficient_library_size_reason"],
        serde_json::json!(
            "tiny_smoke_duplicate_observation_is_insufficient_for_library_size_estimate"
        )
    );
    assert_eq!(stage_metrics_json["expectation_matched"], serde_json::json!(true));

    Ok(())
}
