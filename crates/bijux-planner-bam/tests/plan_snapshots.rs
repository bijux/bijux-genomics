use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_core::contract::PlanPolicy;
use bijux_core::prelude::{
    ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StageVersion, ToolConstraints, ToolId,
};
use bijux_planner_bam::{
    pipeline_stage_ids, plan_bam_to_bam__adna_capture__v1, plan_bam_to_bam__adna_shotgun__v1,
    BamPipelineInputs, BamPlanConfig, BamPlanner,
};
use bijux_stage_contract::StagePlanV1;

fn plan_for(stage_id: &str, tool_id: &str) -> StagePlanV1 {
    StagePlanV1 {
        stage_id: StageId::new(stage_id),
        stage_version: StageVersion(1),
        tool_id: ToolId::new(tool_id),
        tool_version: "0.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: format!("{tool_id}:latest"),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec![tool_id.to_string()],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        io: bijux_stage_contract::StageIO {
            inputs: vec![bijux_stage_contract::ArtifactRef::required(
                ArtifactId::from_static("input"),
                PathBuf::from("input.bam"),
                bijux_core::contract::ArtifactRole::Bam,
            )],
            outputs: vec![bijux_stage_contract::ArtifactRef::required(
                ArtifactId::from_static("output"),
                PathBuf::from("output.bam"),
                bijux_core::contract::ArtifactRole::Bam,
            )],
        },
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({"sample_id":"s1"}),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
        reason: bijux_stage_contract::PlanDecisionReason::default(),
    }
}

#[test]
fn bam_planner_plan_snapshot() {
    let stages = vec![
        plan_for("bam.qc_pre", "samtools"),
        plan_for("bam.qc_post", "mosdepth"),
    ];
    let config = BamPlanConfig {
        pipeline_id: "bam.qc.v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        stages,
    };
    let plan = BamPlanner::plan(&config).expect("plan");
    let json = serde_json::to_value(&plan).expect("serialize");
    insta::assert_json_snapshot!(json);
}

#[test]
fn adna_shotgun_plan_snapshot_is_stable() {
    if !cfg!(feature = "bam_downstream") {
        return;
    }
    let mut tool_specs = BTreeMap::new();
    for stage_id in pipeline_stage_ids("bam-to-bam__adna_shotgun__v1") {
        tool_specs.insert(
            stage_id.clone(),
            bijux_core::contract::ToolExecutionSpecV1 {
                tool_id: ToolId::new(format!("{stage_id}.tool")),
                tool_version: "0.0.0".to_string(),
                image: ContainerImageRefV1 {
                    image: "bijux/test".to_string(),
                    digest: Some("sha256:bam".to_string()),
                },
                command: CommandSpecV1 {
                    template: vec!["echo".to_string()],
                },
                resources: ToolConstraints {
                    runtime: "docker".to_string(),
                    mem_gb: 1,
                    tmp_gb: 1,
                    threads: 1,
                },
            },
        );
    }
    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs,
        params_overrides: BTreeMap::new(),
        bam: PathBuf::from("input.bam"),
        bam_index: Some(PathBuf::from("input.bam.bai")),
        reference: Some(PathBuf::from("ref.fa")),
        sample_id: Some("sample".to_string()),
        out_dir: PathBuf::from("out"),
    };
    let plan = plan_bam_to_bam__adna_shotgun__v1(&inputs).expect("plan");
    insta::assert_json_snapshot!("bam_adna_shotgun_plan", plan);
}

#[test]
fn adna_capture_plan_snapshot_is_stable() {
    if !cfg!(feature = "bam_downstream") {
        return;
    }
    let mut tool_specs = BTreeMap::new();
    for stage_id in pipeline_stage_ids("bam-to-bam__adna_capture__v1") {
        tool_specs.insert(
            stage_id.clone(),
            bijux_core::contract::ToolExecutionSpecV1 {
                tool_id: ToolId::new(format!("{stage_id}.tool")),
                tool_version: "0.0.0".to_string(),
                image: ContainerImageRefV1 {
                    image: "bijux/test".to_string(),
                    digest: Some("sha256:bam".to_string()),
                },
                command: CommandSpecV1 {
                    template: vec!["echo".to_string()],
                },
                resources: ToolConstraints {
                    runtime: "docker".to_string(),
                    mem_gb: 1,
                    tmp_gb: 1,
                    threads: 1,
                },
            },
        );
    }
    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs,
        params_overrides: BTreeMap::new(),
        bam: PathBuf::from("input.bam"),
        bam_index: Some(PathBuf::from("input.bam.bai")),
        reference: Some(PathBuf::from("ref.fa")),
        sample_id: Some("sample".to_string()),
        out_dir: PathBuf::from("out"),
    };
    let plan = plan_bam_to_bam__adna_capture__v1(&inputs).expect("plan");
    insta::assert_json_snapshot!("bam_adna_capture_plan", plan);
}
