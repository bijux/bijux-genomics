use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::run::{
    assess_failed_replay_eligibility, replay_failed_run, ExecuteRequest, RuntimeKind, execute,
};
use bijux_dna_core::contract::{ExecutionGraph, ExecutionStep, PlanPolicy};
use bijux_dna_core::ids::{ArtifactId, StageId, StepId};
use bijux_dna_core::prelude::{
    ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1, StageIO, ToolConstraints,
};

fn failing_graph(run_root: &std::path::Path, command: &str) -> Result<ExecutionGraph> {
    std::fs::create_dir_all(run_root)?;
    let reads = run_root.join("reads.fastq");
    let output = run_root.join("validated.fastq");
    std::fs::write(&reads, b"@read\nACGT\n+\n!!!!\n")?;
    let step = ExecutionStep {
        step_id: StepId::new("fastq.validate_reads"),
        stage_id: StageId::new("fastq.validate_reads"),
        command: CommandSpecV1 {
            template: vec!["sh".to_string(), "-c".to_string(), command.to_string()],
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
                output.clone(),
                ArtifactRole::Reads,
            )],
        },
        out_dir: run_root.join("out"),
        aux_images: BTreeMap::default(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    };
    Ok(ExecutionGraph::new(
        "fastq-to-fastq__failed_replay__v1",
        "api.replay.failed",
        PlanPolicy::PreferAccuracy,
        vec![step],
        Vec::new(),
    )?)
}

#[test]
fn failed_run_eligibility_marks_pending_stage_as_replayable() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = failing_graph(temp.path(), "exit 9")?;
    let response = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Local,
        run_dir: temp.path().join("run"),
        mode: bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced,
    })?;
    assert_eq!(response.state, bijux_dna_runtime::run_layout::RunLifecycleStateV1::Failed);
    let run_dir = response
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("manifest missing parent"))?;
    let eligibility = assess_failed_replay_eligibility(run_dir)?;
    assert_eq!(
        eligibility.get("failed").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        eligibility
            .get("unsafe_resume")
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert!(
        eligibility
            .get("eligible_stage_ids")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|entries| !entries.is_empty())
    );
    Ok(())
}

#[test]
fn failed_run_replay_replays_in_new_directory_without_overwriting_source() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = failing_graph(temp.path(), "exit 9")?;
    let response = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Local,
        run_dir: temp.path().join("run"),
        mode: bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced,
    })?;
    let run_dir = response
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("manifest missing parent"))?;

    let graph_path = run_dir.join("manifests").join("graph.json");
    let replay_input = run_dir.join("reads.fastq");
    std::fs::write(&replay_input, b"@read\nACGT\n+\n!!!!\n")?;
    let replay_output = run_dir.join("validated.fastq");
    let mut graph_json: serde_json::Value = serde_json::from_slice(&std::fs::read(&graph_path)?)?;
    graph_json["steps"][0]["io"]["inputs"][0]["path"] =
        serde_json::Value::String(replay_input.display().to_string());
    graph_json["steps"][0]["io"]["outputs"][0]["path"] =
        serde_json::Value::String(replay_output.display().to_string());
    graph_json["steps"][0]["command"]["template"] = serde_json::json!([
        "sh",
        "-c",
        format!("printf 'ACGT\\n' > {}", replay_output.display())
    ]);
    bijux_dna_infra::atomic_write_json(&graph_path, &graph_json)?;

    let replay = replay_failed_run(run_dir)?;
    let replay_dir = replay
        .get("replay_run_dir")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("missing replay_run_dir"))?;
    assert!(std::path::Path::new(replay_dir).join("run_manifest.json").exists());
    assert!(run_dir.join("run_manifest.json").exists());
    Ok(())
}

#[test]
fn failed_run_replay_refuses_unsafe_resume_classes() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = failing_graph(temp.path(), "exit 9")?;
    let response = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Local,
        run_dir: temp.path().join("run"),
        mode: bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced,
    })?;
    let run_dir = response
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("manifest missing parent"))?;
    let failure_path = run_dir.join("run_failure.json");
    let mut failure: serde_json::Value = serde_json::from_slice(&std::fs::read(&failure_path)?)?;
    failure["failure_code"] = serde_json::Value::String("invariant_violation".to_string());
    bijux_dna_infra::atomic_write_json(&failure_path, &failure)?;

    let err = replay_failed_run(run_dir).err().ok_or_else(|| anyhow!("unsafe replay must fail"))?;
    assert!(err.to_string().contains("unsafe resume refused"));
    Ok(())
}
