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
fn local_markdup_smoke_plans_use_governed_duplicate_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_markdup_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM markdup case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-markdup-cluster")
        .unwrap_or_else(|| panic!("governed BAM markdup case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.markdup");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(case.bam, PathBuf::from("assets/toy/core-v1/bam/markdup_duplicate_cluster.sam"));
    assert_eq!(case.expected_input_reads, 4);
    assert_eq!(case.expected_output_reads, 4);
    assert_eq!(case.expected_removed_reads, 0);
    assert_eq!(case.expected_duplicate_reads_before, 0);
    assert_eq!(case.expected_duplicate_reads_after, 1);
    assert_eq!(case.expected_duplicate_fraction, 0.25);
    assert_eq!(case.expected_newly_marked_reads, 1);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.markdup/core-v1-markdup-cluster/samtools")
    );
    assert_eq!(case.plan.params["duplicate_action"], serde_json::json!("mark"));
    assert_eq!(case.plan.params["optical_duplicates"], serde_json::json!("mark_only"));
    assert_eq!(case.plan.params["umi_policy"], serde_json::json!("ignore"));

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
            "markdup_bam",
            "markdup_bai",
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
        .unwrap_or_else(|| panic!("summary output missing from BAM markdup plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.markdup/core-v1-markdup-cluster/samtools/markdup.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_markdup_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalMarkdupSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_markdup_smoke_plans;
}

fn write_local_markdup_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-markdup.toml"), body)?;
    Ok(())
}

#[test]
fn local_markdup_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_markdup_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_markdup.v1"
tool_id = "samtools"

[[cases]]
sample_id = " "
bam = "assets/toy/core-v1/bam/markdup_duplicate_cluster.sam"
duplicate_action = "mark"
optical_duplicates = "mark_only"
umi_policy = "ignore"
expected_input_reads = 4
expected_output_reads = 4
expected_removed_reads = 0
expected_duplicate_reads_before = 0
expected_duplicate_reads_after = 1
expected_duplicate_fraction = 0.25
expected_newly_marked_reads = 1
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_markdup_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.markdup sample_id must not be empty");
    Ok(())
}

#[test]
fn local_markdup_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_markdup_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_markdup.v1"
tool_id = "samtools"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/markdup_duplicate_cluster.sam"
duplicate_action = "mark"
optical_duplicates = "mark_only"
umi_policy = "ignore"
expected_input_reads = 4
expected_output_reads = 4
expected_removed_reads = 0
expected_duplicate_reads_before = 0
expected_duplicate_reads_after = 1
expected_duplicate_fraction = 0.25
expected_newly_marked_reads = 1

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/markdup_duplicate_cluster.sam"
duplicate_action = "mark"
optical_duplicates = "mark_only"
umi_policy = "ignore"
expected_input_reads = 4
expected_output_reads = 4
expected_removed_reads = 0
expected_duplicate_reads_before = 0
expected_duplicate_reads_after = 1
expected_duplicate_fraction = 0.25
expected_newly_marked_reads = 1
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_markdup_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before plan construction");
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.markdup sample_id `duplicate-case`"
    );
    Ok(())
}

#[test]
fn markdup_plan_accepts_picard_governed_planning_contract() -> Result<()> {
    let repo_root = repo_root();
    let stage_id = StageId::new("bam.markdup".to_string());
    let tool_id = ToolId::new("picard");
    let tool_spec =
        bijux_dna_planner_bam::stage_api::load_bam_domain_tool_planning_spec(
            &repo_root,
            &stage_id,
            &tool_id,
        )?;
    let bam = PathBuf::from("assets/toy/core-v1/bam/markdup_duplicate_cluster.sam");
    let params = bijux_dna_domain_bam::params::MarkDupEffectiveParams {
        optical_duplicates: bijux_dna_domain_bam::params::OpticalDuplicatePolicy::MarkOnly,
        umi_policy: bijux_dna_domain_bam::params::UmiPolicy::Ignore,
        duplicate_action: bijux_dna_domain_bam::params::DuplicateAction::Mark,
    };
    let out_dir = PathBuf::from("target/local-smoke/bam.markdup/core-v1-markdup-cluster/picard");
    let plan = bijux_dna_planner_bam::tool_adapters::bam::markdup::plan(
        &tool_spec, &bam, &out_dir, &params,
    )?;

    assert_eq!(plan.stage_id.as_str(), "bam.markdup");
    assert_eq!(plan.tool_id.as_str(), "picard");
    assert_eq!(plan.out_dir, out_dir);
    assert_eq!(plan.params["duplicate_action"], serde_json::json!("mark"));
    assert_eq!(plan.params["optical_duplicates"], serde_json::json!("mark_only"));
    assert_eq!(plan.params["umi_policy"], serde_json::json!("ignore"));

    let output_names = plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec![
            "markdup_bam",
            "markdup_bai",
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
        .unwrap_or_else(|| panic!("summary output missing from picard BAM markdup plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.markdup/core-v1-markdup-cluster/picard/markdup.summary.json"
        )
    );

    Ok(())
}
