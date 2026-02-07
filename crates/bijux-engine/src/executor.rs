//! Owner: bijux-engine

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;

use anyhow::{anyhow, Result};
use bijux_core::contract::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_core::contract::{RunRecordV1, StageExecutionRecordV1};
use bijux_infra::ensure_dir;
use bijux_runtime::{Invocation, Runner};
use chrono::Utc;

use crate::{CancellationToken, EngineEvent, EngineHooks};

pub fn execute_plan(
    graph: &ExecutionGraph,
    runner: &dyn Runner,
    hooks: Option<&dyn EngineHooks>,
    cancel: Option<&CancellationToken>,
) -> Result<RunRecordV1> {
    let graph = graph.normalize()?;
    let ordered = topo_order(
        graph.steps(),
        graph.edges(),
        graph.deterministic_scheduler(),
    )?;
    let mut results = Vec::with_capacity(ordered.len());
    for step in ordered {
        if cancel.is_some_and(CancellationToken::is_cancelled) {
            return Err(anyhow!("execution cancelled before {}", step.step_id.0));
        }
        if let Some(hooks) = hooks {
            hooks.on_event(EngineEvent::StepStart {
                step_id: step.step_id.clone(),
                attempt: 0,
            });
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
            let started_at = Utc::now().to_rfc3339();
            let invocation = Invocation {
                step: step.clone(),
                attempt,
            };
            let outcome = runner.run(&invocation)?;
            let duration = outcome.duration;
            let finished_at = Utc::now().to_rfc3339();
            record_execution(
                step,
                attempt,
                &started_at,
                &finished_at,
                duration.as_secs_f64(),
                outcome.exit_code,
            )?;
            let success = outcome.exit_code == 0;
            if let Some(timeout_s) = graph.step_timeout_s() {
                if duration.as_secs() > timeout_s {
                    return Err(anyhow!(
                        "step {} exceeded timeout {}s",
                        step.step_id.0,
                        timeout_s
                    ));
                }
            }
            if success {
                enforce_contract(step)?;
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
            let allow_retry = retry_policy
                .retry_on_exit_codes
                .contains(&outcome.exit_code);
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
        results.push(StageExecutionRecordV1 {
            stage_id: step.step_id.to_string(),
            attempt,
            success: last_success,
            cached: false,
        });
    }
    Ok(RunRecordV1::new(results))
}

fn enforce_contract(step: &ExecutionStep) -> Result<()> {
    ContractEnforcer::new(step).enforce()
}

fn topo_order<'a>(
    steps: &'a [ExecutionStep],
    edges: &'a [ExecutionEdge],
    deterministic: bool,
) -> Result<Vec<&'a ExecutionStep>> {
    let mut by_id: HashMap<&str, &ExecutionStep> = HashMap::new();
    for step in steps {
        by_id.insert(step.step_id.as_str(), step);
    }
    let mut indegree: HashMap<&str, usize> = steps
        .iter()
        .map(|step| (step.step_id.as_str(), 0))
        .collect();
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in edges {
        let from = edge.from().as_str();
        let to = edge.to().as_str();
        if !by_id.contains_key(from) || !by_id.contains_key(to) {
            return Err(anyhow!("edge references unknown step: {from} -> {to}"));
        }
        adjacency.entry(from).or_default().push(to);
        *indegree.entry(to).or_insert(0) += 1;
    }
    let mut queue: VecDeque<&str> = steps
        .iter()
        .filter(|step| indegree.get(step.step_id.as_str()).copied().unwrap_or(0) == 0)
        .map(|step| step.step_id.as_str())
        .collect();
    if deterministic {
        let mut ordered: Vec<&str> = queue.drain(..).collect();
        ordered.sort_unstable();
        queue.extend(ordered);
    }
    let mut order = Vec::with_capacity(steps.len());
    let mut seen = HashSet::new();
    while let Some(node) = queue.pop_front() {
        if !seen.insert(node) {
            continue;
        }
        if let Some(stage) = by_id.get(node) {
            order.push(*stage);
        }
        if let Some(children) = adjacency.get(node) {
            let mut children = children.clone();
            if deterministic {
                children.sort_unstable();
            }
            for child in children {
                let entry = indegree.entry(child).or_insert(0);
                *entry = entry.saturating_sub(1);
                if *entry == 0 {
                    queue.push_back(child);
                }
            }
        }
    }
    if order.len() != steps.len() {
        return Err(anyhow!("execution plan contains a cycle"));
    }
    Ok(order)
}

fn record_execution(
    step: &ExecutionStep,
    attempt: u32,
    started_at: &str,
    finished_at: &str,
    duration_s: f64,
    exit_code: i32,
) -> Result<()> {
    let run_artifacts_dir = step.out_dir.join("run_artifacts");
    ensure_dir(&run_artifacts_dir)?;
    let payload = serde_json::json!({
        "schema_version": "bijux.execution_record.v1",
        "step_id": step.step_id.to_string(),
        "stage_id": step.stage_id.to_string(),
        "attempt": attempt,
        "started_at": started_at,
        "finished_at": finished_at,
        "duration_s": duration_s,
        "exit_code": exit_code,
    });
    let path = run_artifacts_dir.join("execution_record.json");
    bijux_runtime::recording::write_canonical_json(&path, &payload)?;
    Ok(())
}

struct ContractEnforcer<'a> {
    step: &'a ExecutionStep,
}

impl<'a> ContractEnforcer<'a> {
    fn new(step: &'a ExecutionStep) -> Self {
        Self { step }
    }

    fn enforce(&self) -> Result<()> {
        self.verify_outputs()?;
        self.verify_metrics_envelope()?;
        self.verify_required_run_artifacts()?;
        Ok(())
    }

    fn contract_error(&self, artifact_id: &str, path: &str, message: &str) -> anyhow::Error {
        crate::errors::EngineError::Contract {
            step_id: self.step.step_id.as_str().to_string(),
            artifact_id: artifact_id.to_string(),
            path: path.to_string(),
            message: message.to_string(),
        }
        .into()
    }

    fn verify_outputs(&self) -> Result<()> {
        for output in &self.step.io.outputs {
            if output.optional && !output.path.exists() {
                continue;
            }
            if !output.path.exists() {
                return Err(self.contract_error(
                    output.name.as_str(),
                    &output.path.display().to_string(),
                    "missing output",
                ));
            }
            let metadata = fs::metadata(&output.path).map_err(|err| {
                self.contract_error(
                    output.name.as_str(),
                    &output.path.display().to_string(),
                    &format!("unable to stat output: {err}"),
                )
            })?;
            if metadata.len() == 0 {
                return Err(self.contract_error(
                    output.name.as_str(),
                    &output.path.display().to_string(),
                    "output is empty",
                ));
            }
            tracing::info!(
                target: "exec.contract",
                stage_id = %self.step.step_id.0,
                path = %output.path.display(),
                "artifact verified"
            );
            if matches!(
                output.role,
                bijux_core::contract::ArtifactRole::MetricsJson
                    | bijux_core::contract::ArtifactRole::MetricsEnvelope
            ) {
                let raw = fs::read_to_string(&output.path)?;
                serde_json::from_str::<serde_json::Value>(&raw).map_err(|err| {
                    self.contract_error(
                        output.name.as_str(),
                        &output.path.display().to_string(),
                        &format!("metrics output not parseable: {err}"),
                    )
                })?;
            }
        }
        Ok(())
    }

    fn verify_metrics_envelope(&self) -> Result<()> {
        if self.step.metrics_schema_ids.is_empty() {
            return Ok(());
        }
        let metrics_path = self
            .step
            .out_dir
            .join("run_artifacts")
            .join("metrics_envelope.json");
        if !metrics_path.exists() {
            return Err(self.contract_error(
                "metrics_envelope",
                &metrics_path.display().to_string(),
                "missing metrics_envelope.json",
            ));
        }
        let raw = fs::read_to_string(&metrics_path)?;
        serde_json::from_str::<serde_json::Value>(&raw).map_err(|err| {
            self.contract_error(
                "metrics_envelope",
                &metrics_path.display().to_string(),
                &format!("metrics_envelope.json parse failed: {err}"),
            )
        })?;
        Ok(())
    }

    fn verify_required_run_artifacts(&self) -> Result<()> {
        let run_artifacts_dir = self.step.out_dir.join("run_artifacts");
        let required = [
            ("metrics.json", run_artifacts_dir.join("metrics.json")),
            (
                "effective_config.json",
                run_artifacts_dir.join("effective_config.json"),
            ),
            (
                "stage_report.json",
                run_artifacts_dir.join("stage_report.json"),
            ),
            (
                "tool_invocation.json",
                run_artifacts_dir.join("tool_invocation.json"),
            ),
            (
                "execution_record.json",
                run_artifacts_dir.join("execution_record.json"),
            ),
        ];
        for (label, path) in required {
            if !path.exists() {
                return Err(self.contract_error(
                    label,
                    &path.display().to_string(),
                    "missing run artifact",
                ));
            }
            let metadata = fs::metadata(&path).map_err(|err| {
                self.contract_error(
                    label,
                    &path.display().to_string(),
                    &format!("unable to stat run artifact: {err}"),
                )
            })?;
            if metadata.len() == 0 {
                return Err(self.contract_error(
                    label,
                    &path.display().to_string(),
                    "artifact empty",
                ));
            }
        }
        Ok(())
    }
}
