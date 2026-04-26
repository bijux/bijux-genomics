//! Owner: bijux-dna-bench
//! Deterministic observation JSONL reader and contract validator.

use std::path::Path;

use anyhow::{Context, Result};

use bijux_dna_bench_model::contract::validate_observation;
use bijux_dna_bench_model::BenchmarkObservation;

pub(super) type ObservationKey =
    (String, String, Option<String>, Option<String>, String, String, String, u32);

pub(super) fn read_observations_jsonl(path: &Path) -> Result<Vec<BenchmarkObservation>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read observations {}", path.display()))?;
    let mut observations = Vec::new();
    for (line_number, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let obs: BenchmarkObservation = serde_json::from_str(line).with_context(|| {
            format!("parse observation {} line {}", path.display(), line_number + 1)
        })?;
        validate_observation(&obs).with_context(|| {
            format!("validate observation {} line {}", path.display(), line_number + 1)
        })?;
        observations.push(obs);
    }
    Ok(observations)
}
