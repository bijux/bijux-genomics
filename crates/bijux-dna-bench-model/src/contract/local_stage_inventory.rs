//! Preserve public error taxonomy while canonical stage contracts own admission.
use crate::BenchError;

/// Admit canonical stage records.
///
/// # Errors
/// Maps semantic rejection into the retained benchmark policy error.
pub fn validate_benchmark_local_stage_inventory(
    inventory: &crate::BenchmarkLocalStageInventory,
) -> Result<(), BenchError> {
    bijux_dna_bench_stage::validate_benchmark_local_stage_inventory(inventory)
        .map_err(|error| BenchError::InvalidPolicy(error.into_reason()))
}

/// Admit canonical stage records.
///
/// # Errors
/// Maps semantic rejection into the retained benchmark policy error.
pub fn validate_benchmark_local_all_domain_stage_inventory(
    report: &crate::BenchmarkLocalAllDomainStageInventory,
) -> Result<(), BenchError> {
    bijux_dna_bench_stage::validate_benchmark_local_all_domain_stage_inventory(report)
        .map_err(|error| BenchError::InvalidPolicy(error.into_reason()))
}
