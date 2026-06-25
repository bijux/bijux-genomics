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
fn write_local_sex_tool_smoke_report_materializes_retained_tool_cases() -> Result<()> {
    let repo_root = repo_root()?;
    let _lock = crate::support::bench_output_lock()
        .lock()
        .unwrap_or_else(|err| panic!("lock BAM sex tool benchmark output: {err}"));
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_root = repo_root.join("runs/bench/local-smoke/bam.sex");
    if output_root.exists() {
        std::fs::remove_dir_all(&output_root)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_sex_tool_smoke_report()?;
    assert_eq!(report_path, repo_root.join("runs/bench/local-smoke/bam.sex/tool_smoke.json"));

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["schema_version"], serde_json::json!("bijux.bam.sex.tool_smoke.report.v1"));
    assert_eq!(payload["stage_id"], serde_json::json!("bam.sex"));
    assert_eq!(payload["ready_sample_id"], serde_json::json!("adna_xy_autosome_coverage"));
    assert_eq!(payload["insufficient_sample_id"], serde_json::json!("adna_y_haplogroup_panel"));
    assert_eq!(payload["case_count"], serde_json::json!(6));

    let tool_ids = required_array(&payload, "tool_ids")?;
    assert_eq!(tool_ids.len(), 3);
    assert!(tool_ids.iter().any(|tool| tool.as_str() == Some("angsd")));
    assert!(tool_ids.iter().any(|tool| tool.as_str() == Some("rxy")));
    assert!(tool_ids.iter().any(|tool| tool.as_str() == Some("yleaf")));

    let rows = required_array(&payload, "rows")?;
    assert_eq!(rows.len(), 6);

    let ready_angsd = rows
        .iter()
        .find(|row| {
            row["tool_id"] == "angsd"
                && row["proof_case"] == "ready"
                && row["sample_id"] == "adna_xy_autosome_coverage"
        })
        .ok_or_else(|| anyhow::anyhow!("ready angsd row missing"))?;
    assert_eq!(ready_angsd["method"], serde_json::json!("angsd"));
    assert_eq!(ready_angsd["call"], serde_json::json!("male"));
    assert_eq!(ready_angsd["status"], serde_json::json!("ok"));
    assert_eq!(ready_angsd["confidence"], serde_json::json!(0.9));
    assert_eq!(ready_angsd["x_coverage"], serde_json::json!(0.5));
    assert_eq!(ready_angsd["y_coverage"], serde_json::json!(0.5));
    assert_eq!(ready_angsd["autosomal_coverage"], serde_json::json!(1.0));
    assert_eq!(ready_angsd["insufficiency_reason"], serde_json::Value::Null);
    assert_eq!(ready_angsd["expectation_matched"], serde_json::json!(true));

    let insufficient_yleaf = rows
        .iter()
        .find(|row| {
            row["tool_id"] == "yleaf"
                && row["proof_case"] == "insufficient"
                && row["sample_id"] == "adna_y_haplogroup_panel"
        })
        .ok_or_else(|| anyhow::anyhow!("insufficient yleaf row missing"))?;
    assert_eq!(insufficient_yleaf["method"], serde_json::json!("yleaf"));
    assert_eq!(insufficient_yleaf["call"], serde_json::json!("insufficient"));
    assert_eq!(insufficient_yleaf["confidence"], serde_json::json!(0.0));
    assert_eq!(insufficient_yleaf["status"], serde_json::json!("insufficient_chromosomes"));
    assert_eq!(
        insufficient_yleaf["insufficiency_reason"],
        serde_json::json!("insufficient_chromosomes")
    );
    assert_eq!(insufficient_yleaf["x_coverage"], serde_json::json!(0.0));
    assert_eq!(insufficient_yleaf["autosomal_coverage"], serde_json::json!(0.0));
    assert_eq!(insufficient_yleaf["expectation_matched"], serde_json::json!(true));

    for row in rows {
        for key in [
            "sex_report",
            "sex_estimate",
            "population_metrics",
            "haplogroup_report",
            "sex_summary",
            "stage_metrics",
        ] {
            let path = repo_root.join(
                row[key]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing governed path for {key}"))?,
            );
            assert!(
                path.is_file(),
                "expected governed BAM sex tool-smoke artifact: {}",
                path.display()
            );
        }
    }

    Ok(())
}

fn required_array<'a>(
    payload: &'a serde_json::Value,
    key: &str,
) -> Result<&'a [serde_json::Value]> {
    payload[key].as_array().map(Vec::as_slice).ok_or_else(|| anyhow::anyhow!("{key} array missing"))
}
