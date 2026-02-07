use std::collections::BTreeMap;

use bijux_stage_contract::{
    ExecutionPlan, PlanEdge, PlannerContractV1, StageInvocationV1, StagePlanV1,
    StagePluginOutputV1,
};

fn stage_plan() -> StagePlanV1 {
    StagePlanV1 {
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
            inputs: vec![bijux_core::contract::ArtifactRef::required(
                bijux_core::ids::ArtifactId::new("reads_in"),
                "reads.fq".into(),
                bijux_core::contract::ArtifactRole::Reads,
            )],
            outputs: vec![bijux_core::contract::ArtifactRef::required(
                bijux_core::ids::ArtifactId::new("reads_out"),
                "reads.trimmed.fq".into(),
                bijux_core::contract::ArtifactRole::TrimmedReads,
            )],
        },
        out_dir: "out/fastq.trim".into(),
        params: serde_json::json!({"quality": 20}),
        effective_params: serde_json::json!({"quality": 20}),
        aux_images: BTreeMap::new(),
        reason: bijux_stage_contract::PlanDecisionReason::default(),
    }
}

fn write_snapshot(path: &str, value: &serde_json::Value) {
    let actual = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(value).expect("canonical"),
    )
    .expect("utf8");
    let expected = include_str!("fixtures/public_types/stage_plan.json");
    if path.ends_with("stage_plan.json") {
        assert_eq!(actual, expected);
        return;
    }
    let expected = include_str!("fixtures/public_types/execution_plan.json");
    if path.ends_with("execution_plan.json") {
        assert_eq!(actual, expected);
        return;
    }
    let expected = include_str!("fixtures/public_types/stage_invocation.json");
    if path.ends_with("stage_invocation.json") {
        assert_eq!(actual, expected);
        return;
    }
    let expected = include_str!("fixtures/public_types/stage_plugin_output.json");
    if path.ends_with("stage_plugin_output.json") {
        assert_eq!(actual, expected);
        return;
    }
    let expected = include_str!("fixtures/public_types/run_execution_plan.json");
    assert_eq!(actual, expected);
}

#[test]
fn stage_plan_snapshot() {
    let plan = stage_plan();
    write_snapshot(
        "stage_plan.json",
        &serde_json::to_value(plan).expect("stage plan"),
    );
}

#[test]
fn execution_plan_snapshot() {
    let plan = stage_plan();
    let execution = ExecutionPlan::new(
        "fastq-to-fastq__default__v1",
        "planner",
        bijux_core::contract::PlanPolicy::default(),
        vec![plan.clone()],
        vec![PlanEdge::new(plan.stage_id.to_string(), plan.stage_id.to_string())],
    )
    .expect("execution plan");
    write_snapshot(
        "execution_plan.json",
        &serde_json::to_value(execution).expect("execution plan"),
    );
}

#[test]
fn stage_invocation_snapshot() {
    let invocation = StageInvocationV1 {
        command: vec!["fastp".to_string()],
        env: BTreeMap::new(),
        expected_outputs: Vec::new(),
    };
    write_snapshot(
        "stage_invocation.json",
        &serde_json::to_value(invocation).expect("invocation"),
    );
}

#[test]
fn stage_plugin_output_snapshot() {
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
            parameters_json_normalized: serde_json::json!({"quality": 20}),
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
    write_snapshot(
        "stage_plugin_output.json",
        &serde_json::to_value(output).expect("output"),
    );
}

#[test]
fn run_execution_plan_snapshot() {
    let plan = stage_plan();
    let run_plan = bijux_stage_contract::RunExecutionPlan {
        run_id: bijux_core::ids::RunId("run-1".to_string()),
        run_dir: "runs/run-1".into(),
        logs_dir: "runs/run-1/logs".into(),
        artifacts_dir: "runs/run-1/artifacts".into(),
        stage: plan,
        tool: bijux_core::contract::ToolExecutionSpecV1 {
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
        },
    };
    write_snapshot(
        "run_execution_plan.json",
        &serde_json::to_value(run_plan).expect("run plan"),
    );
}
