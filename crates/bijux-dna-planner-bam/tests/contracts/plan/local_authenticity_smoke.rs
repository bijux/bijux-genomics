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
fn local_authenticity_smoke_plans_use_governed_bam_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_authenticity_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM authenticity case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "adna_damage_non_udg")
        .unwrap_or_else(|| panic!("governed BAM authenticity case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.authenticity");
    assert_eq!(case.plan.tool_id.as_str(), "authenticct");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam"
        )
    );
    assert_eq!(case.damage_terminal_c_to_t_5p, 0.18);
    assert_eq!(case.damage_terminal_g_to_a_3p, 0.11);
    assert_eq!(case.contamination_method, "mitochondrial_panel_screen");
    assert_eq!(case.contamination_estimate, 0.03);
    assert_eq!(case.contamination_ci_low, 0.01);
    assert_eq!(case.contamination_ci_high, 0.05);
    assert_eq!(case.complexity_min_reads, 3);
    assert_eq!(case.complexity_projection_points, vec![6, 12]);
    assert_eq!(case.coverage_depth_thresholds, vec![1, 5, 10]);
    assert_eq!(case.expected_score, 0.5333333333333333);
    assert_eq!(case.expected_confidence, 0.8133333333333334);
    assert!(case.expected_pmd_like_signal_present);
    assert_eq!(
        case.expected_consumed_metrics,
        vec![
            "damage".to_string(),
            "contamination".to_string(),
            "complexity".to_string(),
            "coverage".to_string(),
            "mapping".to_string(),
        ]
    );
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.authenticity/adna_damage_non_udg/authenticct")
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam"
        )
    );
    assert_eq!(case.plan.params["mode"], serde_json::json!("aggregate"));
    assert_eq!(case.plan.params["pmd_filter_enabled"], serde_json::json!(false));
    assert_eq!(case.plan.params["evidence_only"], serde_json::json!(true));
    assert_eq!(case.plan.params["disallow_certification"], serde_json::json!(true));

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(output_names, vec!["authenticity_report", "summary", "stage_metrics"]);

    let authenticity_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "authenticity_report")
        .unwrap_or_else(|| panic!("authenticity report output missing from BAM plan"));
    assert_eq!(
        authenticity_output.path,
        PathBuf::from(
            "target/local-smoke/bam.authenticity/adna_damage_non_udg/authenticct/authenticity.json"
        )
    );

    Ok(())
}

#[test]
fn local_authenticity_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalAuthenticitySmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_authenticity_smoke_plans;
}

fn write_local_authenticity_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("benchmarks/configs/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-authenticity.toml"), body)?;
    Ok(())
}

fn stage_api_temp_repo() -> Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    let repo_root = repo_root();
    let tool_dir = temp.path().join("domain/bam/tools");
    fs::create_dir_all(&tool_dir)?;
    fs::copy(
        repo_root.join("domain/bam/tools/authenticct.yaml"),
        tool_dir.join("authenticct.yaml"),
    )?;
    Ok(temp)
}

#[test]
fn local_authenticity_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_authenticity_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_authenticity.v1"
tool_id = "authenticct"

[[cases]]
sample_id = " "
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/adna_like_damage.sam"
damage_terminal_c_to_t_5p = 0.18
damage_terminal_g_to_a_3p = 0.11
contamination_method = "mitochondrial_panel_screen"
contamination_estimate = 0.03
contamination_ci_low = 0.01
contamination_ci_high = 0.05
complexity_min_reads = 3
complexity_projection_points = [6, 12]
coverage_depth_thresholds = [1, 5, 10]
expected_score = 0.8666666666666667
expected_confidence = 0.9466666666666668
expected_pmd_like_signal_present = true
expected_consumed_metrics = ["damage", "contamination", "complexity", "coverage", "mapping"]
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_authenticity_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before authenticity plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.authenticity sample_id must not be empty");
    Ok(())
}

#[test]
fn local_authenticity_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_authenticity_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_authenticity.v1"
tool_id = "authenticct"

[[cases]]
sample_id = "duplicate-case"
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/adna_like_damage.sam"
damage_terminal_c_to_t_5p = 0.18
damage_terminal_g_to_a_3p = 0.11
contamination_method = "mitochondrial_panel_screen"
contamination_estimate = 0.03
contamination_ci_low = 0.01
contamination_ci_high = 0.05
complexity_min_reads = 3
complexity_projection_points = [6, 12]
coverage_depth_thresholds = [1, 5, 10]
expected_score = 0.8666666666666667
expected_confidence = 0.9466666666666668
expected_pmd_like_signal_present = true
expected_consumed_metrics = ["damage", "contamination", "complexity", "coverage", "mapping"]

[[cases]]
sample_id = "duplicate-case"
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/adna_like_damage.sam"
damage_terminal_c_to_t_5p = 0.18
damage_terminal_g_to_a_3p = 0.11
contamination_method = "mitochondrial_panel_screen"
contamination_estimate = 0.03
contamination_ci_low = 0.01
contamination_ci_high = 0.05
complexity_min_reads = 3
complexity_projection_points = [6, 12]
coverage_depth_thresholds = [1, 5, 10]
expected_score = 0.8666666666666667
expected_confidence = 0.9466666666666668
expected_pmd_like_signal_present = true
expected_consumed_metrics = ["damage", "contamination", "complexity", "coverage", "mapping"]
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_authenticity_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before authenticity plan construction");
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.authenticity sample_id `duplicate-case`"
    );
    Ok(())
}

#[test]
fn local_authenticity_smoke_plans_require_contamination_interval_to_contain_estimate() -> Result<()>
{
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_authenticity_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_authenticity.v1"
tool_id = "authenticct"

[[cases]]
sample_id = "bad-contamination-interval"
bam = "{bam}"
damage_terminal_c_to_t_5p = 0.18
damage_terminal_g_to_a_3p = 0.11
contamination_method = "mitochondrial_panel_screen"
contamination_estimate = 0.07
contamination_ci_low = 0.01
contamination_ci_high = 0.05
complexity_min_reads = 3
complexity_projection_points = [6, 12]
coverage_depth_thresholds = [1, 5, 10]
expected_score = 0.8666666666666667
expected_confidence = 0.9466666666666668
expected_pmd_like_signal_present = true
expected_consumed_metrics = ["damage", "contamination", "complexity", "coverage", "mapping"]
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/adna_like_damage.sam")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_authenticity_smoke_plans(temp.path())
        .expect_err("contamination estimate must stay inside its governed confidence interval");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.authenticity case `bad-contamination-interval` must keep contamination_estimate within the declared confidence interval"
    );
    Ok(())
}

#[test]
fn local_authenticity_smoke_plans_require_governed_consumed_metric_ids() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_authenticity_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_authenticity.v1"
tool_id = "authenticct"

[[cases]]
sample_id = "wrong-metric-ids"
bam = "{bam}"
damage_terminal_c_to_t_5p = 0.18
damage_terminal_g_to_a_3p = 0.11
contamination_method = "mitochondrial_panel_screen"
contamination_estimate = 0.03
contamination_ci_low = 0.01
contamination_ci_high = 0.05
complexity_min_reads = 3
complexity_projection_points = [6, 12]
coverage_depth_thresholds = [1, 5, 10]
expected_score = 0.8666666666666667
expected_confidence = 0.9466666666666668
expected_pmd_like_signal_present = true
expected_consumed_metrics = ["damage", "coverage", "mapping"]
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/adna_like_damage.sam")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_authenticity_smoke_plans(temp.path())
        .expect_err("authenticity composition must keep the governed consumed metric ids");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.authenticity case `wrong-metric-ids` must keep expected_consumed_metrics aligned with the governed composition inputs"
    );
    Ok(())
}

#[test]
fn local_authenticity_smoke_plans_require_expected_score_to_match_governed_advisory() -> Result<()>
{
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_authenticity_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_authenticity.v1"
tool_id = "authenticct"

[[cases]]
sample_id = "score-drift"
bam = "{bam}"
damage_terminal_c_to_t_5p = 0.18
damage_terminal_g_to_a_3p = 0.11
contamination_method = "mitochondrial_panel_screen"
contamination_estimate = 0.03
contamination_ci_low = 0.01
contamination_ci_high = 0.05
complexity_min_reads = 3
complexity_projection_points = [6, 12]
coverage_depth_thresholds = [1, 5, 10]
expected_score = 0.9
expected_confidence = 0.9466666666666668
expected_pmd_like_signal_present = true
expected_consumed_metrics = ["damage", "contamination", "complexity", "coverage", "mapping"]
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/adna_like_damage.sam")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_authenticity_smoke_plans(temp.path())
        .expect_err("authenticity expected_score must match the governed advisory");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.authenticity case `score-drift` must keep expected_score aligned with the governed authenticity advisory"
    );
    Ok(())
}
