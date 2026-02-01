//! Owner: bijux-bench
//! Deterministic, atomic artifact writers.
//! Owns bench output serialization.
//! Must not perform analysis logic.
//! Invariants: writes are atomic and stable.
#![allow(dead_code)]

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use std::collections::BTreeSet;

use crate::model::{BenchmarkObservation, BenchmarkSummary};
use crate::policy::GateDecision;

type ObservationKey = (String, String, String, String, String);

fn write_atomic_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("missing parent for {}", path.display()))?;
    fs::create_dir_all(dir)?;
    let mut temp = PathBuf::from(path);
    temp.set_extension("tmp");
    let mut file = File::create(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(&temp, path)?;
    Ok(())
}

/// Write observations as deterministic JSONL.
///
/// # Errors
/// Returns an error if the file cannot be written.
#[derive(Debug, Clone, Copy)]
pub enum WriteMode {
    Resume,
    Force,
}

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
    let canonical = bijux_core::canonicalize_json_value(&json);
    Ok(serde_json::to_string(&canonical)?)
}

fn load_existing_keys(path: &Path) -> Result<BTreeSet<ObservationKey>> {
    let mut keys = BTreeSet::new();
    if !path.exists() {
        return Ok(keys);
    }
    let raw = std::fs::read_to_string(path)?;
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)?;
        let key = (
            value
                .get("dataset_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            value
                .get("stage_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            value
                .get("tool_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            value
                .get("params_hash")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            value
                .get("replicate_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        );
        keys.insert(key);
    }
    Ok(keys)
}

/// Read observations from JSONL.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn read_observations_jsonl(path: &Path) -> Result<Vec<BenchmarkObservation>> {
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

pub fn write_observations_jsonl(
    path: &Path,
    observations: &[BenchmarkObservation],
    mode: WriteMode,
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
        load_existing_keys(path)?
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

/// Write summary JSON.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_summary_json(path: &Path, summary: &BenchmarkSummary) -> Result<()> {
    let json = serde_json::to_value(summary)?;
    let canonical = bijux_core::canonicalize_json_value(&json);
    let bytes = serde_json::to_vec_pretty(&canonical)?;
    write_atomic_bytes(path, &bytes).with_context(|| format!("write summary {}", path.display()))
}

pub fn write_decision_json(path: &Path, decision: &GateDecision) -> Result<()> {
    let json = serde_json::to_value(decision)?;
    let canonical = bijux_core::canonicalize_json_value(&json);
    let bytes = serde_json::to_vec_pretty(&canonical)?;
    write_atomic_bytes(path, &bytes).with_context(|| format!("write decision {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{
        read_observations_jsonl, write_decision_json, write_observations_jsonl, write_summary_json,
        WriteMode,
    };
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use crate::model::{BenchmarkObservation, BenchmarkSummary, MetricSummary, SummaryRow};
    use crate::policy::GateDecision;
    use crate::stats::robust_stats;
    use crate::MetricsEnvelope;

    #[test]
    fn artifacts_are_stable_and_atomic() -> anyhow::Result<()> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let out_dir = manifest_dir
            .join("target")
            .join("test-fixtures")
            .join("bench_artifacts");
        std::fs::create_dir_all(&out_dir)?;

        let obs = BenchmarkObservation {
            schema_version: "bijux.bench.observation.v1".to_string(),
            run_id: "run-1".to_string(),
            dataset_id: "dataset-1".to_string(),
            dataset_class: "trueseq".to_string(),
            read_layout: "paired".to_string(),
            stage_id: "fastq.trim".to_string(),
            tool_id: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: "sha256:abc".to_string(),
            params_hash: "params-a".to_string(),
            input_hash: "input".to_string(),
            runtime_s: 1.0,
            memory_mb: 100.0,
            exit_code: 0,
            failure_kind: None,
            metrics: MetricsEnvelope {
                stage_id: "fastq.trim".to_string(),
                schema_version: "metrics.v1".to_string(),
                values: BTreeMap::new(),
            },
            replicate_id: "r1".to_string(),
            replicate_index: 0,
            runner: "docker".to_string(),
            platform: "linux".to_string(),
            cpu: "x86_64".to_string(),
            threads: 4,
            io_mode: "local".to_string(),
        };
        let observations = vec![obs];
        write_observations_jsonl(
            &out_dir.join("observations.jsonl"),
            &observations,
            WriteMode::Force,
        )?;

        let summary = BenchmarkSummary {
            schema_version: "bijux.bench.summary.v1".to_string(),
            suite_id: "suite-1".to_string(),
            rows: vec![SummaryRow {
                dataset_id: "dataset-1".to_string(),
                dataset_class: "trueseq".to_string(),
                read_layout: "paired".to_string(),
                stage_id: "fastq.trim".to_string(),
                tool_id: "fastp".to_string(),
                params_hash: "params-a".to_string(),
                runtime: MetricSummary {
                    metric_id: "runtime_s".to_string(),
                    n: 3,
                    stats: robust_stats(&[1.0, 1.1, 0.9]),
                    ci_low: Some(0.9),
                    ci_high: Some(1.1),
                    outlier_count: 0,
                    outlier_replicates: Vec::new(),
                    practical_threshold: Some(0.05),
                    power_warning: false,
                },
                memory: MetricSummary {
                    metric_id: "memory_mb".to_string(),
                    n: 3,
                    stats: robust_stats(&[100.0, 110.0, 90.0]),
                    ci_low: Some(90.0),
                    ci_high: Some(110.0),
                    outlier_count: 0,
                    outlier_replicates: Vec::new(),
                    practical_threshold: Some(0.05),
                    power_warning: false,
                },
                metrics: Vec::new(),
                failure_rate: 0.0,
                completeness: 1.0,
                n_effective: 3,
                low_power: false,
            }],
            strata: Vec::new(),
            warnings: Vec::new(),
            scientifically_invalid: false,
            invalid_reasons: Vec::new(),
        };
        write_summary_json(&out_dir.join("summary.json"), &summary)?;

        let decision = GateDecision {
            schema_version: "bijux.bench.gate.v1".to_string(),
            dataset_id: "dataset-1".to_string(),
            stage_id: "fastq.trim".to_string(),
            tool_id: "fastp".to_string(),
            params_hash: "params-a".to_string(),
            passes: true,
            violations: Vec::new(),
            missing_metrics: Vec::new(),
            completeness_score: 1.0,
            rationale_trace: Vec::new(),
        };
        write_decision_json(&out_dir.join("decision.json"), &decision)?;

        let observations_text = std::fs::read_to_string(out_dir.join("observations.jsonl"))?;
        let summary_text = std::fs::read_to_string(out_dir.join("summary.json"))?;
        let decision_text = std::fs::read_to_string(out_dir.join("decision.json"))?;

        let expected_observation = {
            let json = serde_json::to_value(&observations[0])?;
            let canonical = bijux_core::canonicalize_json_value(&json);
            serde_json::to_string(&canonical)?
        };
        assert_eq!(observations_text.trim(), expected_observation.trim());

        let expected_summary = {
            let json = serde_json::to_value(&summary)?;
            let canonical = bijux_core::canonicalize_json_value(&json);
            serde_json::to_string_pretty(&canonical)?
        };
        assert_eq!(summary_text.trim(), expected_summary.trim());

        let expected_decision = {
            let json = serde_json::to_value(&decision)?;
            let canonical = bijux_core::canonicalize_json_value(&json);
            serde_json::to_string_pretty(&canonical)?
        };
        assert_eq!(decision_text.trim(), expected_decision.trim());

        let reloaded = read_observations_jsonl(&out_dir.join("observations.jsonl"))?;
        assert_eq!(reloaded.len(), 1);
        Ok(())
    }
}
