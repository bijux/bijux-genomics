#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn write_local_haplogroups_smoke_report_materializes_ready_and_coverage_gate_cases() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("runs/bench/local-smoke/bam.haplogroups");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_haplogroups_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("runs/bench/local-smoke/bam.haplogroups/haplogroups.json")
    );
    assert!(report_path.is_file(), "haplogroups smoke report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.haplogroups.local_smoke.report.v1")
    );
    assert_eq!(payload["stage_id"], serde_json::json!("bam.haplogroups"));
    assert_eq!(payload["tool_id"], serde_json::json!("yleaf"));
    assert_eq!(payload["sample_id"], serde_json::json!("adna_y_haplogroup_panel"));
    assert_eq!(payload["reference_panel_id"], serde_json::json!("adna-y-hg38-mini"));
    assert_eq!(payload["reference_build"], serde_json::json!("hg38"));
    assert_eq!(payload["case_count"], serde_json::json!(2));

    let rows =
        payload["rows"].as_array().unwrap_or_else(|| panic!("rows must serialize as an array"));
    assert_eq!(rows.len(), 2);

    let ready = rows
        .iter()
        .find(|row| row["proof_case"] == "ready")
        .unwrap_or_else(|| panic!("ready haplogroups case missing"));
    assert_eq!(ready["tool_id"], serde_json::json!("yleaf"));
    assert_eq!(ready["reference_panel_id"], serde_json::json!("adna-y-hg38-mini"));
    assert_eq!(ready["reference_build"], serde_json::json!("hg38"));
    assert_eq!(ready["population_scope"], serde_json::json!("adna_y_haplogroup_panel"));
    assert_eq!(ready["minimum_coverage"], serde_json::json!(2.0));
    assert_eq!(ready["observed_mean_coverage"], serde_json::json!(2.0));
    assert_eq!(ready["ready"], serde_json::json!(true));
    assert_eq!(ready["haplogroup_call"], serde_json::json!("R1b1a"));
    assert_eq!(ready["confidence"], serde_json::json!(1.0));
    assert_eq!(ready["status"], serde_json::json!("ready"));
    assert_eq!(ready["markers_total"], serde_json::json!(2));
    assert_eq!(ready["markers_supported"], serde_json::json!(2));
    assert_eq!(ready["supported_marker_ids"], serde_json::json!(["adna-y-snp1", "adna-y-snp2"]));
    assert_eq!(ready["refusal_codes"], serde_json::json!([]));
    assert_eq!(ready["expectation_matched"], serde_json::json!(true));

    let ready_summary = repo_root.join(
        ready["haplogroups_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("ready summary path missing")),
    );
    let ready_summary_payload: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&ready_summary)?)?;
    assert_eq!(
        ready_summary_payload["schema_version"],
        serde_json::json!("bijux.bam.haplogroup_readiness.v1")
    );
    assert_eq!(ready_summary_payload["ready"], serde_json::json!(true));
    assert_eq!(ready_summary_payload["minimum_coverage"], serde_json::json!(2.0));
    assert_eq!(ready_summary_payload["observed_mean_coverage"], serde_json::json!(2.0));

    let insufficient = rows
        .iter()
        .find(|row| row["proof_case"] == "insufficient")
        .unwrap_or_else(|| panic!("insufficient haplogroups case missing"));
    assert_eq!(insufficient["minimum_coverage"], serde_json::json!(2.5));
    assert_eq!(insufficient["observed_mean_coverage"], serde_json::json!(2.0));
    assert_eq!(insufficient["ready"], serde_json::json!(false));
    assert_eq!(insufficient["haplogroup_call"], serde_json::Value::Null);
    assert_eq!(insufficient["confidence"], serde_json::json!(0.0));
    assert_eq!(insufficient["status"], serde_json::json!("coverage_gate_not_met"));
    assert_eq!(
        insufficient["refusal_codes"],
        serde_json::json!(["coverage_below_haplogroup_minimum"])
    );
    assert_eq!(insufficient["expectation_matched"], serde_json::json!(true));

    let insufficient_report = repo_root.join(
        insufficient["haplogroups_report"]
            .as_str()
            .unwrap_or_else(|| panic!("insufficient report path missing")),
    );
    let insufficient_report_payload: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&insufficient_report)?)?;
    assert_eq!(
        insufficient_report_payload["schema_version"],
        serde_json::json!("bijux.bam.haplogroups.v1")
    );
    assert_eq!(insufficient_report_payload["status"], serde_json::json!("coverage_gate_not_met"));
    assert_eq!(insufficient_report_payload["markers_supported"], serde_json::json!(2));

    Ok(())
}
