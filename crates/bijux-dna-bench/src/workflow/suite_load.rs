//! Owner: bijux-dna-bench
//! Repository-backed loading for benchmark observations.

use anyhow::Result;

use crate::repo::RunRepository;
use bijux_dna_bench_model::BenchmarkObservation;

/// Load observations for a suite from a repository.
///
/// # Errors
/// Returns an error if the repository cannot load observations.
pub fn load_suite(
    repo: &dyn RunRepository,
    run_ids: Option<&[String]>,
) -> Result<Vec<BenchmarkObservation>> {
    let ids = match run_ids {
        Some(ids) => ids.to_vec(),
        None => repo.list_runs()?,
    };
    let mut observations = Vec::new();
    for run_id in ids {
        observations.extend(repo.load_observations(&run_id)?);
    }
    Ok(observations)
}
