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
fn write_local_damage_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.damage");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_damage_smoke_report()?;
    assert_eq!(report_path, repo_root.join("target/local-smoke/bam.damage/damage.json"));
    assert!(report_path.is_file(), "local-smoke BAM damage report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.damage"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.damage.local_smoke.report.v1")
    );
    assert_eq!(payload["sample_id"], serde_json::json!("core-v1-damage-short-fragments"));
    assert_eq!(payload["expectation_matched"], serde_json::json!(true));
    assert_eq!(payload["method"], serde_json::json!("pydamage"));
    assert_eq!(payload["tools_seen"], serde_json::json!(["pydamage", "mapdamage2"]));
    assert_eq!(payload["terminal_c_to_t_5p"], serde_json::json!(0.18));
    assert_eq!(payload["terminal_g_to_a_3p"], serde_json::json!(0.11));
    assert_eq!(payload["short_fragment_fraction"], serde_json::json!(1.0));
    assert_eq!(payload["damage_signal"], serde_json::json!("moderate"));
    assert_eq!(payload["strict_profile_upgraded"], serde_json::json!(false));

    let damage_report = repo_root.join(
        payload["damage_report"].as_str().unwrap_or_else(|| panic!("damage_report path missing")),
    );
    let terminal_position_metrics = repo_root.join(
        payload["terminal_position_metrics"]
            .as_str()
            .unwrap_or_else(|| panic!("terminal_position_metrics path missing")),
    );
    let parser_output = repo_root.join(
        payload["parser_output"].as_str().unwrap_or_else(|| panic!("parser_output path missing")),
    );
    let advisory_boundary = repo_root.join(
        payload["advisory_boundary"]
            .as_str()
            .unwrap_or_else(|| panic!("advisory_boundary path missing")),
    );
    let udg_regime = repo_root
        .join(payload["udg_regime"].as_str().unwrap_or_else(|| panic!("udg_regime path missing")));
    let stage_metrics = repo_root.join(
        payload["stage_metrics"].as_str().unwrap_or_else(|| panic!("stage_metrics path missing")),
    );
    for path in [
        &damage_report,
        &terminal_position_metrics,
        &parser_output,
        &advisory_boundary,
        &udg_regime,
        &stage_metrics,
    ] {
        assert!(path.is_file(), "governed BAM damage artifact must exist: {}", path.display());
    }

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&damage_report)?)?;
    assert_eq!(summary_json["schema_version"], serde_json::json!("bijux.bam.damage_evidence.v1"));
    assert_eq!(summary_json["stage_id"], serde_json::json!("bam.damage"));
    assert_eq!(summary_json["terminal_c_to_t_5p"], serde_json::json!(0.18));
    assert_eq!(summary_json["terminal_g_to_a_3p"], serde_json::json!(0.11));
    assert_eq!(summary_json["short_fragment_fraction"], serde_json::json!(1.0));
    assert_eq!(summary_json["damage_signal"], serde_json::json!("moderate"));
    assert_eq!(summary_json["strict_profile_upgraded"], serde_json::json!(false));

    let unified_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&terminal_position_metrics)?)?;
    assert_eq!(unified_json["canonical"]["c_to_t_5p"], serde_json::json!(0.18));
    assert_eq!(unified_json["canonical"]["g_to_a_3p"], serde_json::json!(0.11));
    assert_eq!(unified_json["tools_seen"], serde_json::json!(["pydamage", "mapdamage2"]));

    let parser_output_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&parser_output)?)?;
    assert_eq!(
        parser_output_json["schema_version"],
        serde_json::json!("bijux.bam.damage.parser_output.v1")
    );
    assert_eq!(parser_output_json["stage_id"], serde_json::json!("bam.damage"));
    assert_eq!(
        parser_output_json["parsed_tools"][0]["tool_id"],
        serde_json::json!("pydamage")
    );

    let advisory_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&advisory_boundary)?)?;
    assert_eq!(advisory_json["stage_id"], serde_json::json!("bam.damage"));
    assert_eq!(advisory_json["advisory_only"], serde_json::json!(true));

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.damage.stage_metrics.v1")
    );
    assert_eq!(stage_metrics_json["tool_id"], serde_json::json!("pydamage"));
    assert_eq!(stage_metrics_json["tools_seen"], serde_json::json!(["pydamage", "mapdamage2"]));
    assert_eq!(stage_metrics_json["expected_terminal_c_to_t_5p"], serde_json::json!(0.18));
    assert_eq!(stage_metrics_json["terminal_c_to_t_5p"], serde_json::json!(0.18));
    assert_eq!(stage_metrics_json["terminal_c_to_t_5p_delta"], serde_json::json!(0.0));
    assert_eq!(stage_metrics_json["expected_terminal_g_to_a_3p"], serde_json::json!(0.11));
    assert_eq!(stage_metrics_json["terminal_g_to_a_3p"], serde_json::json!(0.11));
    assert_eq!(stage_metrics_json["terminal_g_to_a_3p_delta"], serde_json::json!(0.0));
    assert_eq!(stage_metrics_json["expected_short_fragment_fraction"], serde_json::json!(1.0));
    assert_eq!(stage_metrics_json["short_fragment_fraction"], serde_json::json!(1.0));
    assert_eq!(stage_metrics_json["short_fragment_fraction_delta"], serde_json::json!(0.0));
    assert_eq!(stage_metrics_json["expected_damage_signal"], serde_json::json!("moderate"));
    assert_eq!(stage_metrics_json["damage_signal"], serde_json::json!("moderate"));
    assert_eq!(
        stage_metrics_json["expected_strict_profile_upgraded"],
        serde_json::json!(false)
    );
    assert_eq!(stage_metrics_json["strict_profile_upgraded"], serde_json::json!(false));
    assert_eq!(stage_metrics_json["expectation_matched"], serde_json::json!(true));

    Ok(())
}
