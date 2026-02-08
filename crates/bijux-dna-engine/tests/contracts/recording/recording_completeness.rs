use std::collections::BTreeMap;

use anyhow::Result;
use bijux_dna_core::contract::{
    ArtifactRef, ArtifactRole, ExecutionGraph, PlanPolicy, StageIO, ToolConstraints,
};
use bijux_dna_core::prelude::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};
use bijux_dna_engine::Engine;

use crate::support::{execution_setup, RecordingRunner};

#[test]
fn step_emits_truth_set() -> Result<()> {
    let (_temp, layout) = execution_setup()?;
    let out_dir = layout.stages_dir.join("stage_1");
    bijux_dna_infra::ensure_dir(&out_dir)?;
    let input_path = out_dir.join("input.txt");
    bijux_dna_infra::write_bytes(&input_path, "input")?;
    let output_path = out_dir.join("output.txt");

    let step = bijux_dna_core::contract::ExecutionStep {
        step_id: StepId::new("fastq.trim"),
        stage_id: StageId::new("fastq.trim"),
        image: ContainerImageRefV1 {
            image: "tool".to_string(),
            digest: Some("sha256:img".to_string()),
        },
        command: CommandSpecV1 {
            template: vec!["tool".to_string()],
        },
        resources: ToolConstraints {
            runtime: "short".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("input"),
                input_path.clone(),
                ArtifactRole::Unknown,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::from_static("output"),
                output_path.clone(),
                ArtifactRole::Unknown,
            )],
        },
        out_dir: out_dir.clone(),
        aux_images: BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    };
    let graph = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner",
        PlanPolicy::PreferAccuracy,
        vec![step],
        Vec::new(),
    )?;

    Engine::default().execute(&graph, &RecordingRunner, &layout, None, None)?;

    let run_artifacts = out_dir.join("run_artifacts");
    for name in [
        "metrics.json",
        "effective_config.json",
        "stage_report.json",
        "tool_invocation.json",
        "execution_record.json",
    ] {
        assert!(run_artifacts.join(name).exists(), "missing {name}");
    }
    Ok(())
}
