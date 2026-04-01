//! Owner: bijux-dna-bench
//! Suite-level orchestration that validates and persists benchmark outputs.

use std::collections::BTreeSet;

use anyhow::Result;

use crate::artifacts::{
    read_observations_jsonl, write_decision_json, write_observations_jsonl, write_summary_json,
    WriteMode,
};
use bijux_dna_bench_model::contract::{validate_decision, validate_summary};
use bijux_dna_bench_model::policy::{GateDecision, GatePolicy};
use bijux_dna_bench_model::{
    BenchError, BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary,
};

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
    write_suite_artifacts(&merged, &summary, &decisions, options)?;
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

    let out_dir = options
        .output_dir
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("resume requires output_dir"))?;
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

fn write_suite_artifacts(
    observations: &[BenchmarkObservation],
    summary: &BenchmarkSummary,
    decisions: &[GateDecision],
    options: &BenchRunOptions,
) -> Result<()> {
    let Some(out_dir) = &options.output_dir else {
        return Ok(());
    };

    let mode = if options.force {
        WriteMode::Force
    } else if options.resume {
        WriteMode::Resume
    } else {
        WriteMode::Force
    };
    write_observations_jsonl(&out_dir.join("observations.jsonl"), observations, mode)
        .map_err(|err: anyhow::Error| BenchError::ArtifactWriteError(err.to_string()))?;
    write_summary_json(&out_dir.join("summary.json"), summary)
        .map_err(|err: anyhow::Error| BenchError::ArtifactWriteError(err.to_string()))?;
    if let Some(decision) = decisions.first() {
        write_decision_json(&out_dir.join("decision.json"), decision)
            .map_err(|err: anyhow::Error| BenchError::ArtifactWriteError(err.to_string()))?;
    }
    Ok(())
}

fn observation_identity(
    obs: &BenchmarkObservation,
) -> (String, String, Option<String>, Option<String>, String, String, String) {
    (
        obs.dataset_id.clone(),
        obs.stage_id.clone(),
        obs.stage_instance_id.clone(),
        obs.lineage_id.clone(),
        obs.tool_id.clone(),
        obs.params_hash.clone(),
        obs.replicate_id.clone(),
    )
}
