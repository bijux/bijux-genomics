//! Owner: bijux-engine

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;

use anyhow::{anyhow, Result};
use bijux_core::plan::execution_graph::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_core::plan::{Invocation, Runner};
use bijux_core::{RunRecordV1, StageExecutionRecordV1};

#[derive(Debug, Clone, Default)]
pub struct ExecutionOptions {
    pub retries: u32,
}

pub fn execute_plan(
    graph: &ExecutionGraph,
    runner: &dyn Runner,
    options: &ExecutionOptions,
) -> Result<RunRecordV1> {
    let graph = graph.normalize()?;
    let ordered = topo_order(graph.steps(), graph.edges())?;
    let mut results = Vec::with_capacity(ordered.len());
    for step in ordered {
        tracing::info!(
            target: "exec.step",
            stage_id = %step.step_id.0,
            tool = %step.image.image,
            "execute step"
        );
        let mut attempt = 0;
        let last_success = loop {
            let invocation = Invocation {
                step: step.clone(),
                attempt,
            };
            let outcome = runner.run(&invocation)?;
            let success = outcome.exit_code == 0;
            if success {
                enforce_contract(step)?;
                break success;
            }
            if attempt >= options.retries {
                let step_id = step.step_id.to_string();
                return Err(anyhow!("step failed after retries: {step_id}"));
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
    for output in &step.io.outputs {
        if output.optional && !output.path.exists() {
            continue;
        }
        if !output.path.exists() {
            return Err(anyhow!(
                "contract error: missing output {} at {}",
                output.name,
                output.path.display()
            ));
        }
        let metadata = fs::metadata(&output.path).map_err(|err| {
            anyhow!(
                "contract error: unable to stat output {}: {err}",
                output.path.display()
            )
        })?;
        if metadata.len() == 0 {
            return Err(anyhow!(
                "contract error: output {} is empty at {}",
                output.name,
                output.path.display()
            ));
        }
        if matches!(
            output.role,
            bijux_core::contract::ArtifactRole::MetricsJson
                | bijux_core::contract::ArtifactRole::MetricsEnvelope
        ) {
            let raw = fs::read_to_string(&output.path)?;
            serde_json::from_str::<serde_json::Value>(&raw).map_err(|err| {
                anyhow!(
                    "contract error: metrics output {} not parseable: {err}",
                    output.path.display()
                )
            })?;
        }
    }
    if !step.metrics_schema_ids.is_empty() {
        let metrics_path = step
            .out_dir
            .join("run_artifacts")
            .join("metrics_envelope.json");
        if !metrics_path.exists() {
            return Err(anyhow!(
                "contract error: missing metrics_envelope.json for {}",
                step.step_id.0
            ));
        }
        let raw = fs::read_to_string(&metrics_path)?;
        serde_json::from_str::<serde_json::Value>(&raw).map_err(|err| {
            anyhow!(
                "contract error: metrics_envelope.json parse failed for {}: {err}",
                step.step_id.0
            )
        })?;
    }
    Ok(())
}

fn topo_order<'a>(
    steps: &'a [ExecutionStep],
    edges: &'a [ExecutionEdge],
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
