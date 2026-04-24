use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::{anyhow, Result};
use bijux_dna_core::contract::{ExecutionEdge, ExecutionStep};

pub(crate) fn topo_order<'a>(
    steps: &'a [ExecutionStep],
    edges: &'a [ExecutionEdge],
    deterministic: bool,
) -> Result<Vec<&'a ExecutionStep>> {
    let mut by_id: HashMap<&str, &ExecutionStep> = HashMap::new();
    for step in steps {
        by_id.insert(step.step_id.as_str(), step);
    }
    let mut indegree: HashMap<&str, usize> =
        steps.iter().map(|step| (step.step_id.as_str(), 0)).collect();
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
