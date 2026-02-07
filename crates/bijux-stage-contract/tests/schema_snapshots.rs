use bijux_stage_contract::{
    ExecutionPlan, PlanEdge, PlannerContractV1, StageInvocationV1, StagePlanV1, StagePluginOutputV1,
};

#[test]
fn stage_contract_schema_snapshot() {
    let plan = StagePlanV1 {
        stage_id: bijux_core::ids::StageId::new("fastq.trim"),
        stage_version: bijux_core::ids::StageVersion(1),
        tool_id: bijux_core::ids::ToolId::new("fastp"),
        tool_version: "1.0".to_string(),
        image: bijux_core::foundation::ContainerImageRefV1 {
            image: "fastp".to_string(),
            digest: None,
        },
        command: bijux_core::foundation::CommandSpecV1 {
            template: vec!["fastp".to_string()],
        },
        resources: bijux_core::contract::ToolConstraints::default(),
        io: bijux_core::contract::StageIO {
            inputs: vec![],
            outputs: vec![],
        },
        out_dir: "out".into(),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        aux_images: Default::default(),
        reason: bijux_stage_contract::PlanDecisionReason::default(),
    };
    let execution = ExecutionPlan::new(
        "fastq-to-fastq__default__v1",
        "planner",
        bijux_core::contract::PlanPolicy::default(),
        vec![plan.clone()],
        vec![PlanEdge::new(
            plan.stage_id.to_string(),
            plan.stage_id.to_string(),
        )],
    )
    .expect("execution plan");
    let invocation = StageInvocationV1 {
        command: vec!["fastp".to_string()],
        env: std::collections::BTreeMap::new(),
        expected_outputs: Vec::new(),
    };
    let output = StagePluginOutputV1 {
        metrics: bijux_core::metrics::MetricsEnvelope {
            schema_version: "bijux.metrics_envelope.v2".to_string(),
            contract_version: bijux_core::contract::ContractVersion::v1(),
            stage_id: "fastq.trim".to_string(),
            stage_version: 1,
            tool_id: "fastp".to_string(),
            tool_version: "1.0".to_string(),
            image_digest: "fastp".to_string(),
            parameters_fingerprint: "params-hash".to_string(),
            input_fingerprint: "input-hash".to_string(),
            parameters_json_normalized: serde_json::json!({}),
            input_hashes: Vec::new(),
            metrics: serde_json::json!({}),
        },
        artifacts: Vec::new(),
        report_parts: Vec::new(),
        warnings: Vec::new(),
        invariants: Vec::new(),
        verdict: None,
        event_hints: Vec::new(),
    };

    let expected = include_str!("fixtures/stage_contract_schema.json");
    let payload = serde_json::json!({
        "plan": plan,
        "execution": execution,
        "invocation": invocation,
        "output": output,
    });
    let actual = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(&payload).expect("canonical"),
    )
    .expect("utf8");
    assert_eq!(actual, expected);
}
