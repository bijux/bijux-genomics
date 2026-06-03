use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_damage_smoke_plans_use_governed_bam_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_damage_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed local-smoke config must keep exactly one BAM damage case");

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-damage-short-fragments")
        .unwrap_or_else(|| panic!("governed BAM damage case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.damage");
    assert_eq!(case.plan.tool_id.as_str(), "pydamage");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(case.bam, PathBuf::from("assets/toy/core-v1/bam/damage_short_fragments.sam"));
    assert_eq!(case.expected_terminal_c_to_t_5p, 0.18);
    assert_eq!(case.expected_terminal_g_to_a_3p, 0.11);
    assert_eq!(case.expected_short_fragment_fraction, 1.0);
    assert_eq!(case.expected_damage_signal, "moderate");
    assert!(!case.expected_strict_profile_upgraded);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.damage/core-v1-damage-short-fragments/pydamage")
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!("assets/toy/core-v1/bam/damage_short_fragments.sam")
    );
    assert_eq!(case.plan.params["udg_model"], serde_json::json!("non_udg"));
    assert_eq!(case.plan.params["damage_tool_profile"], serde_json::json!("ancient_dna_evidence"));
    assert_eq!(case.plan.params["evidence_only"], serde_json::json!(true));

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec!["damage_report", "terminal_position_metrics", "parser_output", "stage_metrics"]
    );

    let damage_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "damage_report")
        .unwrap_or_else(|| panic!("damage report output missing from BAM plan"));
    assert_eq!(
        damage_output.path,
        PathBuf::from(
            "target/local-smoke/bam.damage/core-v1-damage-short-fragments/pydamage/damage.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_damage_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalDamageSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_damage_smoke_plans;
}

fn write_local_damage_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-damage.toml"), body)?;
    Ok(())
}

#[test]
fn local_damage_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_damage_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_damage.v1"
tool_id = "pydamage"

[[cases]]
sample_id = " "
bam = "assets/toy/core-v1/bam/damage_short_fragments.sam"
expected_terminal_c_to_t_5p = 0.18
expected_terminal_g_to_a_3p = 0.11
expected_short_fragment_fraction = 1.0
expected_damage_signal = "moderate"
expected_strict_profile_upgraded = false
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_damage_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before damage plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.damage sample_id must not be empty");
    Ok(())
}

#[test]
fn local_damage_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_damage_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_damage.v1"
tool_id = "pydamage"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/damage_short_fragments.sam"
expected_terminal_c_to_t_5p = 0.18
expected_terminal_g_to_a_3p = 0.11
expected_short_fragment_fraction = 1.0
expected_damage_signal = "moderate"
expected_strict_profile_upgraded = false

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/damage_short_fragments.sam"
expected_terminal_c_to_t_5p = 0.18
expected_terminal_g_to_a_3p = 0.11
expected_short_fragment_fraction = 1.0
expected_damage_signal = "moderate"
expected_strict_profile_upgraded = false
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_damage_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before damage plan construction");
    assert_eq!(error.to_string(), "duplicate local-smoke bam.damage sample_id `duplicate-case`");
    Ok(())
}
