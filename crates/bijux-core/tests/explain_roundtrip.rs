use bijux_core::explain::PlanExplainV1;
use bijux_core::plan::execution_plan::{ExecutionPlan, PlanPolicy};
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, StageId, StagePlanV1, StageVersion, ToolConstraints, ToolId,
};
use std::path::PathBuf;

#[test]
fn explain_roundtrip_is_deterministic() -> anyhow::Result<()> {
    let stage = StagePlanV1 {
        stage_id: StageId::from_static("stage.a"),
        stage_version: StageVersion(1),
        tool_id: ToolId::from_static("tool"),
        tool_version: "1.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "tool:1.0.0".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec!["tool".to_string()],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        io: bijux_core::plan::stage_plan::StageIO {
            inputs: vec![bijux_core::plan::stage_plan::ArtifactRef {
                name: "input".to_string(),
                path: PathBuf::from("input"),
            }],
            outputs: vec![bijux_core::plan::stage_plan::ArtifactRef {
                name: "output".to_string(),
                path: PathBuf::from("output"),
            }],
        },
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({"a": 1}),
        effective_params: serde_json::json!({"a": 1}),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
    };
    let plan = ExecutionPlan::new(
        "pipeline",
        "planner",
        PlanPolicy::PreferAccuracy,
        vec![stage],
        Vec::new(),
    )?;

    let explain = PlanExplainV1::from_plan(&plan);
    let json = serde_json::to_string(&explain)?;
    let parsed: PlanExplainV1 = serde_json::from_str(&json)?;
    let json_roundtrip = serde_json::to_string(&parsed)?;
    assert_eq!(json, json_roundtrip);

    let explain_again = PlanExplainV1::from_plan(&plan);
    assert_eq!(explain, explain_again);
    Ok(())
}
