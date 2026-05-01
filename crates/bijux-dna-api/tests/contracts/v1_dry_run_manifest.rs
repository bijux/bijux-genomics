use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::run::{
    cancel_run, dry_run, execute, operator_health, pause_run, replay_manifest, resume_run,
    DryRunRequest, ExecuteRequest, RuntimeKind,
};
use bijux_dna_core::contract::{ExecutionGraph, ExecutionStep, PlanPolicy};
use bijux_dna_core::ids::{ArtifactId, StageId, StepId};
use bijux_dna_core::prelude::{
    ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1, StageIO, ToolConstraints,
};

fn minimal_graph(run_dir: &Path) -> Result<ExecutionGraph> {
    std::fs::create_dir_all(run_dir)?;
    let reads = run_dir.join("reads.fastq");
    let validated = run_dir.join("validated.fastq");
    std::fs::write(&reads, b"@read\nACGT\n+\n!!!!\n")?;
    let step = ExecutionStep {
        step_id: StepId::new("fastq.validate_reads"),
        stage_id: StageId::new("fastq.validate_reads"),
        command: CommandSpecV1 {
            template: vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("printf 'ACGT\\n' > {}", validated.display()),
            ],
        },
        image: ContainerImageRefV1 {
            image: "example/validator:1".to_string(),
            digest: Some("sha256:deadbeef".to_string()),
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactSpec::required(
                ArtifactId::new("reads"),
                reads,
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactSpec::required(
                ArtifactId::new("validated"),
                validated,
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
    assert!(response.run_summary_text_path.exists());
    assert!(response.backend_descriptor_path.exists());
    assert!(response.scheduling_decision_path.exists());
    assert!(response.queue_state_path.exists());
    assert!(response.lease_path.exists());
    assert!(response.control_state_path.exists());
    assert!(response.health_report_path.exists());
    assert!(response.evidence_bundle_path.exists());
    assert!(response.evidence_verification_path.exists());
    assert!(response.artifact_inventory_path.exists());
    assert!(response.replay_manifest_path.exists());
    assert!(response.hash_ledger_path.exists());
    assert!(response.correlation_id.starts_with("dry_run:"));
    assert!(temp.path().join("summary").join("run_summary.json").exists());
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
    let temp = tempfile::tempdir()?;
    let graph = minimal_graph(temp.path())?;
    let response = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Local,
        run_dir: temp.path().to_path_buf(),
        mode: bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced,
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
fn execute_simulation_writes_governed_runtime_contracts_without_process_execution() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = minimal_graph(temp.path())?;
    let response = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Local,
        run_dir: temp.path().to_path_buf(),
        mode: bijux_dna_runtime::run_layout::RunExecutionModeV1::Simulation,
    })?;

    assert_eq!(response.mode, bijux_dna_runtime::run_layout::RunExecutionModeV1::Simulation);
    assert_eq!(response.state, bijux_dna_runtime::run_layout::RunLifecycleStateV1::Succeeded);
    assert!(response.run_state_path.exists());
    assert!(response.runtime_policy_path.exists());
    assert!(response.executor_descriptor_path.exists());
    assert!(response.backend_descriptor_path.exists());
    assert!(response.scheduling_decision_path.exists());
    assert!(response.queue_state_path.exists());
    assert!(response.lease_path.exists());
    assert!(response.control_state_path.exists());
    assert!(response.health_report_path.exists());
    assert!(response.checkpoint_path.exists());
    assert!(response.artifact_inventory_path.exists());
    assert!(response.hash_ledger_path.exists());
    assert!(response.failure_path.is_none());
    Ok(())
}

#[test]
fn execute_failure_writes_inspectable_failure_record() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let reads = temp.path().join("reads.fastq");
    std::fs::write(&reads, b"@read\nACGT\n+\n!!!!\n")?;
    let step = ExecutionStep {
        step_id: StepId::new("fastq.validate_reads"),
        stage_id: StageId::new("fastq.validate_reads"),
        command: CommandSpecV1 {
            template: vec!["sh".to_string(), "-c".to_string(), "exit 9".to_string()],
        },
        image: ContainerImageRefV1 {
            image: "example/validator:1".to_string(),
            digest: Some("sha256:deadbeef".to_string()),
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactSpec::required(
                ArtifactId::new("reads"),
                reads,
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactSpec::required(
                ArtifactId::new("validated"),
                temp.path().join("validated.fastq"),
                ArtifactRole::Reads,
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
    )?;

    let response = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Local,
        run_dir: temp.path().to_path_buf(),
        mode: bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced,
    })?;

    assert_eq!(response.state, bijux_dna_runtime::run_layout::RunLifecycleStateV1::Failed);
    let failure_path = response
        .failure_path
        .ok_or_else(|| anyhow!("failure response must include failure path"))?;
    let failure: serde_json::Value = serde_json::from_slice(&std::fs::read(&failure_path)?)?;
    assert_eq!(failure["schema_version"], "bijux.run_failure.v1");
    assert_eq!(failure["state"], "failed");
    Ok(())
}

#[test]
fn replay_manifest_reuses_local_runner_descriptor() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = minimal_graph(temp.path())?;
    let replay_graph = graph.clone();
    let response = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Local,
        run_dir: temp.path().to_path_buf(),
        mode: bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced,
    })?;
    let run_dir = response
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("manifest path missing parent directory"))?
        .to_path_buf();
    let graph_payload =
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&replay_graph)?;
    bijux_dna_infra::atomic_write_bytes(
        &run_dir.join("manifests/graph.json"),
        graph_payload.as_slice(),
    )?;
    std::fs::remove_file(temp.path().join("validated.fastq"))?;
    let previous_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp.path())?;

    let replay_result = replay_manifest(&response.manifest_path, false);
    std::env::set_current_dir(previous_dir)?;
    replay_result?;

    assert!(temp.path().join("validated.fastq").exists());
    assert!(run_dir.join("executor_descriptor.json").exists());
    Ok(())
}

#[test]
fn run_control_commands_write_auditable_control_state() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = minimal_graph(temp.path())?;
    let response = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Local,
        run_dir: temp.path().to_path_buf(),
        mode: bijux_dna_runtime::run_layout::RunExecutionModeV1::Simulation,
    })?;
    let run_dir = response
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("manifest path missing parent directory"))?;

    let paused = pause_run(run_dir)?;
    assert_eq!(
        paused.state.requested_action,
        Some(bijux_dna_runtime::run_layout::RunControlActionV1::Pause)
    );
    let resumed = resume_run(run_dir)?;
    assert_eq!(
        resumed.state.requested_action,
        Some(bijux_dna_runtime::run_layout::RunControlActionV1::Resume)
    );
    let cancelled = cancel_run(run_dir)?;
    assert_eq!(
        cancelled.state.requested_action,
        Some(bijux_dna_runtime::run_layout::RunControlActionV1::Cancel)
    );
    assert!(cancelled.control_state_path.exists());
    Ok(())
}

#[test]
fn operator_health_rewrites_report_from_executor_descriptor() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = minimal_graph(temp.path())?;
    let response = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Local,
        run_dir: temp.path().to_path_buf(),
        mode: bijux_dna_runtime::run_layout::RunExecutionModeV1::Simulation,
    })?;
    let run_dir = response
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("manifest path missing parent directory"))?;

    let health = operator_health(run_dir)?;
    assert!(health.health_report_path.exists());
    assert!(health.report.checks.iter().any(|check| check.check_id == "storage"));
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
        mode: bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced,
    }) else {
        panic!("unknown stage prefix must fail before execution");
    };
    assert!(err.to_string().contains("no stage-runner contract"), "unexpected error: {err}");
}
