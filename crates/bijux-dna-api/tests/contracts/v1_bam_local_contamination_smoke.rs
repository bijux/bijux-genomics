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
fn write_local_contamination_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("runs/bench/local-smoke/bam.contamination");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_contamination_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("runs/bench/local-smoke/bam.contamination/local_smoke.json")
    );
    assert!(report_path.is_file(), "local-smoke BAM contamination report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.contamination"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.contamination.local_smoke.report.v1")
    );
    assert_eq!(payload["tool_ids"], serde_json::json!(["contammix", "schmutzi", "verifybamid2"]));
    assert_eq!(payload["case_count"], serde_json::json!(6));

    let rows = payload["rows"].as_array().unwrap_or_else(|| panic!("rows array missing"));
    assert_eq!(rows.len(), 6);
    for tool_id in ["contammix", "schmutzi", "verifybamid2"] {
        let ready = rows
            .iter()
            .find(|row| {
                row["tool_id"] == serde_json::json!(tool_id)
                    && row["proof_case"] == serde_json::json!("ready")
            })
            .unwrap_or_else(|| panic!("ready case missing for {tool_id}"));
        let insufficient = rows
            .iter()
            .find(|row| {
                row["tool_id"] == serde_json::json!(tool_id)
                    && row["proof_case"] == serde_json::json!("insufficient")
            })
            .unwrap_or_else(|| panic!("insufficient case missing for {tool_id}"));

        assert_eq!(ready["expectation_matched"], serde_json::json!(true));
        assert_eq!(ready["prerequisites_passed"], serde_json::json!(true));
        assert_eq!(insufficient["expectation_matched"], serde_json::json!(true));
        assert_eq!(insufficient["prerequisites_passed"], serde_json::json!(false));
        assert!(
            insufficient["refusal_codes"].as_array().is_some_and(|codes| !codes.is_empty()),
            "insufficient contamination case must keep refusal codes for {tool_id}"
        );

        let ready_report =
            repo_root.join(ready["contamination_report"].as_str().unwrap_or_else(|| {
                panic!("ready contamination_report path missing for {tool_id}")
            }));
        let ready_summary =
            repo_root.join(ready["contamination_summary"].as_str().unwrap_or_else(|| {
                panic!("ready contamination_summary path missing for {tool_id}")
            }));
        let ready_stage_metrics = repo_root.join(
            ready["stage_metrics"]
                .as_str()
                .unwrap_or_else(|| panic!("ready stage_metrics path missing for {tool_id}")),
        );
        let insufficient_summary =
            repo_root.join(insufficient["contamination_summary"].as_str().unwrap_or_else(|| {
                panic!("insufficient contamination_summary path missing for {tool_id}")
            }));
        let insufficient_stage_metrics =
            repo_root.join(insufficient["stage_metrics"].as_str().unwrap_or_else(|| {
                panic!("insufficient stage_metrics path missing for {tool_id}")
            }));

        for path in [
            &ready_report,
            &ready_summary,
            &ready_stage_metrics,
            &insufficient_summary,
            &insufficient_stage_metrics,
        ] {
            assert!(
                path.is_file(),
                "governed BAM contamination artifact must exist: {}",
                path.display()
            );
        }

        let ready_report_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&ready_report)?)?;
        assert_eq!(ready_report_json["method"], serde_json::json!(tool_id));
        assert_eq!(ready_report_json["estimate"], serde_json::json!(0.02));
        assert_eq!(ready_report_json["ci_low"], serde_json::json!(0.01));
        assert_eq!(ready_report_json["ci_high"], serde_json::json!(0.03));

        let ready_summary_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&ready_summary)?)?;
        assert_eq!(
            ready_summary_json["schema_version"],
            serde_json::json!("bijux.bam.contamination_evidence.v1")
        );
        assert_eq!(ready_summary_json["stage_id"], serde_json::json!("bam.contamination"));
        assert_eq!(ready_summary_json["tool"], serde_json::json!(tool_id));
        assert_eq!(ready_summary_json["prerequisites_passed"], serde_json::json!(true));
        assert_eq!(ready_summary_json["estimate"], serde_json::json!(0.02));

        let insufficient_summary_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&insufficient_summary)?)?;
        assert_eq!(
            insufficient_summary_json["schema_version"],
            serde_json::json!("bijux.bam.contamination_evidence.v1")
        );
        assert_eq!(insufficient_summary_json["prerequisites_passed"], serde_json::json!(false));
        assert_eq!(insufficient_summary_json["estimate"], serde_json::Value::Null);
        assert!(
            insufficient_summary_json["refusal_codes"]
                .as_array()
                .is_some_and(|codes| !codes.is_empty()),
            "insufficient summary must keep refusal codes for {tool_id}"
        );

        let ready_stage_metrics_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&ready_stage_metrics)?)?;
        assert_eq!(
            ready_stage_metrics_json["schema_version"],
            serde_json::json!("bijux.bam.contamination.stage_metrics.v1")
        );
        assert_eq!(ready_stage_metrics_json["proof_case"], serde_json::json!("ready"));
        assert_eq!(
            ready_stage_metrics_json["expected_prerequisites_passed"],
            serde_json::json!(true)
        );
        assert_eq!(ready_stage_metrics_json["prerequisites_passed"], serde_json::json!(true));
        assert_eq!(ready_stage_metrics_json["reported_estimate"], serde_json::json!(0.02));
        assert_eq!(ready_stage_metrics_json["expectation_matched"], serde_json::json!(true));

        let insufficient_stage_metrics_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&insufficient_stage_metrics)?)?;
        assert_eq!(
            insufficient_stage_metrics_json["proof_case"],
            serde_json::json!("insufficient")
        );
        assert_eq!(
            insufficient_stage_metrics_json["expected_prerequisites_passed"],
            serde_json::json!(false)
        );
        assert_eq!(
            insufficient_stage_metrics_json["prerequisites_passed"],
            serde_json::json!(false)
        );
        assert_eq!(insufficient_stage_metrics_json["reported_estimate"], serde_json::Value::Null);
        assert_eq!(insufficient_stage_metrics_json["expectation_matched"], serde_json::json!(true));
    }

    Ok(())
}
