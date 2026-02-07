use bijux_core::contract::{ExecutionEdge, RetryPolicy};
use bijux_core::prelude::StepId;
use bijux_engine::Engine;

use crate::support::{build_graph, execution_setup, plan_for, FakeRunner};

#[test]
fn execute_plan_orders_dag() {
    let stages = vec![plan_for("A"), plan_for("B"), plan_for("C")];
    let edges = vec![
        ExecutionEdge::new(StepId::new("A"), StepId::new("C")),
        ExecutionEdge::new(StepId::new("B"), StepId::new("C")),
    ];
    let plan = build_graph(stages, edges);
    let runner = FakeRunner::new();
    let (_dir, layout) = execution_setup().expect("layout");
    let result = Engine::default()
        .execute(&plan, &runner, &layout, None, None)
        .expect("run");
    let order: Vec<String> = result.stages.into_iter().map(|r| r.stage_id).collect();
    assert_eq!(order, vec!["A", "B", "C"]);
}

#[test]
fn execute_plan_retries_failures() {
    let stages = vec![plan_for("A")];
    let plan = build_graph(stages, Vec::new());
    let runner = FakeRunner::new();
    runner.fail_first("A");
    let plan = plan.with_retry_policy(RetryPolicy {
        max_attempts: 2,
        retry_on_exit_codes: vec![1],
    });
    let (_dir, layout) = execution_setup().expect("layout");
    let result = Engine::default()
        .execute(&plan, &runner, &layout, None, None)
        .expect("run");
    assert_eq!(result.stages[0].attempt, 1);
    assert!(result.stages[0].success);
}

#[test]
fn execute_plan_stops_on_failure() {
    let stages = vec![plan_for("A"), plan_for("B")];
    let plan = build_graph(
        stages,
        vec![ExecutionEdge::new(StepId::new("A"), StepId::new("B"))],
    );
    let runner = FakeRunner::new();
    runner.fail_first("A");
    let plan = plan.with_retry_policy(RetryPolicy {
        max_attempts: 1,
        retry_on_exit_codes: vec![1],
    });
    let (_dir, layout) = execution_setup().expect("layout");
    let err = Engine::default()
        .execute(&plan, &runner, &layout, None, None)
        .expect_err("expected failure");
    assert!(err.to_string().contains("step failed"));
    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].starts_with("A:"));
}

#[test]
fn execute_plan_respects_resume_cache() {
    let stages = vec![plan_for("A"), plan_for("B")];
    let plan = build_graph(
        stages,
        vec![ExecutionEdge::new(StepId::new("A"), StepId::new("B"))],
    );
    let runner = FakeRunner::new();
    let (_dir, layout) = execution_setup().expect("layout");
    let result = Engine::default()
        .execute(&plan, &runner, &layout, None, None)
        .expect("run");
    assert_eq!(result.stages.len(), 2);
}
