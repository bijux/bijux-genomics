//! Owner: bijux-dna-engine

use anyhow::Result;
use bijux_dna_core::contract::{ExecutionGraph, RunRecordV1};
use bijux_dna_runtime::Runner;

use crate::{CancellationToken, EngineHooks};

mod contract_enforcer;
mod graph;
mod recording;
mod step_execution;
mod topology;

pub fn execute_plan(
    graph: &ExecutionGraph,
    runner: &dyn Runner,
    hooks: Option<&dyn EngineHooks>,
    cancel: Option<&CancellationToken>,
) -> Result<RunRecordV1> {
    let prepared = graph::normalize_for_execution(graph)?;
    step_execution::execute_ordered_steps(&prepared.graph, &prepared.ordered_steps, runner, hooks, cancel)
}
