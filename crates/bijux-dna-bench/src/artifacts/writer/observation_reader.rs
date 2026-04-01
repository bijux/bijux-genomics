use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;

use bijux_dna_bench_model::BenchmarkObservation;

pub(super) type ObservationKey = (String, String, String, String, String);

fn required_jsonl_key<'a>(
    value: &'a serde_json::Value,
    field: &str,
    line_number: usize,
) -> Result<&'a str> {
    value
        .get(field)
        .and_then(|entry| entry.as_str())
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .ok_or_else(|| anyhow::anyhow!("observation row {line_number} missing `{field}`"))
}

pub(super) fn load_existing_keys(path: &Path, tool_id_key: &str) -> Result<BTreeSet<ObservationKey>> {
    let mut keys = BTreeSet::new();
    if !path.exists() {
        return Ok(keys);
    }
    let raw = std::fs::read_to_string(path)?;
    for (line_number, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)?;
        let key = (
            required_jsonl_key(&value, "dataset_id", line_number + 1)?.to_string(),
            required_jsonl_key(&value, "stage_id", line_number + 1)?.to_string(),
            required_jsonl_key(&value, tool_id_key, line_number + 1)?.to_string(),
            required_jsonl_key(&value, "params_hash", line_number + 1)?.to_string(),
            required_jsonl_key(&value, "replicate_id", line_number + 1)?.to_string(),
        );
        keys.insert(key);
    }
    Ok(keys)
}

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
