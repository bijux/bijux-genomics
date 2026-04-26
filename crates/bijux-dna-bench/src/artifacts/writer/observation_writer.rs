//! Owner: bijux-dna-bench
//! Deterministic observation JSONL writer.

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
        obs.stage_instance_id.clone(),
        obs.lineage_id.clone(),
        obs.tool_id.clone(),
        obs.params_hash.clone(),
        obs.replicate_id.clone(),
        obs.replicate_index,
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
    _tool_id_key: &str,
) -> Result<()> {
    let mut ordered = if matches!(mode, WriteMode::Resume) {
        observation_reader::read_observations_jsonl(path)?
    } else {
        Vec::new()
    };
    let mut existing = ordered.iter().map(observation_key).collect::<BTreeSet<_>>();
    for obs in observations {
        if existing.insert(observation_key(obs)) {
            ordered.push(obs.clone());
        }
    }
    ordered.sort_by(|a, b| {
        (&a.dataset_id, &a.stage_id, &a.tool_id, &a.params_hash, &a.replicate_id, a.replicate_index)
            .cmp(&(
                &b.dataset_id,
                &b.stage_id,
                &b.tool_id,
                &b.params_hash,
                &b.replicate_id,
                b.replicate_index,
            ))
    });
    let mut payload = String::new();
    for obs in ordered {
        payload.push_str(&canonical_json_line(&obs)?);
        payload.push('\n');
    }
    write_atomic_bytes(path, payload.as_bytes())
        .with_context(|| format!("write observations {}", path.display()))
}
