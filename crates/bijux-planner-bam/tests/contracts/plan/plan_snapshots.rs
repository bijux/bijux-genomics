use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_core::contract::PlanPolicy;
use bijux_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_planner_bam::{
    plan_bam_to_bam__adna_capture__v1, plan_bam_to_bam__adna_shotgun__v1, BamPipelineInputs,
    BamPlanConfig, BamPlanner,
};
use bijux_testkit::snapshot_name;

#[test]
fn bam_plan_snapshot() {
    let tool_align = ToolExecutionSpecV1 {
        tool_id: ToolId::from_static("bwa"),
        tool_version: "0.7.17".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/bwa".to_string(),
            digest: Some("sha256:bwa".to_string()),
        },
        command: CommandSpecV1 {
            template: vec!["bwa".to_string()],
        },
        resources: ToolConstraints {
            runtime: "short".to_string(),
            mem_gb: 2,
            tmp_gb: 1,
            threads: 2,
        },
    };
    let config = BamPlanConfig {
        pipeline_id: "bam-to-bam__adna_shotgun__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        stages: vec!["bam.align".to_string()],
        tools: vec![tool_align],
        aux_images: BTreeMap::new(),
        bam: PathBuf::from("sample.bam"),
        bam_index: None,
        reference: None,
        sample_id: Some("sample".to_string()),
        out_dir: PathBuf::from("out"),
    };
    let plan = BamPlanner::plan(&config).expect("plan");
    let name = snapshot_name("contracts", "bam_plan_snapshot");
    insta::assert_json_snapshot!(name, plan);
}

#[test]
fn bam_adna_shotgun_plan_snapshot() -> anyhow::Result<()> {
    let temp = bijux_infra::temp_dir("bam-adna-shotgun-plan")?;
    let bam = temp.path().join("sample.bam");
    std::fs::write(&bam, b"")?;
    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs: BTreeMap::new(),
        params_overrides: BTreeMap::new(),
        bam: bam.clone(),
        bam_index: None,
        reference: None,
        sample_id: Some("sample".to_string()),
        out_dir: temp.path().join("out"),
    };
    let plan = plan_bam_to_bam__adna_shotgun__v1(&inputs)?;
    let name = snapshot_name("contracts", "bam_adna_shotgun_plan");
    insta::assert_json_snapshot!(name, plan);
    Ok(())
}

#[test]
fn bam_adna_capture_plan_snapshot() -> anyhow::Result<()> {
    let temp = bijux_infra::temp_dir("bam-adna-capture-plan")?;
    let bam = temp.path().join("sample.bam");
    std::fs::write(&bam, b"")?;
    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs: BTreeMap::new(),
        params_overrides: BTreeMap::new(),
        bam: bam.clone(),
        bam_index: None,
        reference: None,
        sample_id: Some("sample".to_string()),
        out_dir: temp.path().join("out"),
    };
    let plan = plan_bam_to_bam__adna_capture__v1(&inputs)?;
    let name = snapshot_name("contracts", "bam_adna_capture_plan");
    insta::assert_json_snapshot!(name, plan);
    Ok(())
}
