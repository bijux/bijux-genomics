//! Fast-gate critical path and qualified timing observations.

use crate::commands::ops::tooling::ci::fast::contract::FastGroup;
use crate::commands::ops::tooling::ci::fast::contract::FastReferenceRunner;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(super) const FAST_TIMING_SCHEMA_V5: &str = "bijux.ci.fast_timing.v5";

pub(super) const CI_CRITICAL_PATH_BASELINE_SCHEMA_V1: &str = "bijux.ci.critical_path_baseline.v1";

pub(super) const FAST_TIMING_HISTORY_SCHEMA_V4: &str = "bijux.ci.fast_timing_history.v4";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FastTimingHistory {
    pub(super) schema_version: String,
    pub(super) observations: Vec<FastTimingObservation>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FastTimingObservation {
    pub(super) observed_at_utc: String,
    pub(super) source_commit: String,
    pub(super) os: String,
    pub(super) architecture: String,
    #[serde(default)]
    pub(super) rust_toolchain: String,
    #[serde(default)]
    pub(super) compiler_wrapper: String,
    pub(super) cache_state: String,
    #[serde(default)]
    pub(super) cache_authority: String,
    #[serde(default)]
    pub(super) build_authority: String,
    #[serde(default)]
    pub(super) cache_hit_ratio_basis_points: u64,
    #[serde(default)]
    pub(super) cache_reuse_mode: String,
    pub(super) wall_time_milliseconds: u64,
}

pub(super) fn cargo_lock_wait_events(log: &[u8]) -> usize {
    String::from_utf8_lossy(log)
        .to_ascii_lowercase()
        .matches("blocking waiting for file lock")
        .count()
}

pub(super) fn ci_fast_critical_path(
    groups: &[FastGroup],
    durations: &BTreeMap<String, u64>,
) -> Option<(Vec<String>, u64)> {
    fn visit(
        id: &str,
        groups: &BTreeMap<&str, &FastGroup>,
        durations: &BTreeMap<String, u64>,
        memo: &mut BTreeMap<String, (Vec<String>, u64)>,
    ) -> (Vec<String>, u64) {
        if let Some(cached) = memo.get(id) {
            return cached.clone();
        }
        let group = groups[id];
        let mut path = group
            .dependencies
            .iter()
            .map(|dependency| visit(dependency, groups, durations, memo))
            .max_by_key(|(_, duration)| *duration)
            .unwrap_or_default();
        path.0.push(id.to_string());
        path.1 += durations.get(id).copied().unwrap_or_default();
        memo.insert(id.to_string(), path.clone());
        path
    }

    let groups_by_id =
        groups.iter().map(|group| (group.id.as_str(), group)).collect::<BTreeMap<_, _>>();
    let mut memo = BTreeMap::new();
    groups
        .iter()
        .map(|group| visit(&group.id, &groups_by_id, durations, &mut memo))
        .max_by_key(|(_, duration)| *duration)
}

pub(super) fn load_fast_timing_history(path: &Path) -> Result<FastTimingHistory> {
    if !path.try_exists()? {
        return Ok(FastTimingHistory {
            schema_version: FAST_TIMING_HISTORY_SCHEMA_V4.to_string(),
            observations: Vec::new(),
        });
    }
    let mut history: FastTimingHistory = serde_json::from_slice(&fs::read(path)?)
        .with_context(|| format!("parse {}", path.display()))?;
    if matches!(
        history.schema_version.as_str(),
        "bijux.ci.fast_timing_history.v1"
            | "bijux.ci.fast_timing_history.v2"
            | "bijux.ci.fast_timing_history.v3"
    ) {
        history.schema_version = FAST_TIMING_HISTORY_SCHEMA_V4.to_string();
    } else if history.schema_version != FAST_TIMING_HISTORY_SCHEMA_V4 {
        anyhow::bail!("unsupported ci-fast timing history schema");
    }
    Ok(history)
}

pub(super) fn percentile(values: &[u64], percentile: usize) -> Option<u64> {
    if values.is_empty() {
        return None;
    }
    let mut ordered = values.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100).saturating_sub(1);
    ordered.get(rank).copied()
}

pub(super) fn reference_timing_samples(
    history: &FastTimingHistory,
    reference: &FastReferenceRunner,
) -> Vec<u64> {
    history
        .observations
        .iter()
        .filter(|observation| {
            observation.os == reference.os
                && observation.architecture == reference.architecture
                && observation.rust_toolchain == reference.rust_toolchain
                && observation.compiler_wrapper == reference.compiler_wrapper
                && observation.cache_state == "warm_verified"
                && !observation.cache_authority.is_empty()
                && !observation.build_authority.is_empty()
                && match observation.cache_reuse_mode.as_str() {
                    "build_target" => true,
                    "compiler_cache" | "" => observation.cache_hit_ratio_basis_points > 0,
                    _ => false,
                }
        })
        .map(|observation| observation.wall_time_milliseconds)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::cargo_lock_wait_events;
    use super::ci_fast_critical_path;
    use super::percentile;
    use super::reference_timing_samples;
    use super::FastTimingHistory;
    use super::FastTimingObservation;
    use super::FAST_TIMING_HISTORY_SCHEMA_V4;
    use crate::commands::ops::tooling::ci::fast::contract::fast_contract;
    use crate::commands::ops::tooling::ci::fast::contract::FastGroup;
    use std::collections::BTreeMap;

    #[test]
    fn cargo_lock_wait_measurement_counts_diagnostics_without_inference() {
        let log = b"Blocking waiting for file lock on build directory\n\
                    Compiling example v0.1.0\n\
                    blocking waiting for file lock on package cache\n";
        assert_eq!(cargo_lock_wait_events(log), 2);
        assert_eq!(cargo_lock_wait_events(b"Compiling without contention\n"), 0);
    }

    #[test]
    fn critical_path_accumulates_dependency_chain_durations() {
        let groups = vec![
            FastGroup {
                id: "source".to_string(),
                dependencies: Vec::new(),
                program: "true".to_string(),
                args: Vec::new(),
            },
            FastGroup {
                id: "dependent".to_string(),
                dependencies: vec!["source".to_string()],
                program: "true".to_string(),
                args: Vec::new(),
            },
            FastGroup {
                id: "independent".to_string(),
                dependencies: Vec::new(),
                program: "true".to_string(),
                args: Vec::new(),
            },
        ];
        let durations = BTreeMap::from([
            ("source".to_string(), 40),
            ("dependent".to_string(), 30),
            ("independent".to_string(), 60),
        ]);

        assert_eq!(
            ci_fast_critical_path(&groups, &durations),
            Some((vec!["source".to_string(), "dependent".to_string()], 70))
        );
    }

    #[test]
    fn timing_percentiles_use_nearest_rank() {
        assert_eq!(percentile(&[], 95), None);
        assert_eq!(percentile(&[100, 20, 40, 60, 80], 50), Some(60));
        assert_eq!(percentile(&[100, 20, 40, 60, 80], 95), Some(100));
    }

    #[test]
    fn timing_samples_require_exact_warm_reference_runner() {
        let reference = fast_contract().reference_runner;
        let observation = |os: &str,
                           architecture: &str,
                           rust_toolchain: &str,
                           compiler_wrapper: &str,
                           cache_state: &str| {
            FastTimingObservation {
                observed_at_utc: "2026-08-10T00:00:00Z".to_string(),
                source_commit: "0123456789abcdef".to_string(),
                os: os.to_string(),
                architecture: architecture.to_string(),
                rust_toolchain: rust_toolchain.to_string(),
                compiler_wrapper: compiler_wrapper.to_string(),
                cache_state: cache_state.to_string(),
                cache_authority: if cache_state == "warm_verified" {
                    "/shared/sccache".to_string()
                } else {
                    String::new()
                },
                build_authority: if cache_state == "warm_verified" {
                    "/shared/ci-fast-target".to_string()
                } else {
                    String::new()
                },
                cache_hit_ratio_basis_points: if cache_state == "warm_verified" {
                    9_000
                } else {
                    0
                },
                cache_reuse_mode: if cache_state == "warm_verified" {
                    "compiler_cache".to_string()
                } else {
                    String::new()
                },
                wall_time_milliseconds: 42,
            }
        };
        let mut reused_build_target =
            observation("linux", "x86_64", "1.95.0", "sccache", "warm_verified");
        reused_build_target.cache_hit_ratio_basis_points = 0;
        reused_build_target.cache_reuse_mode = "build_target".to_string();
        reused_build_target.wall_time_milliseconds = 43;
        let history = FastTimingHistory {
            schema_version: FAST_TIMING_HISTORY_SCHEMA_V4.to_string(),
            observations: vec![
                observation("linux", "x86_64", "1.95.0", "sccache", "warm_verified"),
                reused_build_target,
                observation("linux", "x86_64", "1.95.0", "sccache", "warm"),
                observation("macos", "aarch64", "1.95.0", "sccache", "warm_verified"),
                observation("linux", "x86_64", "1.94.0", "sccache", "warm_verified"),
                observation("linux", "x86_64", "1.95.0", "sccache", "cold"),
                observation("linux", "x86_64", "1.95.0", "", "warm_verified"),
            ],
        };

        assert_eq!(reference_timing_samples(&history, &reference), vec![42, 43]);
    }
}
