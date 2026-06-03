use anyhow::Result;
use bijux_dna_core::prelude::{StageId, ToolId};
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
fn local_length_filter_smoke_plans_use_governed_threshold_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_length_filter_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM length filter case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-length-threshold")
        .unwrap_or_else(|| panic!("governed BAM length filter case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.length_filter");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(case.bam, PathBuf::from("assets/toy/core-v1/bam/length_threshold_ladder.sam"));
    assert_eq!(case.min_length, 8);
    assert_eq!(case.expected_input_reads, 4);
    assert_eq!(case.expected_kept_reads, 3);
    assert_eq!(case.expected_removed_reads, 1);
    assert_eq!(case.expected_observed_min_length, 8);
    assert_eq!(case.expected_observed_max_length, 12);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.length_filter/core-v1-length-threshold/samtools")
    );
    assert_eq!(case.plan.params["action"], serde_json::json!("length_filter"));
    assert_eq!(case.plan.params["min_length"], serde_json::json!(8));
    assert_eq!(case.plan.params["strict_transform"], serde_json::json!(true));

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec![
            "filtered_bam",
            "filtered_bai",
            "flagstat_before",
            "flagstat_after",
            "idxstats_before",
            "idxstats_after",
            "summary",
            "stage_metrics",
        ]
    );

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("summary output missing from BAM length filter plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.length_filter/core-v1-length-threshold/samtools/length_filter.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_length_filter_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalLengthFilterSmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_length_filter_smoke_plans;
}

fn write_local_length_filter_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-length-filter.toml"), body)?;
    Ok(())
}

#[test]
fn local_length_filter_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_length_filter_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_length_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = " "
bam = "assets/toy/core-v1/bam/length_threshold_ladder.sam"
min_length = 8
expected_input_reads = 4
expected_kept_reads = 3
expected_removed_reads = 1
expected_observed_min_length = 8
expected_observed_max_length = 12
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_length_filter_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.length_filter sample_id must not be empty");
    Ok(())
}

#[test]
fn local_length_filter_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_length_filter_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_length_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/length_threshold_ladder.sam"
min_length = 8
expected_input_reads = 4
expected_kept_reads = 3
expected_removed_reads = 1
expected_observed_min_length = 8
expected_observed_max_length = 12

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/length_threshold_ladder.sam"
min_length = 8
expected_input_reads = 4
expected_kept_reads = 3
expected_removed_reads = 1
expected_observed_min_length = 8
expected_observed_max_length = 12
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_length_filter_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before plan construction");
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.length_filter sample_id `duplicate-case`"
    );
    Ok(())
}

#[test]
fn length_filter_plan_accepts_picard_governed_planning_contract() -> Result<()> {
    let repo_root = repo_root();
    let stage_id = StageId::new("bam.length_filter".to_string());
    let tool_id = ToolId::new("picard");
    let tool_spec =
        bijux_dna_planner_bam::stage_api::load_bam_domain_tool_planning_spec(
            &repo_root,
            &stage_id,
            &tool_id,
        )?;
    let bam = PathBuf::from("assets/toy/core-v1/bam/length_threshold_ladder.sam");
    let params = bijux_dna_domain_bam::params::FilterEffectiveParams {
        mapq_threshold: 0,
        include_flags: vec![],
        exclude_flags: vec![],
        min_length: 8,
        remove_duplicates: false,
        base_quality_threshold: 20,
    };
    let out_dir = PathBuf::from("target/local-smoke/bam.length_filter/core-v1-length-threshold/picard");
    let plan = bijux_dna_planner_bam::tool_adapters::stages_pre::length_filter::plan(
        &tool_spec, &bam, &out_dir, &params,
    )?;

    assert_eq!(plan.stage_id.as_str(), "bam.length_filter");
    assert_eq!(plan.tool_id.as_str(), "picard");
    assert_eq!(plan.out_dir, out_dir);
    assert_eq!(plan.params["action"], serde_json::json!("length_filter"));
    assert_eq!(plan.params["min_length"], serde_json::json!(8));
    assert_eq!(plan.params["strict_transform"], serde_json::json!(true));

    let output_names = plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec![
            "filtered_bam",
            "filtered_bai",
            "flagstat_before",
            "flagstat_after",
            "idxstats_before",
            "idxstats_after",
            "summary",
            "stage_metrics",
        ]
    );

    let summary_output = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("summary output missing from picard BAM length-filter plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.length_filter/core-v1-length-threshold/picard/length_filter.summary.json"
        )
    );

    Ok(())
}
