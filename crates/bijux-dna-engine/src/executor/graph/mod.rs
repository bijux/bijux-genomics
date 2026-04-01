use anyhow::Result;
use bijux_dna_core::contract::{ExecutionGraph, ExecutionStep};

use crate::executor::topology;

pub(super) struct PreparedExecutionGraph {
    pub(super) graph: ExecutionGraph,
    pub(super) ordered_steps: Vec<ExecutionStep>,
}

pub(super) fn normalize_for_execution(graph: &ExecutionGraph) -> Result<PreparedExecutionGraph> {
    let graph = graph.normalize()?;
    let ordered_steps = topology::topo_order(
        graph.steps(),
        graph.edges(),
        graph.deterministic_scheduler(),
    )?
    .into_iter()
    .cloned()
    .collect();
    Ok(PreparedExecutionGraph {
        graph,
        ordered_steps,
    })
}
