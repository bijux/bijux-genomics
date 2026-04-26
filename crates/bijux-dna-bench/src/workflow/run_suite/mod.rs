//! Owner: bijux-dna-bench
//! Suite-level orchestration that validates and persists benchmark outputs.

mod persistence;

use std::collections::BTreeSet;

use anyhow::Result;

use crate::artifacts::read_observations_jsonl;
use bijux_dna_bench_model::contract::{validate_decision, validate_summary};
use bijux_dna_bench_model::policy::{GateDecision, GatePolicy};
use bijux_dna_bench_model::{BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary};

use super::{gate, summarize, BenchRunOptions};

/// Run a suite: summarize, gate, and write artifacts.
///
/// # Errors
/// Returns an error if contracts fail or artifacts cannot be written.
pub fn run_suite(
    suite: &BenchmarkSuiteSpec,
    observations: &[BenchmarkObservation],
    policy: &GatePolicy,
    options: &BenchRunOptions,
) -> Result<(BenchmarkSummary, Vec<GateDecision>)> {
    let merged = merge_observations(observations, options)?;
    let summary = summarize(suite, &merged, options)?;
    validate_summary(&summary)?;
    let decisions = gate(policy, &summary);
    for decision in &decisions {
        validate_decision(decision)?;
    }
    persistence::write_suite_artifacts(&merged, &summary, &decisions, options)?;
    Ok((summary, decisions))
}

fn merge_observations(
    observations: &[BenchmarkObservation],
    options: &BenchRunOptions,
) -> Result<Vec<BenchmarkObservation>> {
    let mut merged = observations.to_vec();
    if !options.resume {
        return Ok(merged);
    }

    let out_dir =
        options.output_dir.as_ref().ok_or_else(|| anyhow::anyhow!("resume requires output_dir"))?;
    let path = out_dir.join("observations.jsonl");
    if !path.exists() {
        return Ok(merged);
    }

    let existing = read_observations_jsonl(&path)?;
    let mut seen = BTreeSet::new();
    for obs in &merged {
        seen.insert(observation_identity(obs));
    }
    for obs in existing {
        let key = observation_identity(&obs);
        if !seen.contains(&key) {
            merged.push(obs);
        }
    }
    Ok(merged)
}

fn observation_identity(
    obs: &BenchmarkObservation,
) -> (String, String, Option<String>, Option<String>, String, String, String, u32) {
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_bench_model::{BenchmarkObservation, MetricsEnvelope};

    use super::{merge_observations, BenchRunOptions};

    fn sample_observation(replicate_index: u32) -> BenchmarkObservation {
        BenchmarkObservation {
            schema_version: "bijux.bench.observation.v1".to_string(),
            run_id: format!("run-{replicate_index}"),
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
            replicate_index,
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
    fn merge_observations_keeps_distinct_replicate_indices() -> anyhow::Result<()> {
        let merged = merge_observations(
            &[sample_observation(0), sample_observation(1)],
            &BenchRunOptions::default(),
        )?;

        assert_eq!(merged.len(), 2);
        assert!(merged.iter().any(|obs| obs.replicate_index == 0));
        assert!(merged.iter().any(|obs| obs.replicate_index == 1));
        Ok(())
    }
}
