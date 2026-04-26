use std::path::Path;

use anyhow::Result;

use bijux_dna_bench_model::BenchmarkObservation;

pub(super) type ObservationKey = (String, String, String, String, String);

pub(super) fn read_observations_jsonl(path: &Path) -> Result<Vec<BenchmarkObservation>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(path)?;
    let mut observations = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let obs: BenchmarkObservation = serde_json::from_str(line)?;
        observations.push(obs);
    }
    Ok(observations)
}
