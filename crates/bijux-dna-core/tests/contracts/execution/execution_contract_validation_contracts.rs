use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::execution::{
    validate_execution_outputs, ExecutionEdge, ExecutionGraph, ExecutionStep,
};
use bijux_dna_core::contract::{
    ArtifactRole, ExecutionContract, PlanPolicy, StageIO, ToolConstraints,
};
use bijux_dna_core::metrics::{
    validate_derived_metric_id_str, ToolInvocationSpecV1, ToolInvocationV1,
};
use bijux_dna_core::prelude::{
    ArtifactId, CommandSpecV1, ContainerImageRefV1, ContractVersion, StageId, StepId, ToolId,
};

fn mk_step(step_id: &str, stage_id: &str) -> ExecutionStep {
    ExecutionStep {
        step_id: StepId::new(step_id),
        stage_id: StageId::new(stage_id),
        command: CommandSpecV1 { template: vec!["echo".to_string(), "ok".to_string()] },
        image: ContainerImageRefV1 {
            image: "local/tool:latest".to_string(),
            digest: Some("sha256:abc".to_string()),
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![bijux_dna_core::contract::ArtifactSpec::required(
                ArtifactId::new("in"),
                PathBuf::from("in.fastq"),
                ArtifactRole::Reads,
            )],
            outputs: vec![bijux_dna_core::contract::ArtifactSpec::required(
                ArtifactId::new("out"),
                PathBuf::from("out.fastq"),
                ArtifactRole::Reads,
            )],
        },
        out_dir: PathBuf::from("out"),
        aux_images: BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    }
}

#[test]
fn validate_execution_outputs_covers_contract_paths() {
    let root = tempfile::tempdir();
    assert!(root.is_ok());
    let root = root.unwrap_or_else(|err| panic!("tempdir: {err}"));
    assert!(std::fs::create_dir_all(root.path().join("nested")).is_ok());
    assert!(std::fs::write(root.path().join("ok.txt"), "x").is_ok());
    assert!(std::fs::write(root.path().join("nested/reads.fastq.gz"), "x").is_ok());

    let ok_contract = ExecutionContract {
        required_inputs: Vec::new(),
        optional_inputs: Vec::new(),
        expected_outputs: vec!["ok.txt".to_string(), "nested/*.fastq.gz".to_string()],
        optional_outputs: Vec::new(),
        forbidden_outputs: vec!["*.tmp".to_string()],
        forbid_unexpected_outputs: false,
        ..ExecutionContract::default()
    };
    assert!(validate_execution_outputs(&ok_contract, root.path()).is_ok());

    let missing_expected = ExecutionContract {
        required_inputs: Vec::new(),
        optional_inputs: Vec::new(),
        expected_outputs: vec!["missing.json".to_string()],
        optional_outputs: Vec::new(),
        forbidden_outputs: Vec::new(),
        forbid_unexpected_outputs: false,
        ..ExecutionContract::default()
    };
    assert!(validate_execution_outputs(&missing_expected, root.path()).is_err());

    let forbidden = ExecutionContract {
        required_inputs: Vec::new(),
        optional_inputs: Vec::new(),
        expected_outputs: vec!["ok.txt".to_string(), "nested/*.fastq.gz".to_string()],
        optional_outputs: Vec::new(),
        forbidden_outputs: vec!["*.fastq.gz".to_string()],
        forbid_unexpected_outputs: false,
        ..ExecutionContract::default()
    };
    assert!(validate_execution_outputs(&forbidden, root.path()).is_err());

    let strict = ExecutionContract {
        required_inputs: Vec::new(),
        optional_inputs: Vec::new(),
        expected_outputs: vec!["ok.txt".to_string()],
        optional_outputs: vec!["nested/*.fastq.gz".to_string()],
        forbidden_outputs: Vec::new(),
        forbid_unexpected_outputs: true,
        ..ExecutionContract::default()
    };
    assert!(validate_execution_outputs(&strict, root.path()).is_ok());

    let missing_dir = root.path().join("no_such_dir");
    assert!(validate_execution_outputs(&ok_contract, &missing_dir).is_err());
}

#[test]
fn execution_graph_validation_rejects_multiple_lint_failures() {
    let duplicate_steps = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![mk_step("dup", "fastq.trim_reads"), mk_step("dup", "fastq.report_qc")],
        Vec::new(),
    );
    assert!(duplicate_steps.is_err());

    let missing_stage = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![mk_step("a", "")],
        Vec::new(),
    );
    assert!(missing_stage.is_err());

    let missing_command = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![ExecutionStep {
            command: CommandSpecV1 { template: Vec::new() },
            ..mk_step("a", "fastq.trim_reads")
        }],
        Vec::new(),
    );
    assert!(missing_command.is_err());

    let missing_image = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![ExecutionStep {
            image: ContainerImageRefV1 { image: String::new(), digest: None },
            ..mk_step("a", "fastq.trim_reads")
        }],
        Vec::new(),
    );
    assert!(missing_image.is_err());

    let dup_artifacts = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![ExecutionStep {
            io: StageIO {
                inputs: vec![
                    bijux_dna_core::contract::ArtifactSpec::required(
                        ArtifactId::new("in"),
                        PathBuf::from("in.fastq"),
                        ArtifactRole::Reads,
                    ),
                    bijux_dna_core::contract::ArtifactSpec::required(
                        ArtifactId::new("in"),
                        PathBuf::from("in2.fastq"),
                        ArtifactRole::Reads,
                    ),
                ],
                outputs: vec![bijux_dna_core::contract::ArtifactSpec::required(
                    ArtifactId::new("out"),
                    PathBuf::from("out.fastq"),
                    ArtifactRole::Reads,
                )],
            },
            ..mk_step("a", "fastq.trim_reads")
        }],
        Vec::new(),
    );
    assert!(dup_artifacts.is_err());

    let unknown_edge = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![mk_step("a", "fastq.trim_reads")],
        vec![ExecutionEdge::new(StepId::new("a"), StepId::new("missing"))],
    );
    assert!(unknown_edge.is_err());
}

#[test]
fn derived_metric_validation_and_tool_invocation_constructor_paths() {
    assert!(validate_derived_metric_id_str("read_retention").is_ok());
    assert!(validate_derived_metric_id_str("base_retention").is_ok());
    assert!(validate_derived_metric_id_str("merge_efficiency").is_ok());
    assert!(validate_derived_metric_id_str("error_reduction_proxy").is_ok());
    assert!(validate_derived_metric_id_str("nope").is_err());

    let invocation = ToolInvocationV1::new(ToolInvocationSpecV1 {
        schema_version: "bijux.tool_invocation.v1".to_string(),
        contract_version: ContractVersion::v1(),
        stage_id: StageId::new("fastq.trim_reads"),
        tool_id: ToolId::new("fastp"),
        tool_version: "0.23.4".to_string(),
        resolved_tool_version: Some("0.23.4+patched".to_string()),
        image_digest: "sha256:abc".to_string(),
        runner_kind: "container".to_string(),
        platform: "linux/arm64".to_string(),
        parameters_json: serde_json::json!({"k": "v"}),
        parameters_json_normalized: serde_json::json!({"k": "v"}),
        effective_params_json: serde_json::json!({"k": "v"}),
        effective_params_json_normalized: serde_json::json!({"k": "v"}),
        params_provenance: serde_json::json!({"src": "test"}),
        params_provenance_normalized: serde_json::json!({"src": "test"}),
        resources: ToolConstraints::default(),
        environment: BTreeMap::from([("RUST_LOG".to_string(), "info".to_string())]),
        input_hashes: vec!["in_hash".to_string()],
        output_hashes: vec!["out_hash".to_string()],
        executed_command: Some("fastp -i in -o out".to_string()),
    });

    assert_eq!(invocation.schema_version, "bijux.tool_invocation.v1");
    assert_eq!(invocation.stage_id.as_str(), "fastq.trim_reads");
    assert_eq!(invocation.tool_id.as_str(), "fastp");
    assert_eq!(invocation.runner_kind, "container");
    assert_eq!(invocation.platform, "linux/arm64");
    assert!(invocation.adapter_bank.is_none());
    assert!(invocation.banks.is_none());
    assert!(invocation.bank_assets.is_none());
    assert_eq!(invocation.input_hashes.len(), 1);
    assert_eq!(invocation.output_hashes.len(), 1);
}
