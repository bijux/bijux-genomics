use bijux_stage_contract::{ExecutionPlan, PlanEdge, PlannerContractV1, StagePlanV1, StagePluginOutputV1, StageInvocationV1};

#[test]
fn stage_contract_schema_snapshot() {
    let plan = StagePlanV1 {
        stage_id: bijux_core::ids::StageId::new("fastq.trim"),
        stage_version: bijux_core::ids::StageVersion::new("v1"),
        tool_id: bijux_core::ids::ToolId::new("fastp"),
        tool_version: "1.0".to_string(),
        image: bijux_core::foundation::ContainerImageRefV1 { image: "fastp".to_string(), digest: None },
        command: bijux_core::foundation::CommandSpecV1 { tool_id: bijux_core::ids::ToolId::new("fastp"), template: "fastp".to_string(), args: vec![], working_dir: None },
        resources: bijux_core::contract::ToolConstraints::default(),
        io: bijux_core::contract::StageIO { inputs: vec![], outputs: vec![] },
        out_dir: "out".into(),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        aux_images: Default::default(),
        reason: bijux_stage_contract::PlanDecisionReason::default(),
    };
    let execution = ExecutionPlan {
        schema_version: "bijux.execution_plan.v1".to_string(),
        contract: PlannerContractV1 {
            pipeline_id: "fastq-to-fastq__default__v1".to_string(),
            planner_version: "planner".to_string(),
        },
        steps: vec![plan.clone()],
        edges: vec![PlanEdge { from: plan.stage_id.clone(), to: plan.stage_id.clone() }],
    };
    let invocation = StageInvocationV1 {
        schema_version: "bijux.stage_invocation.v1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        run_id: "run-1".to_string(),
        out_dir: "out".to_string(),
    };
    let output = StagePluginOutputV1 {
        schema_version: "bijux.stage_plugin_output.v1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        outputs: Vec::new(),
        metrics_path: None,
        report_path: None,
    };

    let expected = include_str!("fixtures/stage_contract_schema.json");
    let payload = serde_json::json!({
        "plan": plan,
        "execution": execution,
        "invocation": invocation,
        "output": output,
    });
    let actual = String::from_utf8(bijux_core::contract::canonical::to_canonical_json_bytes(&payload).expect("canonical"))
        .expect("utf8");
    assert_eq!(actual, expected);
}
