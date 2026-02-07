//! Shared test helpers for bijux-engine.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::Result;
use bijux_core::contract::{
    ArtifactRef, ArtifactRole, ExecutionEdge, ExecutionGraph, ExecutionStep, PlanPolicy, RetryPolicy,
    StageIO, ToolConstraints,
};
use bijux_core::prelude::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};
use bijux_engine::Engine;
use bijux_runtime::run_layout::create_run_layout;
use bijux_runtime::{Invocation, Runner, RunnerResult};

#[derive(Default)]
pub struct FakeRunner {
    calls: RefCell<Vec<String>>,
    fail_first: RefCell<BTreeSet<String>>,
}

impl FakeRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fail_first(&self, step_id: &str) {
        self.fail_first.borrow_mut().insert(step_id.to_string());
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl Runner for FakeRunner {
    fn run(&self, invocation: &Invocation) -> Result<RunnerResult> {
        let plan = &invocation.step;
        let attempt = invocation.attempt;
        self.calls
            .borrow_mut()
            .push(format!("{}:{}", plan.step_id.0, attempt));
        let should_fail = self
            .fail_first
            .borrow_mut()
            .take(&plan.step_id.0)
            .is_some()
            && attempt == 0;
        let run_artifacts = plan.out_dir.join("run_artifacts");
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
        Ok(RunnerResult {
            exit_code: i32::from(should_fail),
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::from_millis(1),
            artifacts: Vec::new(),
        })
    }
}

pub struct DeterministicRunner;

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

pub struct RecordingRunner;

impl Runner for RecordingRunner {
    fn run(&self, invocation: &Invocation) -> Result<RunnerResult> {
        let step = &invocation.step;
        let run_artifacts = step.out_dir.join("run_artifacts");
        bijux_infra::ensure_dir(&run_artifacts)?;
        for name in [
            "metrics.json",
            "effective_config.json",
            "stage_report.json",
            "tool_invocation.json",
            "execution_record.json",
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
            bijux_infra::write_bytes(&output.path, "data")?;
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

pub fn plan_for(stage_id: &str) -> ExecutionStep {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    let suffix = COUNTER.fetch_add(1, Ordering::Relaxed);
    let out_dir = std::env::temp_dir().join(format!("bijux-engine-test-{stage_id}-{suffix}"));
    ExecutionStep {
        step_id: StepId::new(stage_id),
        stage_id: StageId::new(stage_id),
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
                PathBuf::from("input"),
                ArtifactRole::Unknown,
            )],
            outputs: vec![ArtifactRef::optional(
                ArtifactId::from_static("output"),
                PathBuf::from("output"),
                ArtifactRole::Unknown,
            )],
        },
        out_dir,
        aux_images: BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    }
}

pub fn build_graph(stages: Vec<ExecutionStep>, edges: Vec<ExecutionEdge>) -> ExecutionGraph {
    ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner",
        PlanPolicy::PreferAccuracy,
        stages,
        edges,
    )
    .expect("plan")
}

pub fn run_with_layout(graph: &ExecutionGraph, runner: &dyn Runner) -> Result<bijux_core::contract::RunRecordV1> {
    let dir = tempfile::tempdir().expect("tempdir");
    let (_run_id, layout) = create_run_layout(dir.path()).expect("layout");
    Engine::default().execute(graph, runner, &layout, None, None)
}

pub fn layout_tree_text(root: &Path) -> Result<String> {
    let mut entries = Vec::new();
    for entry in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(path);
        let hash = bijux_infra::hash_file_sha256(path)?;
        entries.push(format!("{}\t{}", rel.display(), hash));
    }
    entries.sort();
    Ok(entries.join("\n"))
}

pub fn manifest_hash(layout: &bijux_runtime::run_layout::RunLayout) -> Result<String> {
    let raw = std::fs::read_to_string(&layout.manifest_path)?;
    let manifest: bijux_runtime::run_layout::RunManifest = serde_json::from_str(&raw)?;
    Ok(manifest.hash()?)
}

pub fn execution_setup() -> Result<(tempfile::TempDir, bijux_runtime::run_layout::RunLayout)> {
    let temp = bijux_infra::temp_dir("bijux-engine-test")?;
    let (_run_id, layout) = bijux_runtime::run_layout::create_run_layout(temp.path())?;
    Ok((temp, layout))
}
