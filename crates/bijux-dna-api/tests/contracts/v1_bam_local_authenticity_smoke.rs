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
fn write_local_authenticity_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.authenticity");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_authenticity_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("target/local-smoke/bam.authenticity/authenticity.json")
    );
    assert!(report_path.is_file(), "local-smoke BAM authenticity report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.authenticity"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.authenticity.local_smoke.report.v1")
    );
    assert_eq!(payload["sample_id"], serde_json::json!("adna_like_damage"));
    assert_eq!(payload["expectation_matched"], serde_json::json!(true));
    assert_eq!(payload["method"], serde_json::json!("authenticct"));
    assert_eq!(payload["score"], serde_json::json!(0.5333333333333333));
    assert_eq!(payload["confidence"], serde_json::json!(0.8133333333333334));
    assert_eq!(payload["pmd_like_signal_present"], serde_json::json!(true));
    assert_eq!(
        payload["consumed_metrics"],
        serde_json::json!(["damage", "contamination", "complexity", "coverage", "mapping"])
    );
    assert_eq!(payload["missing_metrics"], serde_json::json!([]));

    let authenticity_report = repo_root.join(
        payload["authenticity_report"]
            .as_str()
            .unwrap_or_else(|| panic!("authenticity_report path missing")),
    );
    let authenticity_summary = repo_root.join(
        payload["authenticity_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("authenticity_summary path missing")),
    );
    let authenticity_composite = repo_root.join(
        payload["authenticity_composite"]
            .as_str()
            .unwrap_or_else(|| panic!("authenticity_composite path missing")),
    );
    let advisory_boundary = repo_root.join(
        payload["advisory_boundary"]
            .as_str()
            .unwrap_or_else(|| panic!("advisory_boundary path missing")),
    );
    let stage_metrics = repo_root.join(
        payload["stage_metrics"].as_str().unwrap_or_else(|| panic!("stage_metrics path missing")),
    );
    let damage_unified_metrics = repo_root.join(
        payload["damage_unified_metrics"]
            .as_str()
            .unwrap_or_else(|| panic!("damage_unified_metrics path missing")),
    );
    let contamination_summary = repo_root.join(
        payload["contamination_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("contamination_summary path missing")),
    );
    let complexity_summary = repo_root.join(
        payload["complexity_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("complexity_summary path missing")),
    );
    let coverage_regime = repo_root.join(
        payload["coverage_regime"]
            .as_str()
            .unwrap_or_else(|| panic!("coverage_regime path missing")),
    );
    let mapping_summary = repo_root.join(
        payload["mapping_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("mapping_summary path missing")),
    );
    for path in [
        &authenticity_report,
        &authenticity_summary,
        &authenticity_composite,
        &advisory_boundary,
        &stage_metrics,
        &damage_unified_metrics,
        &contamination_summary,
        &complexity_summary,
        &coverage_regime,
        &mapping_summary,
    ] {
        assert!(
            path.is_file(),
            "governed BAM authenticity artifact must exist: {}",
            path.display()
        );
    }

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&authenticity_summary)?)?;
    assert_eq!(
        summary_json["schema_version"],
        serde_json::json!("bijux.bam.authenticity_advisory.v1")
    );
    assert_eq!(summary_json["stage_id"], serde_json::json!("bam.authenticity"));
    assert_eq!(summary_json["score"], serde_json::json!(0.5333333333333333));
    assert_eq!(summary_json["confidence"], serde_json::json!(0.8133333333333334));
    assert_eq!(summary_json["pmd_like_signal_present"], serde_json::json!(true));

    let composite_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&authenticity_composite)?)?;
    assert_eq!(
        composite_json["schema_version"],
        serde_json::json!("bijux.bam.authenticity.composition.v1")
    );
    assert_eq!(composite_json["score"], serde_json::json!(0.5333333333333333));
    assert_eq!(composite_json["confidence"], serde_json::json!(0.8133333333333334));
    assert_eq!(composite_json["consumed_metrics"]["damage"]["available"], serde_json::json!(true));
    assert_eq!(
        composite_json["consumed_metrics"]["damage"]["source"],
        serde_json::json!("stage_artifact")
    );
    assert_eq!(
        composite_json["consumed_metrics"]["contamination"]["available"],
        serde_json::json!(true)
    );
    assert_eq!(
        composite_json["consumed_metrics"]["contamination"]["source"],
        serde_json::json!("stage_artifact")
    );
    assert_eq!(
        composite_json["consumed_metrics"]["complexity"]["available"],
        serde_json::json!(true)
    );
    assert_eq!(
        composite_json["consumed_metrics"]["complexity"]["source"],
        serde_json::json!("stage_artifact")
    );
    assert_eq!(
        composite_json["consumed_metrics"]["coverage"]["available"],
        serde_json::json!(true)
    );
    assert_eq!(
        composite_json["consumed_metrics"]["coverage"]["source"],
        serde_json::json!("stage_artifact")
    );
    assert_eq!(composite_json["consumed_metrics"]["mapping"]["available"], serde_json::json!(true));
    assert_eq!(
        composite_json["consumed_metrics"]["mapping"]["source"],
        serde_json::json!("stage_artifact")
    );

    let damage_path = PathBuf::from(
        composite_json["consumed_metrics"]["damage"]["path"]
            .as_str()
            .unwrap_or_else(|| panic!("damage path missing from authenticity composition")),
    );
    let contamination_path = PathBuf::from(
        composite_json["consumed_metrics"]["contamination"]["path"]
            .as_str()
            .unwrap_or_else(|| panic!("contamination path missing from authenticity composition")),
    );
    let complexity_path = PathBuf::from(
        composite_json["consumed_metrics"]["complexity"]["path"]
            .as_str()
            .unwrap_or_else(|| panic!("complexity path missing from authenticity composition")),
    );
    let coverage_path = PathBuf::from(
        composite_json["consumed_metrics"]["coverage"]["path"]
            .as_str()
            .unwrap_or_else(|| panic!("coverage path missing from authenticity composition")),
    );
    let mapping_path = PathBuf::from(
        composite_json["consumed_metrics"]["mapping"]["path"]
            .as_str()
            .unwrap_or_else(|| panic!("mapping path missing from authenticity composition")),
    );
    for path in [&damage_path, &contamination_path, &complexity_path, &coverage_path, &mapping_path]
    {
        assert!(
            path.is_file(),
            "authenticity composition input artifact must exist: {}",
            path.display()
        );
    }
    assert!(damage_path.ends_with(
        "target/local-smoke/bam.authenticity/adna_like_damage/damage/damage.unified_metrics.json"
    ));
    assert!(contamination_path.ends_with("target/local-smoke/bam.authenticity/adna_like_damage/contamination/contamination.summary.json"));
    assert!(complexity_path.ends_with(
        "target/local-smoke/bam.authenticity/adna_like_damage/complexity/complexity.summary.json"
    ));
    assert!(coverage_path.ends_with(
        "target/local-smoke/bam.authenticity/adna_like_damage/coverage/coverage.regime.json"
    ));
    assert!(mapping_path.ends_with(
        "target/local-smoke/bam.authenticity/adna_like_damage/mapping_summary/mapping_summary.json"
    ));

    let report_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&authenticity_report)?)?;
    assert_eq!(report_json["schema_version"], serde_json::json!("bijux.bam.authenticity.v1"));
    assert!(report_json.get("summary").is_some());
    assert!(report_json.get("composition").is_some());

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.authenticity.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["sample_id"], serde_json::json!("adna_like_damage"));
    assert_eq!(stage_metrics_json["method"], serde_json::json!("authenticct"));
    assert_eq!(stage_metrics_json["expected_score"], serde_json::json!(0.5333333333333333));
    assert_eq!(stage_metrics_json["score"], serde_json::json!(0.5333333333333333));
    assert_eq!(stage_metrics_json["score_delta"], serde_json::json!(0.0));
    assert_eq!(stage_metrics_json["expected_confidence"], serde_json::json!(0.8133333333333334));
    assert_eq!(stage_metrics_json["confidence"], serde_json::json!(0.8133333333333334));
    assert_eq!(stage_metrics_json["confidence_delta"], serde_json::json!(0.0));
    assert_eq!(stage_metrics_json["expected_pmd_like_signal_present"], serde_json::json!(true));
    assert_eq!(stage_metrics_json["pmd_like_signal_present"], serde_json::json!(true));
    assert_eq!(stage_metrics_json["contamination_estimate"], serde_json::json!(0.03));
    assert_eq!(
        stage_metrics_json["expected_consumed_metric_ids"],
        serde_json::json!(["damage", "contamination", "complexity", "coverage", "mapping"])
    );
    assert_eq!(
        stage_metrics_json["consumed_metric_ids"],
        serde_json::json!(["damage", "contamination", "complexity", "coverage", "mapping"])
    );
    assert_eq!(stage_metrics_json["missing_metric_ids"], serde_json::json!([]));
    assert_eq!(stage_metrics_json["expectation_matched"], serde_json::json!(true));

    Ok(())
}
