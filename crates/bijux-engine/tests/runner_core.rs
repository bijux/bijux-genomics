#![allow(clippy::expect_used)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_core::execution_plan::{ExecutionPlan, PlanEdge, PlanPolicy};
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, StageId, StagePlanV1, StageVersion, ToolConstraints, ToolId,
};
use bijux_engine::runner::{execute_plan, ExecutionOptions, Runner, StageExecution};

struct FakeRunner {
    calls: RefCell<Vec<String>>,
    fail_first: RefCell<Vec<String>>,
    cached: RefCell<Vec<String>>,
}

impl FakeRunner {
    fn new() -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
            fail_first: RefCell::new(Vec::new()),
            cached: RefCell::new(Vec::new()),
        }
    }
}

impl Runner for FakeRunner {
    fn is_cached(&self, plan: &StagePlanV1) -> bool {
        self.cached
            .borrow()
            .iter()
            .any(|id| id == plan.stage_id.0.as_str())
    }

    fn run(&self, plan: &StagePlanV1, attempt: u32) -> anyhow::Result<StageExecution> {
        self.calls
            .borrow_mut()
            .push(format!("{}:{}", plan.stage_id.0, attempt));
        let mut fail_first = self.fail_first.borrow_mut();
        let should_fail =
            fail_first.iter().any(|id| id == plan.stage_id.0.as_str()) && attempt == 0;
        if should_fail {
            fail_first.retain(|id| id != plan.stage_id.0.as_str());
        }
        Ok(StageExecution {
            stage_id: plan.stage_id.0.clone(),
            attempt,
            success: !should_fail,
            cached: false,
        })
    }
}

fn plan_for(stage_id: &str) -> StagePlanV1 {
    StagePlanV1 {
        stage_id: StageId(stage_id.to_string()),
        stage_version: StageVersion(1),
        tool_id: ToolId("tool".to_string()),
        tool_version: "0.0.0".to_string(),
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
        io: bijux_core::StageIO {
            inputs: vec![bijux_core::ArtifactRef {
                name: "input".to_string(),
                path: PathBuf::from("input"),
            }],
            outputs: vec![bijux_core::ArtifactRef {
                name: "output".to_string(),
                path: PathBuf::from("output"),
            }],
        },
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({"sample_id":"s1"}),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
    }
}

#[test]
fn execute_plan_orders_dag() {
    let stages = vec![plan_for("A"), plan_for("B"), plan_for("C")];
    let edges = vec![PlanEdge::new("A", "C"), PlanEdge::new("B", "C")];
    let plan = ExecutionPlan::new(
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
    let plan = ExecutionPlan::new(
        "pipeline",
        "planner",
        PlanPolicy::PreferAccuracy,
        stages,
        Vec::new(),
    )
    .expect("plan");
    let runner = FakeRunner::new();
    runner.fail_first.borrow_mut().push("A".to_string());
    let options = ExecutionOptions {
        retries: 1,
        resume: false,
    };
    let result = execute_plan(&plan, &runner, &options).expect("run");
    assert_eq!(result.stages[0].attempt, 1);
    assert!(result.stages[0].success);
}

#[test]
fn execute_plan_stops_on_failure() {
    let stages = vec![plan_for("A"), plan_for("B")];
    let plan = ExecutionPlan::new(
        "pipeline",
        "planner",
        PlanPolicy::PreferAccuracy,
        stages,
        vec![PlanEdge::new("A", "B")],
    )
    .expect("plan");
    let runner = FakeRunner::new();
    runner.fail_first.borrow_mut().push("A".to_string());
    let options = ExecutionOptions {
        retries: 0,
        resume: false,
    };
    let err = execute_plan(&plan, &runner, &options).expect_err("expected failure");
    assert!(err.to_string().contains("stage failed"));
    let calls = runner.calls.borrow().clone();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].starts_with("A:"));
}

#[test]
fn execute_plan_respects_resume_cache() {
    let stages = vec![plan_for("A"), plan_for("B")];
    let plan = ExecutionPlan::new(
        "pipeline",
        "planner",
        PlanPolicy::PreferAccuracy,
        stages,
        vec![PlanEdge::new("A", "B")],
    )
    .expect("plan");
    let runner = FakeRunner::new();
    runner.cached.borrow_mut().push("A".to_string());
    let options = ExecutionOptions {
        retries: 0,
        resume: true,
    };
    let result = execute_plan(&plan, &runner, &options).expect("run");
    assert!(result.stages[0].cached);
    let calls = runner.calls.borrow().clone();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].starts_with("B:"));
}
