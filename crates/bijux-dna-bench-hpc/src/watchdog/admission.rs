use anyhow::{ensure, Result};
use bijux_dna_bench_model::{
    validate_benchmark_local_pipeline_dag_validation, BenchmarkLocalPipelineDagValidationReport,
};
use std::collections::BTreeSet;
pub(super) fn admit(report: &BenchmarkLocalPipelineDagValidationReport) -> Result<()> {
    validate_benchmark_local_pipeline_dag_validation(report)?;
    ensure!(!report.nodes.is_empty(), "watchdog simulation requires a nonempty DAG");
    let mut completed = BTreeSet::new();
    for node in &report.nodes {
        ensure!(
            node.depends_on.iter().all(|dependency| completed.contains(dependency.as_str())),
            "watchdog nodes must occur after every dependency: {}",
            node.node_id
        );
        ensure!(
            completed.insert(node.node_id.as_str()),
            "duplicate watchdog node: {}",
            node.node_id
        );
    }
    Ok(())
}
