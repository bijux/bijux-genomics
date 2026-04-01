//! Owner: bijux-dna-bench
//! Benchmark suite contract validation entrypoint.

mod edge_contracts;
mod stage_contracts;

use crate::diagnostics::BenchError;
use crate::model::BenchmarkSuiteSpec;

use super::SUITE_SCHEMA_V1;
use crate::contract::suite::{
    declared_graph_nodes, validate_suite_analysis_requirements, validate_suite_dag,
    validate_suite_diversity,
};
use edge_contracts::{validate_explicit_edges, validate_upstream_stage_references};
use stage_contracts::{validate_schema_version, validate_stage_definitions};

type DeclaredStageNodes = std::collections::BTreeSet<String>;

/// # Errors
/// Returns an error if the suite spec violates required fields.
pub fn validate_suite(suite: &BenchmarkSuiteSpec) -> Result<(), BenchError> {
    validate_schema_version(suite, SUITE_SCHEMA_V1)?;
    validate_suite_diversity(suite)?;
    let declared_stage_nodes = validate_stage_definitions(suite)?;
    let declared_graph_nodes = declared_graph_nodes(suite);
    validate_upstream_stage_references(suite, &declared_stage_nodes)?;
    validate_explicit_edges(suite, &declared_graph_nodes)?;
    validate_suite_analysis_requirements(suite)?;
    validate_suite_dag(suite)?;
    Ok(())
}
