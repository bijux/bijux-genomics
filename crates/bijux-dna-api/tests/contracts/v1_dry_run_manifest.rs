use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::run::{dry_run, execute, DryRunRequest, ExecuteRequest, RuntimeKind};
use bijux_dna_core::contract::{ExecutionGraph, ExecutionStep, PlanPolicy};
use bijux_dna_core::ids::{ArtifactId, StageId, StepId};
use bijux_dna_core::prelude::{
    ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1, StageIO, ToolConstraints,
};

#[test]
fn dry_run_emits_manifest_and_graph_without_execution() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "test-planner",
        PlanPolicy::PreferAccuracy,
        Vec::new(),
        Vec::new(),
    )?;
    let request = DryRunRequest {
        graph,
        run_dir: temp.path().to_path_buf(),
        profile_id: "fastq-to-fastq__default__v1".to_string(),
    };
    let response = dry_run(&request)?;
    assert!(response.graph_path.exists());
    assert!(response.manifest_path.exists());
    assert!(temp.path().join("run_summary.json").exists());
    Ok(())
}

#[test]
fn execute_emits_run_summary_artifact() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "test-planner",
        PlanPolicy::PreferAccuracy,
        Vec::new(),
        Vec::new(),
    )?;
    let response = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Docker,
        run_dir: temp.path().to_path_buf(),
    })?;
    let run_dir = response
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("manifest path missing parent directory"))?
        .to_path_buf();
    assert!(run_dir.join("summary").join("run_summary.json").exists());
    Ok(())
}

#[test]
fn execute_fails_fast_when_runner_contract_missing_for_stage() {
    let temp = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let step = ExecutionStep {
        step_id: StepId::new("unknown.stage"),
        stage_id: StageId::new("unknown.stage"),
        command: CommandSpecV1 {
            template: vec!["echo".to_string(), "hello".to_string()],
        },
        image: ContainerImageRefV1 {
            image: "alpine:3.20".to_string(),
            digest: None,
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactSpec::required(
                ArtifactId::new("unknown.input"),
                temp.path().join("input.txt"),
                ArtifactRole::Unknown,
            )],
            outputs: vec![ArtifactSpec::required(
                ArtifactId::new("unknown.output"),
                temp.path().join("output.txt"),
                ArtifactRole::Unknown,
            )],
        },
        out_dir: temp.path().join("out"),
        aux_images: Default::default(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    };
    let graph = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "test-planner",
        PlanPolicy::PreferAccuracy,
        vec![step],
        Vec::new(),
    )
    .unwrap_or_else(|err| panic!("build graph: {err}"));
    let err = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Docker,
        run_dir: temp.path().join("run"),
    })
    .expect_err("unknown stage prefix must fail before execution");
    assert!(
        err.to_string().contains("no stage-runner contract"),
        "unexpected error: {err}"
    );
}
