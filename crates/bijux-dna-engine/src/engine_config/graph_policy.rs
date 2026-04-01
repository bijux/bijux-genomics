use bijux_dna_core::contract::ExecutionGraph;

use super::EngineConfig;

#[must_use]
pub fn apply_engine_config(graph: &ExecutionGraph, config: &EngineConfig) -> ExecutionGraph {
    let mut graph = graph.clone();
    if let Some(timeout) = config.step_timeout_s {
        graph = graph.with_step_timeout(Some(timeout));
    }
    if config.deterministic_scheduler {
        graph = graph.with_deterministic_scheduler(true);
    }
    if let Some(policy) = config.retry_policy.clone() {
        graph = graph.with_retry_policy(policy);
    }
    graph
}
