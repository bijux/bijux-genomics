use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_core::execution_plan::PlanPolicy;
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, StageId, StagePlanV1, StageVersion, ToolConstraints, ToolId,
};
use bijux_planner_bam::{BamPlanConfig, BamPlanner};

fn plan_for(stage_id: &str, tool_id: &str) -> StagePlanV1 {
    StagePlanV1 {
        stage_id: StageId(stage_id.to_string()),
        stage_version: StageVersion(1),
        tool_id: ToolId(tool_id.to_string()),
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
        io: bijux_core::StageIO {
            inputs: vec![bijux_core::ArtifactRef {
                name: "input".to_string(),
                path: PathBuf::from("input.bam"),
            }],
            outputs: vec![bijux_core::ArtifactRef {
                name: "output".to_string(),
                path: PathBuf::from("output.bam"),
            }],
        },
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({"sample_id":"s1"}),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
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
