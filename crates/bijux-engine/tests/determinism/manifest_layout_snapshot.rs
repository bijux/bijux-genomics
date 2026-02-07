use std::collections::BTreeMap;

use anyhow::Result;
use bijux_core::contract::{ArtifactRef, ArtifactRole, ExecutionGraph, PlanPolicy, StageIO, ToolConstraints};
use bijux_core::prelude::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};
use bijux_engine::Engine;

use crate::support::{execution_setup, layout_tree_text, manifest_hash, DeterministicRunner};

#[test]
fn determinism_manifest_hash_and_layout_tree_snapshot() -> Result<()> {
    let (_temp, layout) = execution_setup()?;
    let out_dir = layout.stages_dir.join("stage_1");
    bijux_infra::ensure_dir(&out_dir)?;
    let input_path = out_dir.join("input.txt");
    bijux_infra::write_bytes(&input_path, "input")?;
    let output_path = out_dir.join("output.txt");

    let step = bijux_core::contract::ExecutionStep {
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

    let runner = DeterministicRunner;
    Engine::default().execute(&graph, &runner, &layout, None, None)?;
    let manifest_first = manifest_hash(&layout)?;
    let tree_first = layout_tree_text(&layout.run_dir)?;

    std::fs::remove_file(&output_path)?;
    Engine::default().execute(&graph, &runner, &layout, None, None)?;
    let manifest_second = manifest_hash(&layout)?;
    let tree_second = layout_tree_text(&layout.run_dir)?;

    assert_eq!(manifest_first, manifest_second);
    assert_eq!(tree_first, tree_second);
    Ok(())
}
