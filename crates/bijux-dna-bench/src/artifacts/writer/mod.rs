//! Owner: bijux-dna-bench
//! Deterministic, atomic artifact writers.
//! Owns bench output serialization.
//! Must not perform analysis logic.
//! Invariants: writes are atomic and stable.
#![allow(dead_code)]

mod observation_reader;
mod observation_writer;
mod structured_writer;

use std::path::Path;

use anyhow::Result;

use bijux_dna_bench_model::{BenchmarkObservation, BenchmarkSummary, GateDecision};

type ObservationKey = observation_reader::ObservationKey;

// write_atomic_bytes lives in bijux-dna-runtime::recording.

const TOOL_ID_KEY: &str = concat!("tool", "_", "id");

/// Write observations as deterministic JSONL.
///
/// # Errors
/// Returns an error if the file cannot be written.
#[derive(Debug, Clone, Copy)]
pub enum WriteMode {
    Resume,
    Force,
}

/// Read observations from JSONL.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn read_observations_jsonl(path: &Path) -> Result<Vec<BenchmarkObservation>> {
    observation_reader::read_observations_jsonl(path)
}

pub fn write_observations_jsonl(
    path: &Path,
    observations: &[BenchmarkObservation],
    mode: WriteMode,
) -> Result<()> {
    observation_writer::write_observations_jsonl(path, observations, mode, TOOL_ID_KEY)
}

/// Write summary JSON.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_summary_json(path: &Path, summary: &BenchmarkSummary) -> Result<()> {
    structured_writer::write_summary_json(path, summary)
}

pub fn write_decision_json(path: &Path, decision: &GateDecision) -> Result<()> {
    structured_writer::write_decision_json(path, decision)
}

pub fn write_decisions_json(path: &Path, decisions: &[GateDecision]) -> Result<()> {
    structured_writer::write_decisions_json(path, decisions)
}

#[cfg(test)]
mod tests {
    use super::{
        read_observations_jsonl, write_decision_json, write_decisions_json,
        write_observations_jsonl, write_summary_json, WriteMode,
    };
    use std::collections::BTreeMap;

    use crate::MetricsEnvelope;
    use bijux_dna_bench_model::{
        robust_stats, BenchmarkObservation, BenchmarkSummary, GateDecision, MetricSummary,
        SummaryRow,
    };

    fn sample_observation(run_id: &str, replicate_id: &str) -> BenchmarkObservation {
        BenchmarkObservation {
            schema_version: "bijux.bench.observation.v1".to_string(),
            run_id: run_id.to_string(),
            dataset_id: "dataset-1".to_string(),
            dataset_class: "trueseq".to_string(),
            read_layout: "paired".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: None,
            lineage_id: None,
            tool_id: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: "sha256:abc".to_string(),
            container_digest: "sha256:abc".to_string(),
            params_hash: "params-a".to_string(),
            input_hash: "input".to_string(),
            runtime_s: 1.0,
            memory_mb: 100.0,
            exit_code: 0,
            failure_kind: None,
            metrics: MetricsEnvelope {
                stage_id: "fastq.trim_reads".to_string(),
                schema_version: "metrics.v1".to_string(),
                values: BTreeMap::new(),
            },
            replicate_id: replicate_id.to_string(),
            replicate_index: 0,
            warmup_policy: "none".to_string(),
            seed_policy: "default".to_string(),
            runner: "docker".to_string(),
            platform: "linux".to_string(),
            cpu: "x86_64".to_string(),
            threads: 4,
            io_mode: "local".to_string(),
        }
    }

    #[test]
    fn artifacts_are_stable_and_atomic() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let out_dir = temp.path().join("bench_artifacts");
        bijux_dna_infra::ensure_dir(&out_dir)?;

        let obs = BenchmarkObservation {
            schema_version: "bijux.bench.observation.v1".to_string(),
            run_id: "run-1".to_string(),
            dataset_id: "dataset-1".to_string(),
            dataset_class: "trueseq".to_string(),
            read_layout: "paired".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: None,
            lineage_id: None,
            tool_id: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: "sha256:abc".to_string(),
            container_digest: "sha256:abc".to_string(),
            params_hash: "params-a".to_string(),
            input_hash: "input".to_string(),
            runtime_s: 1.0,
            memory_mb: 100.0,
            exit_code: 0,
            failure_kind: None,
            metrics: MetricsEnvelope {
                stage_id: "fastq.trim_reads".to_string(),
                schema_version: "metrics.v1".to_string(),
                values: BTreeMap::new(),
            },
            replicate_id: "r1".to_string(),
            replicate_index: 0,
            warmup_policy: "none".to_string(),
            seed_policy: "default".to_string(),
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
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: None,
                lineage_id: None,
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
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "fastp".to_string(),
            params_hash: "params-a".to_string(),
            passes: true,
            violations: Vec::new(),
            missing_metrics: Vec::new(),
            completeness_score: 1.0,
            rationale_trace: Vec::new(),
        };
        write_decision_json(&out_dir.join("decision.json"), &decision)?;
        write_decisions_json(&out_dir.join("decisions.json"), &[decision.clone()])?;

        let observations_text = std::fs::read_to_string(out_dir.join("observations.jsonl"))?;
        let summary_text = std::fs::read_to_string(out_dir.join("summary.json"))?;
        let decision_text = std::fs::read_to_string(out_dir.join("decision.json"))?;
        let decisions_text = std::fs::read_to_string(out_dir.join("decisions.json"))?;

        let expected_observation = {
            let json = serde_json::to_value(&observations[0])?;
            let canonical = bijux_dna_core::contract::canonical::canonicalize_json_value(&json);
            serde_json::to_string(&canonical)?
        };
        assert_eq!(observations_text.trim(), expected_observation.trim());

        let expected_summary = {
            let json = serde_json::to_value(&summary)?;
            let canonical = bijux_dna_core::contract::canonical::canonicalize_json_value(&json);
            serde_json::to_string_pretty(&canonical)?
        };
        assert_eq!(summary_text.trim(), expected_summary.trim());

        let expected_decision = {
            let json = serde_json::to_value(&decision)?;
            let canonical = bijux_dna_core::contract::canonical::canonicalize_json_value(&json);
            serde_json::to_string_pretty(&canonical)?
        };
        assert_eq!(decision_text.trim(), expected_decision.trim());
        let decisions_value: serde_json::Value = serde_json::from_str(&decisions_text)?;
        assert_eq!(decisions_value.as_array().map(Vec::len), Some(1));

        let reloaded = read_observations_jsonl(&out_dir.join("observations.jsonl"))?;
        assert_eq!(reloaded.len(), 1);
        Ok(())
    }

    #[test]
    fn resume_observation_write_preserves_existing_rows() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("observations.jsonl");
        let existing = sample_observation("run-1", "r1");
        let incoming = sample_observation("run-2", "r2");

        write_observations_jsonl(&path, &[existing], WriteMode::Force)?;
        write_observations_jsonl(&path, &[incoming], WriteMode::Resume)?;

        let reloaded = read_observations_jsonl(&path)?;
        assert_eq!(reloaded.len(), 2);
        assert!(reloaded.iter().any(|obs| obs.run_id == "run-1"));
        assert!(reloaded.iter().any(|obs| obs.run_id == "run-2"));
        Ok(())
    }

    #[test]
    fn resume_observation_write_keeps_distinct_stage_branches() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("observations.jsonl");
        let mut branch_a = sample_observation("run-1", "r1");
        branch_a.stage_instance_id = Some("fastq.trim_reads.tool.fastp.a".to_string());
        branch_a.lineage_id = Some("branch-a".to_string());
        let mut branch_b = sample_observation("run-2", "r1");
        branch_b.stage_instance_id = Some("fastq.trim_reads.tool.fastp.b".to_string());
        branch_b.lineage_id = Some("branch-b".to_string());

        write_observations_jsonl(&path, &[branch_a], WriteMode::Force)?;
        write_observations_jsonl(&path, &[branch_b], WriteMode::Resume)?;

        let reloaded = read_observations_jsonl(&path)?;
        assert_eq!(reloaded.len(), 2);
        assert!(reloaded.iter().any(|obs| obs.lineage_id.as_deref() == Some("branch-a")));
        assert!(reloaded.iter().any(|obs| obs.lineage_id.as_deref() == Some("branch-b")));
        Ok(())
    }

    #[test]
    fn observation_reader_rejects_invalid_confounded_rows() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("observations.jsonl");
        let mut invalid = sample_observation("run-1", "r1");
        invalid.dataset_class.clear();
        let line = serde_json::to_string(&invalid)?;
        bijux_dna_runtime::recording::write_atomic_bytes(&path, line.as_bytes())?;

        let result = read_observations_jsonl(&path);

        assert!(result.is_err());
        let message = result.err().map(|err| err.to_string()).unwrap_or_default();
        assert!(message.contains("validate observation"));
        Ok(())
    }
}
