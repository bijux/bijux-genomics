use std::collections::BTreeMap;

use bijux_dna_stage_contract::{
    ExecutionPlan, StageInvocationV1, StagePlanV1, StagePluginOutputV1,
};

fn stage_plan() -> StagePlanV1 {
    StagePlanV1 {
        stage_id: bijux_dna_core::ids::StageId::new("fastq.trim_reads"),
        stage_instance_id: None,
        stage_version: bijux_dna_core::ids::StageVersion(1),
        tool_id: bijux_dna_core::ids::ToolId::new("fastp"),
        tool_version: "1.0".to_string(),
        image: bijux_dna_core::prelude::ContainerImageRefV1 {
            image: "fastp".to_string(),
            digest: None,
        },
        command: bijux_dna_core::prelude::CommandSpecV1 { template: vec!["fastp".to_string()] },
        resources: bijux_dna_core::contract::ToolConstraints::default(),
        io: bijux_dna_core::contract::StageIO {
            inputs: vec![bijux_dna_core::contract::ArtifactRef::required(
                bijux_dna_core::ids::ArtifactId::new("reads_in"),
                "reads.fq".into(),
                bijux_dna_core::contract::ArtifactRole::Reads,
            )],
            outputs: vec![bijux_dna_core::contract::ArtifactRef::required(
                bijux_dna_core::ids::ArtifactId::new("reads_out"),
                "reads.trimmed.fq".into(),
                bijux_dna_core::contract::ArtifactRole::TrimmedReads,
            )],
        },
        out_dir: "out/fastq.trim_reads".into(),
        params: serde_json::json!({"quality": 20}),
        effective_params: serde_json::json!({"quality": 20}),
        aux_images: BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    }
}

fn write_snapshot(path: &str, value: &serde_json::Value) {
    let canonical = bijux_dna_core::contract::canonical::to_canonical_json_bytes(value)
        .unwrap_or_else(|err| panic!("canonicalize {path}: {err}"));
    let actual =
        String::from_utf8(canonical).unwrap_or_else(|err| panic!("decode {path} as utf8: {err}"));
    let expected = include_str!("../../fixtures/public_types/default/stage_plan.json");
    if path.ends_with("stage_plan.json") {
        assert_eq!(actual, expected);
        return;
    }
    let expected = include_str!("../../fixtures/public_types/default/run_execution_plan.json");
    if path.ends_with("run_execution_plan.json") {
        assert_eq!(actual, expected);
        return;
    }
    let expected = include_str!("../../fixtures/public_types/default/execution_plan.json");
    if path.ends_with("execution_plan.json") {
        assert_eq!(actual, expected);
        return;
    }
    let expected = include_str!("../../fixtures/public_types/default/stage_invocation.json");
    if path.ends_with("stage_invocation.json") {
        assert_eq!(actual, expected);
        return;
    }
    let expected = include_str!("../../fixtures/public_types/default/stage_plugin_output.json");
    if path.ends_with("stage_plugin_output.json") {
        assert_eq!(actual, expected);
        return;
    }
    let expected = include_str!("../../fixtures/public_types/default/run_execution_plan.json");
    assert_eq!(actual, expected);
}

#[test]
fn stage_plan_snapshot() {
    let plan = stage_plan();
    write_snapshot(
        "stage_plan.json",
        &serde_json::to_value(plan).unwrap_or_else(|err| panic!("serialize stage plan: {err}")),
    );
}

#[test]
fn execution_plan_snapshot() {
    let plan = stage_plan();
    let execution = ExecutionPlan::new(
        "fastq-to-fastq__default__v1",
        "planner",
        bijux_dna_core::contract::PlanPolicy::default(),
        vec![plan.clone()],
        Vec::new(),
    )
    .unwrap_or_else(|err| panic!("build execution plan: {err}"));
    write_snapshot(
        "execution_plan.json",
        &serde_json::to_value(execution)
            .unwrap_or_else(|err| panic!("serialize execution plan: {err}")),
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
        &serde_json::to_value(invocation)
            .unwrap_or_else(|err| panic!("serialize invocation: {err}")),
    );
}

#[test]
fn stage_plugin_output_snapshot() {
    let output = StagePluginOutputV1 {
        metrics: bijux_dna_core::metrics::MetricsEnvelope {
            schema_version: "bijux.metrics_envelope.v2".to_string(),
            contract_version: bijux_dna_core::contract::ContractVersion::v1(),
            stage_id: "fastq.trim_reads".to_string(),
            stage_version: 1,
            tool_id: "fastp".to_string(),
            tool_version: "1.0".to_string(),
            image_digest: "fastp".to_string(),
            parameters_fingerprint: "params-hash".to_string(),
            input_fingerprint: "input-hash".to_string(),
            parameters_json_normalized: serde_json::json!({"quality": 20}),
            input_hashes: Vec::new(),
            metric_provenance: None,
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
        &serde_json::to_value(output).unwrap_or_else(|err| panic!("serialize output: {err}")),
    );
}

#[test]
fn run_execution_plan_snapshot() {
    let plan = stage_plan();
    let run_plan = bijux_dna_stage_contract::RunExecutionPlan {
        run_id: bijux_dna_core::ids::RunId("run-1".to_string()),
        run_dir: "runs/run-1".into(),
        logs_dir: "runs/run-1/logs".into(),
        artifacts_dir: "runs/run-1/artifacts".into(),
        planned_artifacts: vec![bijux_dna_stage_contract::PlannedArtifactV1 {
            artifact_id: "trimmed_r1".to_string(),
            role: "trimmed_reads".to_string(),
            path: "trimmed.fastq.gz".to_string(),
            kind: "fastq".to_string(),
            schema: "bijux.artifact.fastq.v1".to_string(),
        }],
        stage: plan,
        tool: bijux_dna_core::contract::ToolExecutionSpecV1 {
            tool_id: bijux_dna_core::ids::ToolId::new("fastp"),
            tool_version: "1.0".to_string(),
            image: bijux_dna_core::prelude::ContainerImageRefV1 {
                image: "fastp".to_string(),
                digest: None,
            },
            command: bijux_dna_core::prelude::CommandSpecV1 { template: vec!["fastp".to_string()] },
            resources: bijux_dna_core::contract::ToolConstraints::default(),
        },
    };
    write_snapshot(
        "run_execution_plan.json",
        &serde_json::to_value(run_plan).unwrap_or_else(|err| panic!("serialize run plan: {err}")),
    );
}
