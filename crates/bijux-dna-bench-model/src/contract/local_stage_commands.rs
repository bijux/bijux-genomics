//! Preserve public error taxonomy while canonical stage contracts own admission.
use crate::BenchError;

/// Admit canonical stage records.
///
/// # Errors
/// Maps semantic rejection into the retained benchmark policy error.
pub fn validate_benchmark_local_stage_commands(
    manifest: &crate::BenchmarkLocalStageCommandManifest,
) -> Result<(), BenchError> {
    bijux_dna_bench_stage::validate_benchmark_local_stage_commands(manifest)
        .map_err(|error| BenchError::InvalidPolicy(error.into_reason()))
}
