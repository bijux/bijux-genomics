use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result};

use bijux_dna_bench_model::BenchmarkObservation;
use bijux_dna_runtime::recording::write_atomic_bytes;

use super::{observation_reader, ObservationKey, WriteMode};

fn observation_key(obs: &BenchmarkObservation) -> ObservationKey {
    (
        obs.dataset_id.clone(),
        obs.stage_id.clone(),
        obs.tool_id.clone(),
        obs.params_hash.clone(),
        obs.replicate_id.clone(),
    )
}

fn canonical_json_line<T: serde::Serialize>(value: &T) -> Result<String> {
    let json = serde_json::to_value(value)?;
    let canonical = bijux_dna_core::contract::canonical::canonicalize_json_value(&json);
    Ok(serde_json::to_string(&canonical)?)
}

pub(super) fn write_observations_jsonl(
    path: &Path,
    observations: &[BenchmarkObservation],
    mode: WriteMode,
    tool_id_key: &str,
) -> Result<()> {
    let mut ordered = observations.to_vec();
    ordered.sort_by(|a, b| {
        (
            &a.dataset_id,
            &a.stage_id,
            &a.tool_id,
            &a.params_hash,
            &a.replicate_id,
            a.replicate_index,
        )
            .cmp(&(
                &b.dataset_id,
                &b.stage_id,
                &b.tool_id,
                &b.params_hash,
                &b.replicate_id,
                b.replicate_index,
            ))
    });
    let existing = if matches!(mode, WriteMode::Resume) {
        observation_reader::load_existing_keys(path, tool_id_key)?
    } else {
        BTreeSet::new()
    };
    let mut payload = String::new();
    for obs in ordered {
        if matches!(mode, WriteMode::Resume) && existing.contains(&observation_key(&obs)) {
            continue;
        }
        payload.push_str(&canonical_json_line(&obs)?);
        payload.push('\n');
    }
    write_atomic_bytes(path, payload.as_bytes())
        .with_context(|| format!("write observations {}", path.display()))
}
