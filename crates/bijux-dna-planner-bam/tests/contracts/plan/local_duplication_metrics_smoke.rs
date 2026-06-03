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
fn local_duplication_metrics_smoke_plans_use_governed_duplicate_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans =
        bijux_dna_planner_bam::stage_api::local_duplication_metrics_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM duplication metrics case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-duplicate-observation")
        .unwrap_or_else(|| panic!("governed BAM duplication metrics case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.duplication_metrics");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(
        case.bam,
        PathBuf::from("assets/toy/core-v1/bam/duplication_metrics_duplicate_cluster.sam")
    );
    assert_eq!(case.expected_examined_reads, 3);
    assert_eq!(case.expected_duplicate_reads, 1);
    assert_eq!(case.expected_duplicate_fraction, 1.0 / 3.0);
    assert_eq!(case.expected_estimated_library_size, None);
    assert_eq!(
        case.expected_insufficient_library_size_reason.as_deref(),
        Some("tiny_smoke_duplicate_observation_is_insufficient_for_library_size_estimate")
    );
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/bam.duplication_metrics/core-v1-duplicate-observation/samtools"
        )
    );
    assert_eq!(case.plan.params["optical_duplicates"], serde_json::json!("mark_only"));
    assert_eq!(case.plan.params["umi_policy"], serde_json::json!("ignore"));
    assert_eq!(case.plan.params["duplicate_action"], serde_json::json!("mark"));

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec!["duplication_report", "duplication_histogram", "summary", "stage_metrics",]
    );

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("summary output missing from BAM duplication metrics plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.duplication_metrics/core-v1-duplicate-observation/samtools/duplication.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_duplication_metrics_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalDuplicationMetricsSmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_duplication_metrics_smoke_plans;
}

fn write_local_duplication_metrics_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-duplication-metrics.toml"), body)?;
    Ok(())
}

#[test]
fn local_duplication_metrics_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_duplication_metrics_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_duplication_metrics.v1"
tool_id = "samtools"

[[cases]]
sample_id = " "
bam = "assets/toy/core-v1/bam/duplication_metrics_duplicate_cluster.sam"
optical_duplicates = "mark_only"
umi_policy = "ignore"
duplicate_action = "mark"
expected_examined_reads = 3
expected_duplicate_reads = 1
expected_duplicate_fraction = 0.3333333333333333
expected_insufficient_library_size_reason = "tiny_smoke_duplicate_observation_is_insufficient_for_library_size_estimate"
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_duplication_metrics_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before plan construction");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.duplication_metrics sample_id must not be empty"
    );
    Ok(())
}

#[test]
fn local_duplication_metrics_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_duplication_metrics_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_duplication_metrics.v1"
tool_id = "samtools"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/duplication_metrics_duplicate_cluster.sam"
optical_duplicates = "mark_only"
umi_policy = "ignore"
duplicate_action = "mark"
expected_examined_reads = 3
expected_duplicate_reads = 1
expected_duplicate_fraction = 0.3333333333333333
expected_insufficient_library_size_reason = "tiny_smoke_duplicate_observation_is_insufficient_for_library_size_estimate"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/duplication_metrics_duplicate_cluster.sam"
optical_duplicates = "mark_only"
umi_policy = "ignore"
duplicate_action = "mark"
expected_examined_reads = 3
expected_duplicate_reads = 1
expected_duplicate_fraction = 0.3333333333333333
expected_insufficient_library_size_reason = "tiny_smoke_duplicate_observation_is_insufficient_for_library_size_estimate"
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_duplication_metrics_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before plan construction");
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.duplication_metrics sample_id `duplicate-case`"
    );
    Ok(())
}

#[test]
fn duplication_metrics_plan_accepts_picard_governed_planning_contract() -> Result<()> {
    let repo_root = repo_root();
    let stage_id = StageId::new("bam.duplication_metrics".to_string());
    let tool_id = ToolId::new("picard");
    let tool_spec = bijux_dna_planner_bam::stage_api::load_bam_domain_tool_planning_spec(
        &repo_root, &stage_id, &tool_id,
    )?;
    let bam = PathBuf::from("assets/toy/core-v1/bam/duplication_metrics_duplicate_cluster.sam");
    let params = bijux_dna_domain_bam::params::MarkDupEffectiveParams {
        optical_duplicates: bijux_dna_domain_bam::params::OpticalDuplicatePolicy::MarkOnly,
        umi_policy: bijux_dna_domain_bam::params::UmiPolicy::Ignore,
        duplicate_action: bijux_dna_domain_bam::params::DuplicateAction::Mark,
    };
    let out_dir = PathBuf::from(
        "target/local-smoke/bam.duplication_metrics/core-v1-duplicate-observation/picard",
    );
    let plan = bijux_dna_planner_bam::tool_adapters::stages_post::duplication_metrics::plan(
        &tool_spec, &bam, &out_dir, &params,
    )?;

    assert_eq!(plan.stage_id.as_str(), "bam.duplication_metrics");
    assert_eq!(plan.tool_id.as_str(), "picard");
    assert_eq!(plan.out_dir, out_dir);
    assert_eq!(plan.params["optical_duplicates"], serde_json::json!("mark_only"));
    assert_eq!(plan.params["umi_policy"], serde_json::json!("ignore"));
    assert_eq!(plan.params["duplicate_action"], serde_json::json!("mark"));

    let output_names = plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec!["duplication_report", "duplication_histogram", "summary", "stage_metrics",]
    );

    let summary_output =
        plan.io.outputs.iter().find(|artifact| artifact.name.as_str() == "summary").unwrap_or_else(
            || panic!("summary output missing from picard BAM duplication-metrics plan"),
        );
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.duplication_metrics/core-v1-duplicate-observation/picard/duplication.summary.json"
        )
    );

    Ok(())
}
