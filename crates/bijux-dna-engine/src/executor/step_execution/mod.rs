use std::time::Duration;

use anyhow::{anyhow, Result};
use bijux_dna_core::contract::{
    ExecutionGraph, ExecutionStep, RunRecordV1, StageExecutionRecordV1,
};
use bijux_dna_runtime::{Invocation, Runner};
use chrono::Utc;

use crate::{CancellationToken, EngineEvent, EngineHooks};

use super::{contracts, recording};

mod stage_record;

pub(super) fn execute_ordered_steps(
    graph: &ExecutionGraph,
    ordered_steps: &[ExecutionStep],
    runner: &dyn Runner,
    hooks: Option<&dyn EngineHooks>,
    cancel: Option<&CancellationToken>,
) -> Result<RunRecordV1> {
    let mut results = Vec::with_capacity(ordered_steps.len());
    for step in ordered_steps {
        results.push(execute_step(graph, step, runner, hooks, cancel)?);
    }
    Ok(RunRecordV1::new(results))
}

fn execute_step(
    graph: &ExecutionGraph,
    step: &ExecutionStep,
    runner: &dyn Runner,
    hooks: Option<&dyn EngineHooks>,
    cancel: Option<&CancellationToken>,
) -> Result<StageExecutionRecordV1> {
    if cancel.is_some_and(CancellationToken::is_cancelled) {
        return Err(anyhow!("execution cancelled before {}", step.step_id.0));
    }
    tracing::info!(
        target: "exec.step",
        stage_id = %step.step_id.0,
        tool = %step.image.image,
        "execute step"
    );
    let mut attempt = 0;
    let last_success = loop {
        if cancel.is_some_and(CancellationToken::is_cancelled) {
            return Err(anyhow!("execution cancelled during {}", step.step_id.0));
        }
        if let Some(hooks) = hooks {
            hooks.on_event(EngineEvent::StepStart { step_id: step.step_id.clone(), attempt });
        }
        let started_at = Utc::now().to_rfc3339();
        let invocation = Invocation { step: step.clone(), attempt };
        let outcome = runner.run(&invocation)?;
        let duration = outcome.duration;
        let finished_at = Utc::now().to_rfc3339();
        recording::record_execution(
            step,
            attempt,
            &started_at,
            &finished_at,
            duration.as_secs_f64(),
            outcome.exit_code,
        )?;
        let success = outcome.exit_code == 0;
        if let Some(timeout_s) = graph.step_timeout_s() {
            if duration > Duration::from_secs(timeout_s) {
                return Err(anyhow!("step {} exceeded timeout {}s", step.step_id.0, timeout_s));
            }
        }
        if cancel.is_some_and(CancellationToken::is_cancelled) {
            return Err(anyhow!("execution cancelled during {}", step.step_id.0));
        }
        if success {
            contracts::enforce_contract(step)?;
            if let Some(hooks) = hooks {
                hooks.on_event(EngineEvent::StepEnd {
                    step_id: step.step_id.clone(),
                    attempt,
                    success: true,
                });
            }
            break success;
        }
        let retry_policy = graph.retry_policy();
        let allow_retry = retry_policy.retry_on_exit_codes.contains(&outcome.exit_code);
        if !allow_retry || attempt + 1 >= retry_policy.max_attempts {
            let step_id = step.step_id.to_string();
            return Err(anyhow!("step failed after retries: {step_id}"));
        }
        if let Some(hooks) = hooks {
            hooks.on_event(EngineEvent::Retry {
                step_id: step.step_id.clone(),
                attempt: attempt + 1,
                exit_code: outcome.exit_code,
            });
        }
        attempt += 1;
    };
    Ok(stage_record::stage_execution_record(step, attempt, last_success))
}
