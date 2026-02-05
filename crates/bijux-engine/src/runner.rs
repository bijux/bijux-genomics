use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::{anyhow, Result};
use bijux_core::plan::execution_plan::{ExecutionPlan, PlanEdge};
use bijux_core::{RunRecordV1, StageExecutionRecordV1, StagePlanV1};
use bijux_runner::{Invocation, Runner};

#[derive(Debug, Clone, Default)]
pub struct ExecutionOptions {
    pub retries: u32,
    pub resume: bool,
}

pub fn execute_plan(
    plan: &ExecutionPlan,
    runner: &dyn Runner,
    options: &ExecutionOptions,
) -> Result<RunRecordV1> {
    let ordered = topo_order(plan.stages(), plan.edges())?;
    let mut results = Vec::with_capacity(ordered.len());
    for stage in ordered {
        let mut attempt = 0;
        let last_success = loop {
            let invocation = Invocation {
                stage: stage.clone(),
                attempt,
            };
            let outcome = runner.run(&invocation)?;
            let success = outcome.exit_code == 0;
            if success {
                break success;
            }
            if attempt >= options.retries {
                let stage_id = stage.stage_id.0.clone();
                return Err(anyhow!("stage failed after retries: {stage_id}"));
            }
            attempt += 1;
        };
        results.push(StageExecutionRecordV1 {
            stage_id: stage.stage_id.0.clone(),
            attempt,
            success: last_success,
            cached: false,
        });
    }
    Ok(RunRecordV1::new(results))
}

fn topo_order<'a>(
    stages: &'a [StagePlanV1],
    edges: &'a [PlanEdge],
) -> Result<Vec<&'a StagePlanV1>> {
    let mut by_id: HashMap<&str, &StagePlanV1> = HashMap::new();
    for stage in stages {
        by_id.insert(stage.stage_id.0.as_str(), stage);
    }
    let mut indegree: HashMap<&str, usize> = stages
        .iter()
        .map(|stage| (stage.stage_id.0.as_str(), 0))
        .collect();
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in edges {
        let from = edge.from();
        let to = edge.to();
        if !by_id.contains_key(from) || !by_id.contains_key(to) {
            return Err(anyhow!("edge references unknown stage: {from} -> {to}"));
        }
        adjacency.entry(from).or_default().push(to);
        *indegree.entry(to).or_insert(0) += 1;
    }
    let mut queue: VecDeque<&str> = stages
        .iter()
        .filter(|stage| {
            indegree
                .get(stage.stage_id.0.as_str())
                .copied()
                .unwrap_or(0)
                == 0
        })
        .map(|stage| stage.stage_id.0.as_str())
        .collect();
    let mut order = Vec::with_capacity(stages.len());
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
    if order.len() != stages.len() {
        return Err(anyhow!("execution plan contains a cycle"));
    }
    Ok(order)
}
