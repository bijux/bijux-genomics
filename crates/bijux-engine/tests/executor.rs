use std::collections::HashMap;

use anyhow::Result;
use bijux_engine::core::types::MetricSet;
use bijux_engine::core::types::{RunPlan, StageRequirement, ToolInvocation};
use bijux_engine::core::validator::validate_stage;
use bijux_engine::services::executor::{execute_with_runner, ToolRunner};

struct FakeRunner {
    exit_codes: HashMap<String, i32>,
}

impl ToolRunner for FakeRunner {
    fn run(&self, plan: &RunPlan) -> anyhow::Result<bijux_engine::core::types::StageResult> {
        let exit = *self.exit_codes.get(&plan.invocation.stage_id).unwrap_or(&0);
        Ok(bijux_engine::core::types::StageResult {
            invocation: plan.invocation.clone(),
            exit_code: exit,
            stdout: String::new(),
            stderr: String::new(),
            outputs: Vec::new(),
        })
    }
}

fn plan_for_stage(stage: &str) -> RunPlan {
    RunPlan {
        invocation: ToolInvocation {
            stage_id: stage.to_string(),
            tool_id: "tool".to_string(),
            inputs: Vec::new(),
            params: serde_json::json!({}),
            requirements: Some(StageRequirement {
                capabilities: Vec::new(),
            }),
        },
        image_digest: "sha256:abc".to_string(),
        runner: bijux_environment::api::RunnerKind::Docker,
    }
}

#[test]
fn executor_runs_single_stage_successfully() -> Result<()> {
    let runner = FakeRunner {
        exit_codes: HashMap::new(),
    };
    let plan = plan_for_stage("fastq.trim");
    let result = execute_with_runner(&plan, &runner)?;
    let metrics = MetricSet {
        metrics_schema: "engine.metric.v1".to_string(),
        version: 1,
        metrics: serde_json::json!({}),
    };
    let validated = validate_stage(result, metrics)?;
    assert_eq!(validated.result.exit_code, 0);
    Ok(())
}

#[test]
fn executor_propagates_tool_failure() -> Result<()> {
    let mut exit_codes = HashMap::new();
    exit_codes.insert("fastq.trim".to_string(), 42);
    let runner = FakeRunner { exit_codes };
    let plan = plan_for_stage("fastq.trim");
    let result = execute_with_runner(&plan, &runner)?;
    let metrics = MetricSet {
        metrics_schema: "engine.metric.v1".to_string(),
        version: 1,
        metrics: serde_json::json!({}),
    };
    match validate_stage(result, metrics) {
        Ok(_) => panic!("expected stage failure"),
        Err(err) => assert!(err.to_string().contains("stage failed")),
    }
    Ok(())
}
