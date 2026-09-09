//! Owner: bijux-dna-bench-model
//! Suite validation orchestration entrypoint.

use crate::contract::suite::{
    declared_graph_nodes, validate_suite_analysis_requirements, validate_suite_dag,
    validate_suite_diversity,
};
use crate::contract::suite::{BenchmarkSuiteCatalog, CanonicalFastqSuiteCatalog};
use crate::contract::SUITE_SCHEMA_V1;
use crate::diagnostics::BenchError;
use crate::model::BenchmarkSuiteSpec;

use super::edge_contracts::{validate_explicit_edges, validate_upstream_stage_references};
use super::stage_contracts::{validate_schema_version, validate_stage_definitions};

/// # Errors
/// Returns an error if the suite spec violates required fields.
pub fn validate_suite(suite: &BenchmarkSuiteSpec) -> Result<(), BenchError> {
    validate_suite_with_catalog(suite, &CanonicalFastqSuiteCatalog)
}

/// Validates a suite using explicit catalog observations supplied by the caller.
///
/// # Errors
/// Returns an error when the suite violates structural or catalog admission rules.
pub fn validate_suite_with_catalog(
    suite: &BenchmarkSuiteSpec,
    catalog: &dyn BenchmarkSuiteCatalog,
) -> Result<(), BenchError> {
    validate_schema_version(suite, SUITE_SCHEMA_V1)?;
    validate_suite_diversity(suite)?;
    let declared_stage_nodes = validate_stage_definitions(suite, catalog)?;
    let declared_graph_nodes = declared_graph_nodes(suite);
    validate_upstream_stage_references(suite, &declared_stage_nodes)?;
    validate_explicit_edges(suite, &declared_graph_nodes, catalog)?;
    validate_suite_analysis_requirements(suite)?;
    validate_suite_dag(suite)?;
    Ok(())
}
