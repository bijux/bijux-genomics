use std::path::PathBuf;

use anyhow::Result;

use bijux_dna_bench_model::BenchError;

pub fn load_observations(
    path: &PathBuf,
) -> Result<Vec<bijux_dna_bench_model::BenchmarkObservation>> {
    if !path.exists() {
        return Err(BenchError::MissingMetrics(format!(
            "observations file missing: {}",
            path.display()
        ))
        .into());
    }
    let raw = std::fs::read_to_string(path)?;
    let mut observations = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let obs: bijux_dna_bench_model::BenchmarkObservation = serde_json::from_str(line)?;
        observations.push(obs);
    }
    Ok(observations)
}
