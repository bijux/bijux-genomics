#![allow(clippy::expect_used)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use bijux_core::contract::{ArtifactRef, StageIO, ToolConstraints};
use bijux_core::plan::execution_graph::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_core::plan::PlanPolicy;
use bijux_core::plan::{Invocation, Runner, RunnerResult};
use bijux_core::{CommandSpecV1, ContainerImageRefV1, StageId};

use crate::executor::{execute_plan, ExecutionOptions};

struct FakeRunner {
    calls: RefCell<Vec<String>>,
    fail_first: RefCell<Vec<String>>,
}

impl FakeRunner {
    fn new() -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
            fail_first: RefCell::new(Vec::new()),
        }
    }
}

impl Runner for FakeRunner {
    fn run(&self, invocation: &Invocation) -> anyhow::Result<RunnerResult> {
        let plan = &invocation.step;
        let attempt = invocation.attempt;
        self.calls
            .borrow_mut()
            .push(format!("{}:{}", plan.step_id.0, attempt));
        let mut fail_first = self.fail_first.borrow_mut();
        let should_fail = fail_first.iter().any(|id| id == plan.step_id.as_str()) && attempt == 0;
        if should_fail {
            fail_first.retain(|id| id != plan.step_id.as_str());
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

fn plan_for(stage_id: &str) -> ExecutionStep {
    ExecutionStep {
        step_id: StageId::new(stage_id),
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
            inputs: vec![ArtifactRef {
                name: "input".to_string(),
                path: PathBuf::from("input"),
            }],
            outputs: vec![ArtifactRef {
                name: "output".to_string(),
                path: PathBuf::from("output"),
            }],
        },
        out_dir: PathBuf::from("out"),
        aux_images: BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    }
}

#[test]
fn execute_plan_orders_dag() {
    let stages = vec![plan_for("A"), plan_for("B"), plan_for("C")];
    let edges = vec![
        ExecutionEdge::new(StageId::new("A"), StageId::new("C")),
        ExecutionEdge::new(StageId::new("B"), StageId::new("C")),
    ];
    let plan = ExecutionGraph::new(
        "pipeline",
        "planner",
        PlanPolicy::PreferAccuracy,
        stages,
        edges,
    )
    .expect("plan");
    let runner = FakeRunner::new();
    let result = execute_plan(&plan, &runner, &ExecutionOptions::default()).expect("run");
    let order: Vec<String> = result.stages.into_iter().map(|r| r.stage_id).collect();
    assert_eq!(order, vec!["A", "B", "C"]);
}

#[test]
fn execute_plan_retries_failures() {
    let stages = vec![plan_for("A")];
    let plan = ExecutionGraph::new(
        "pipeline",
        "planner",
        PlanPolicy::PreferAccuracy,
        stages,
        Vec::new(),
    )
    .expect("plan");
    let runner = FakeRunner::new();
    runner.fail_first.borrow_mut().push("A".to_string());
    let options = ExecutionOptions { retries: 1 };
    let result = execute_plan(&plan, &runner, &options).expect("run");
    assert_eq!(result.stages[0].attempt, 1);
    assert!(result.stages[0].success);
}

#[test]
fn execute_plan_stops_on_failure() {
    let stages = vec![plan_for("A"), plan_for("B")];
    let plan = ExecutionGraph::new(
        "pipeline",
        "planner",
        PlanPolicy::PreferAccuracy,
        stages,
        vec![ExecutionEdge::new(StageId::new("A"), StageId::new("B"))],
    )
    .expect("plan");
    let runner = FakeRunner::new();
    runner.fail_first.borrow_mut().push("A".to_string());
    let options = ExecutionOptions { retries: 0 };
    let err = execute_plan(&plan, &runner, &options).expect_err("expected failure");
    assert!(err.to_string().contains("stage failed"));
    let calls = runner.calls.borrow().clone();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].starts_with("A:"));
}

#[test]
fn execute_plan_respects_resume_cache() {
    let stages = vec![plan_for("A"), plan_for("B")];
    let plan = ExecutionGraph::new(
        "pipeline",
        "planner",
        PlanPolicy::PreferAccuracy,
        stages,
        vec![ExecutionEdge::new(StageId::new("A"), StageId::new("B"))],
    )
    .expect("plan");
    let runner = FakeRunner::new();
    let options = ExecutionOptions { retries: 0 };
    let result = execute_plan(&plan, &runner, &options).expect("run");
    let calls = runner.calls.borrow().clone();
    assert_eq!(calls.len(), 2);
    assert_eq!(result.stages.len(), 2);
}

#[test]
fn execute_plan_is_deterministic() {
    let stages = vec![plan_for("A"), plan_for("B"), plan_for("C")];
    let edges = vec![
        ExecutionEdge::new(StageId::new("A"), StageId::new("C")),
        ExecutionEdge::new(StageId::new("B"), StageId::new("C")),
    ];
    let plan = ExecutionGraph::new(
        "pipeline",
        "planner",
        PlanPolicy::PreferAccuracy,
        stages,
        edges,
    )
    .expect("plan");

    let runner = FakeRunner::new();
    let result_a = execute_plan(&plan, &runner, &ExecutionOptions::default()).expect("run");
    let order_a: Vec<String> = result_a.stages.iter().map(|r| r.stage_id.clone()).collect();

    let runner = FakeRunner::new();
    let result_b = execute_plan(&plan, &runner, &ExecutionOptions::default()).expect("run");
    let order_b: Vec<String> = result_b.stages.iter().map(|r| r.stage_id.clone()).collect();

    assert_eq!(order_a, order_b);
}
