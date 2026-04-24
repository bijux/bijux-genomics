use crate::internal::fastq::stages::preprocess::*;

use anyhow::{anyhow, Result};

pub(super) fn execution_step_batches(graph: &ExecutionGraph) -> Result<Vec<Vec<ExecutionStep>>> {
    let mut incoming = std::collections::BTreeMap::<String, usize>::new();
    let mut outgoing = std::collections::BTreeMap::<String, Vec<String>>::new();
    for step in graph.steps() {
        incoming.insert(step.step_id.as_str().to_string(), 0);
    }
    for edge in graph.edges() {
        *incoming.entry(edge.to().as_str().to_string()).or_insert(0) += 1;
        outgoing
            .entry(edge.from().as_str().to_string())
            .or_default()
            .push(edge.to().as_str().to_string());
    }
    let mut ready = incoming
        .iter()
        .filter_map(|(node_id, count)| if *count == 0 { Some(node_id.clone()) } else { None })
        .collect::<Vec<_>>();
    ready.sort();
    let mut batches = Vec::new();
    let mut visited = 0usize;
    while !ready.is_empty() {
        let current_batch_ids = std::mem::take(&mut ready);
        let mut batch = current_batch_ids
            .iter()
            .map(|step_id| {
                graph.step_by_id(step_id).cloned().ok_or_else(|| {
                    anyhow!(
                        "execution graph is missing planned step {step_id} during runtime batching"
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;
        batch.sort_by(|left, right| left.step_id.as_str().cmp(right.step_id.as_str()));
        visited += batch.len();
        batches.push(batch);
        let mut next_ready = Vec::new();
        for node_id in current_batch_ids {
            if let Some(children) = outgoing.get(&node_id) {
                for child in children {
                    if let Some(count) = incoming.get_mut(child) {
                        *count -= 1;
                        if *count == 0 {
                            next_ready.push(child.clone());
                        }
                    }
                }
            }
        }
        next_ready.sort();
        next_ready.dedup();
        ready = next_ready;
    }
    if visited != graph.steps().len() {
        return Err(anyhow!(
            "execution graph batching did not visit all steps; graph may be cyclic"
        ));
    }
    Ok(batches)
}

pub(super) fn terminal_step_ids(graph: &ExecutionGraph) -> Vec<bijux_dna_core::prelude::StepId> {
    let mut outgoing = std::collections::BTreeSet::new();
    for edge in graph.edges() {
        outgoing.insert(edge.from().as_str().to_string());
    }
    graph
        .steps()
        .iter()
        .filter(|step| !outgoing.contains(step.step_id.as_str()))
        .map(|step| step.step_id.clone())
        .collect()
}
