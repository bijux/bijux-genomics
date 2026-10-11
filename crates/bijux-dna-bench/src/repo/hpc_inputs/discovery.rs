//! Owner: current HPC input acquisition and stage-hint provenance joins.
use super::path_usage::{collect_step_path_usage, materialize_stage_id};
use super::{HpcDiscoveredSourceInput, HpcStageInputHint};
use crate::{inspect_benchmark_source_asset, BenchmarkSourceAsset};
use anyhow::{anyhow, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
/// Observe source identities while excluding outputs produced by the command sequence.
/// # Errors
/// Returns missing input, stage-hint, path classification or source inspection failures.
pub fn collect_hpc_job_source_inputs<'a, I>(
    repo_root: &Path,
    consumer_label: &str,
    stage_id: &str,
    command_steps: I,
    stage_input_hints: &BTreeMap<String, Vec<HpcStageInputHint>>,
    generated_output_root: &str,
) -> Result<Vec<HpcDiscoveredSourceInput>>
where
    I: IntoIterator<Item = &'a [String]>,
{
    let mut discovered_by_source = BTreeMap::<String, HpcDiscoveredSourceInput>::new();
    let mut produced_paths = BTreeSet::<String>::new();

    for argv in command_steps {
        if let Some(materialized_stage_id) = materialize_stage_id(argv) {
            let hints = stage_input_hints.get(&materialized_stage_id).ok_or_else(|| {
                anyhow!(
                    "missing local stage input hints for materialized stage `{materialized_stage_id}`"
                )
            })?;
            for hint in hints {
                let resolved =
                    inspect_benchmark_source_asset(repo_root, &hint.source_path)?.ok_or_else(|| {
                        anyhow!(
                            "materialized stage `{materialized_stage_id}` depends on unresolved source `{}`",
                            hint.source_path
                        )
                    })?;
                merge_discovered_source_input(
                    &mut discovered_by_source,
                    resolved,
                    Some(hint.artifact_id.clone()),
                    Some(hint.artifact_role.clone()),
                );
            }
            continue;
        }

        let usage = collect_step_path_usage(argv, generated_output_root)?;
        for source_path in usage.consumed.difference(&usage.produced) {
            if produced_paths.contains(source_path.as_str()) {
                continue;
            }
            if let Some(resolved) = inspect_benchmark_source_asset(repo_root, source_path)? {
                merge_discovered_source_input(&mut discovered_by_source, resolved, None, None);
            }
        }
        produced_paths.extend(usage.produced);
    }

    if discovered_by_source.is_empty() {
        if let Some(hints) = stage_input_hints.get(stage_id) {
            for hint in hints {
                let resolved = inspect_benchmark_source_asset(repo_root, &hint.source_path)?
                    .ok_or_else(|| {
                        anyhow!(
                            "stage `{stage_id}` depends on unresolved source `{}`",
                            hint.source_path
                        )
                    })?;
                merge_discovered_source_input(
                    &mut discovered_by_source,
                    resolved,
                    Some(hint.artifact_id.clone()),
                    Some(hint.artifact_role.clone()),
                );
            }
        }
    }

    if discovered_by_source.is_empty() {
        return Err(anyhow!(
            "local HPC input discovery found no source inputs for `{consumer_label}`"
        ));
    }

    Ok(discovered_by_source.into_values().collect())
}

fn merge_discovered_source_input(
    discovered_by_source: &mut BTreeMap<String, HpcDiscoveredSourceInput>,
    resolved: BenchmarkSourceAsset,
    artifact_id: Option<String>,
    artifact_role: Option<String>,
) {
    discovered_by_source
        .entry(resolved.source_path.clone())
        .and_modify(|entry| {
            if entry.artifact_id.is_none() {
                entry.artifact_id.clone_from(&artifact_id);
            }
            if entry.artifact_role.is_none() {
                entry.artifact_role.clone_from(&artifact_role);
            }
        })
        .or_insert(HpcDiscoveredSourceInput { artifact_id, artifact_role, source_asset: resolved });
}
