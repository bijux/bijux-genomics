use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::run::{
    dry_run, execute, replay_manifest, DryRunRequest, ExecuteRequest, RuntimeKind,
};
use bijux_dna_core::contract::{ExecutionGraph, ExecutionStep, PlanPolicy};
use bijux_dna_core::ids::{ArtifactId, StageId, StepId};
use bijux_dna_core::prelude::{
    ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1, StageIO, ToolConstraints,
};

fn minimal_graph(run_dir: &Path) -> Result<ExecutionGraph> {
    let step = ExecutionStep {
        step_id: StepId::new("fastq.validate_reads"),
        stage_id: StageId::new("fastq.validate_reads"),
        command: CommandSpecV1 { template: vec!["echo".to_string(), "hello".to_string()] },
        image: ContainerImageRefV1 {
            image: "example/validator:1".to_string(),
            digest: Some("sha256:deadbeef".to_string()),
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactSpec::required(
                ArtifactId::new("reads"),
                PathBuf::from("reads.fastq"),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactSpec::required(
                ArtifactId::new("validated"),
                PathBuf::from("validated.fastq"),
                ArtifactRole::Reads,
            )],
        },
        out_dir: run_dir.join("out"),
        aux_images: BTreeMap::default(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    };
    Ok(ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "test-planner",
        PlanPolicy::PreferAccuracy,
        vec![step],
        Vec::new(),
    )?)
}

fn docker_contracts_enabled() -> bool {
    matches!(std::env::var("BIJUX_DNA_DOCKER_CONTRACTS").as_deref(), Ok("1" | "true" | "yes"))
}

#[test]
fn dry_run_emits_manifest_and_graph_without_execution() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = minimal_graph(temp.path())?;
    let request = DryRunRequest {
        graph,
        run_dir: temp.path().to_path_buf(),
        profile_id: "fastq-to-fastq__default__v1".to_string(),
    };
    let response = dry_run(&request)?;
    assert!(response.graph_path.exists());
    assert!(response.manifest_path.exists());
    assert!(response.run_summary_path.exists());
    assert!(response.evidence_bundle_path.exists());
    assert!(response.correlation_id.starts_with("dry-run:"));
    assert!(temp.path().join("run_summary.json").exists());
    Ok(())
}

#[test]
fn dry_run_creates_missing_run_dir() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let run_dir = temp.path().join("runs").join("dry-run");
    let graph = minimal_graph(&run_dir)?;
    let request = DryRunRequest {
        graph,
        run_dir: run_dir.clone(),
        profile_id: "fastq-to-fastq__default__v1".to_string(),
    };

    let response = dry_run(&request)?;

    assert!(run_dir.is_dir());
    assert!(response.graph_path.exists());
    assert!(response.manifest_path.exists());
    Ok(())
}

#[test]
fn dry_run_manifest_verifies_output_artifacts() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = minimal_graph(temp.path())?;
    let request = DryRunRequest {
        graph,
        run_dir: temp.path().to_path_buf(),
        profile_id: "fastq-to-fastq__default__v1".to_string(),
    };
    let response = dry_run(&request)?;

    replay_manifest(&response.manifest_path, true)
}

#[test]
fn dry_run_manifest_records_planned_stages() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = minimal_graph(temp.path())?;
    let request = DryRunRequest {
        graph,
        run_dir: temp.path().to_path_buf(),
        profile_id: "fastq-to-fastq__default__v1".to_string(),
    };
    let response = dry_run(&request)?;
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(response.manifest_path)?)?;

    assert_eq!(manifest["stages"][0]["stage_id"], "fastq.validate_reads");
    assert_eq!(manifest["stages"][0]["image_digest"], "sha256:deadbeef");
    Ok(())
}

#[test]
fn execute_emits_run_summary_artifact() -> Result<()> {
    if !docker_contracts_enabled() {
        return Ok(());
    }

    let temp = tempfile::tempdir()?;
    let graph = minimal_graph(temp.path())?;
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
        command: CommandSpecV1 { template: vec!["echo".to_string(), "hello".to_string()] },
        image: ContainerImageRefV1 { image: "alpine:3.20".to_string(), digest: None },
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
        aux_images: BTreeMap::default(),
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
    let Err(err) = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Docker,
        run_dir: temp.path().join("run"),
    }) else {
        panic!("unknown stage prefix must fail before execution");
    };
    assert!(err.to_string().contains("no stage-runner contract"), "unexpected error: {err}");
}
