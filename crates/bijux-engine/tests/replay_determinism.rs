use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use bijux_core::contract::{ArtifactRef, ArtifactRole, StageIO, ToolConstraints};
use bijux_core::execution::execution_graph::ExecutionGraph;
use bijux_core::execution::PlanPolicy;
use bijux_core::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};
use bijux_engine::Engine;
use bijux_runtime::{Invocation, Runner, RunnerResult};
use walkdir::WalkDir;

struct DeterministicRunner;

impl Runner for DeterministicRunner {
    fn run(&self, invocation: &Invocation) -> Result<RunnerResult> {
        let step = &invocation.step;
        let run_artifacts = step.out_dir.join("run_artifacts");
        bijux_infra::ensure_dir(&run_artifacts)?;
        for name in [
            "metrics.json",
            "effective_config.json",
            "stage_report.json",
            "tool_invocation.json",
        ] {
            let path = run_artifacts.join(name);
            bijux_infra::write_bytes(&path, "{}")?;
        }
        for output in &step.io.outputs {
            bijux_infra::ensure_dir(
                output
                    .path
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("output missing parent"))?,
            )?;
            bijux_infra::write_bytes(&output.path, "deterministic")?;
        }
        Ok(RunnerResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::from_millis(1),
            artifacts: Vec::new(),
        })
    }
}

#[test]
fn replay_produces_same_run_record_and_tree() -> Result<()> {
    let temp = bijux_infra::temp_dir("bijux-engine-replay")?;
    let base = temp.path();
    let (run_id, layout) = bijux_runtime::run_layout::create_run_layout(base)?;
    let out_dir = layout.stages_dir.join("stage_1");
    bijux_infra::ensure_dir(&out_dir)?;
    let input_path = out_dir.join("input.txt");
    bijux_infra::write_bytes(&input_path, "input")?;
    let output_path = out_dir.join("output.txt");

    let step = bijux_core::execution::execution_graph::ExecutionStep {
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
    let record_first = Engine::execute(&graph, &runner, &layout, None, None)?;
    let tree_first = hash_tree(&layout.run_dir)?;

    std::fs::remove_file(&output_path)?;
    let record_second = Engine::execute(&graph, &runner, &layout, None, None)?;
    let tree_second = hash_tree(&layout.run_dir)?;

    assert_eq!(
        serde_json::to_value(record_first)?,
        serde_json::to_value(record_second)?
    );
    assert_eq!(tree_first, tree_second);
    assert!(!run_id.is_empty());
    Ok(())
}

fn hash_tree(root: &Path) -> Result<BTreeMap<String, String>> {
    let mut hashes = BTreeMap::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(path);
        if path.file_name().and_then(|name| name.to_str()) == Some("execution_record.json") {
            continue;
        }
        let hash = bijux_infra::hash_file_sha256(path)?;
        hashes.insert(rel.display().to_string(), hash);
    }
    Ok(hashes)
}
