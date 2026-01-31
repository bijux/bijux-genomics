//! Owner: bijux-bench
//! Bench orchestration helpers: summarize, gate, compare, run_suite.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::Result;

use crate::artifacts::{
    read_observations_jsonl, write_decision_json, write_observations_jsonl, write_summary_json,
    WriteMode,
};
use crate::compare::{compare_runs, CompareReport};
use crate::contract::{validate_decision, validate_observation, validate_summary, validate_suite};
use crate::error::BenchError;
use crate::model::{
    BenchmarkDecision, BenchmarkObservation, BenchmarkSummary, DecisionRationale, MetricSummary,
    SummaryRow,
};
use crate::policy::GatePolicy;
use crate::stats::{bootstrap_ci, mad_outliers, robust_stats, seed_from_ids};

#[derive(Debug, Clone)]
pub struct BenchRunOptions {
    pub output_dir: Option<std::path::PathBuf>,
    pub resume: bool,
    pub force: bool,
    pub ci_bootstrap: Option<usize>,
    pub objective: String,
}

impl Default for BenchRunOptions {
    fn default() -> Self {
        Self {
            output_dir: None,
            resume: false,
            force: false,
            ci_bootstrap: None,
            objective: "balanced".to_string(),
        }
    }
}

/// Summarize observations into a benchmark summary.
///
/// # Errors
/// Returns an error if observations violate contract.
pub fn summarize(
    suite: &crate::model::BenchmarkSuiteSpec,
    observations: &[BenchmarkObservation],
    options: &BenchRunOptions,
) -> Result<BenchmarkSummary> {
    validate_suite(suite)?;
    for obs in observations {
        validate_observation(obs)?;
    }

    let mut warnings = Vec::new();
    if suite.replicate_policy.count < 3 {
        warnings.push("low_power".to_string());
    }

    let mut groups: BTreeMap<(String, String, String, String), Vec<&BenchmarkObservation>> =
        BTreeMap::new();
    for obs in observations {
        groups
            .entry((
                obs.dataset_id.clone(),
                obs.stage_id.clone(),
                obs.tool_id.clone(),
                obs.params_hash.clone(),
            ))
            .or_default()
            .push(obs);
    }

    let mut rows = Vec::new();
    for ((dataset_id, stage_id, tool_id, params_hash), group) in groups {
        let runtimes: Vec<f64> = group.iter().map(|o| o.runtime_s).collect();
        let memories: Vec<f64> = group.iter().map(|o| o.memory_mb).collect();

        let runtime_stats = robust_stats(&runtimes);
        let memory_stats = robust_stats(&memories);

        let runtime_outliers = mad_outliers(&runtimes, 3.5);
        let memory_outliers = mad_outliers(&memories, 3.5);

        let runtime_ci = bootstrap_if_enabled(
            suite,
            &stage_id,
            &tool_id,
            "runtime_s",
            &runtimes,
            options.ci_bootstrap,
        );
        let memory_ci = bootstrap_if_enabled(
            suite,
            &stage_id,
            &tool_id,
            "memory_mb",
            &memories,
            options.ci_bootstrap,
        );

        let runtime_summary = MetricSummary {
            metric_id: "runtime_s".to_string(),
            stats: runtime_stats,
            ci_low: runtime_ci.map(|ci| ci.0),
            ci_high: runtime_ci.map(|ci| ci.1),
            outlier_count: runtime_outliers.outlier_count,
            outlier_replicates: indices_to_replicates(&runtime_outliers.outlier_indices, &group),
            practical_threshold: Some(0.05),
        };
        let memory_summary = MetricSummary {
            metric_id: "memory_mb".to_string(),
            stats: memory_stats,
            ci_low: memory_ci.map(|ci| ci.0),
            ci_high: memory_ci.map(|ci| ci.1),
            outlier_count: memory_outliers.outlier_count,
            outlier_replicates: indices_to_replicates(&memory_outliers.outlier_indices, &group),
            practical_threshold: Some(0.05),
        };

        let mut metric_summaries = Vec::new();
        let mut metric_ids = BTreeSet::new();
        for obs in &group {
            metric_ids.extend(obs.metrics.values.keys().cloned());
        }
        for metric_id in metric_ids {
            let values: Vec<f64> = group
                .iter()
                .filter_map(|obs| obs.metrics.values.get(&metric_id).copied())
                .collect();
            let stats = robust_stats(&values);
            let outliers = mad_outliers(&values, 3.5);
            let ci = bootstrap_if_enabled(
                suite,
                &stage_id,
                &tool_id,
                &metric_id,
                &values,
                options.ci_bootstrap,
            );
            metric_summaries.push(MetricSummary {
                metric_id,
                stats,
                ci_low: ci.map(|c| c.0),
                ci_high: ci.map(|c| c.1),
                outlier_count: outliers.outlier_count,
                outlier_replicates: indices_to_replicates(&outliers.outlier_indices, &group),
                practical_threshold: Some(0.05),
            });
        }

        let failures = group.iter().filter(|obs| obs.exit_code != 0).count();
        let failure_rate = if group.is_empty() {
            0.0
        } else {
            failures as f64 / group.len() as f64
        };
        let n_effective = group.len().saturating_sub(failures);
        let low_power = n_effective < 3;
        let completeness = if group.is_empty() {
            0.0
        } else {
            n_effective as f64 / group.len() as f64
        };

        rows.push(SummaryRow {
            dataset_id,
            stage_id,
            tool_id,
            params_hash,
            runtime: runtime_summary,
            memory: memory_summary,
            metrics: metric_summaries,
            failure_rate,
            completeness,
            n_effective,
            low_power,
        });
    }
    rows.sort_by(|a, b| {
        (
            &a.dataset_id,
            &a.stage_id,
            &a.tool_id,
            &a.params_hash,
        )
            .cmp(&(
                &b.dataset_id,
                &b.stage_id,
                &b.tool_id,
                &b.params_hash,
            ))
    });
    Ok(BenchmarkSummary::v1(
        suite.suite_id.clone(),
        rows,
        warnings,
    ))
}

fn bootstrap_if_enabled(
    suite: &crate::model::BenchmarkSuiteSpec,
    stage_id: &str,
    tool_id: &str,
    metric_id: &str,
    values: &[f64],
    samples: Option<usize>,
) -> Option<(f64, f64)> {
    let samples = samples.unwrap_or(0);
    if samples == 0 || values.is_empty() {
        return None;
    }
    let seed = seed_from_ids(&suite.suite_id, metric_id, stage_id, tool_id);
    let result = bootstrap_ci(values, samples, seed);
    Some((result.ci_low, result.ci_high))
}

fn indices_to_replicates(
    indices: &[usize],
    group: &[&BenchmarkObservation],
) -> Vec<String> {
    let mut replicates = Vec::new();
    for idx in indices {
        if let Some(obs) = group.get(*idx) {
            replicates.push(obs.replicate_id.clone());
        }
    }
    replicates
}

/// Gate observations using a policy.
#[must_use]
pub fn gate(policy: &GatePolicy, obs: &BenchmarkObservation) -> BenchmarkDecision {
    gate_with_objective(policy, obs, "balanced")
}

/// Gate observations using a policy with an explicit objective.
#[must_use]
pub fn gate_with_objective(
    policy: &GatePolicy,
    obs: &BenchmarkObservation,
    objective: &str,
) -> BenchmarkDecision {
    let decision = policy.decide(Some(&obs.stage_id), &obs.metrics.values);
    let rationale = decision
        .rationale_trace
        .iter()
        .map(|note| DecisionRationale {
            metric_id: obs.stage_id.clone(),
            observed: obs.runtime_s,
            direction: "unknown".to_string(),
            note: note.clone(),
        })
        .collect();
    BenchmarkDecision::v1(
        obs.stage_id.clone(),
        obs.tool_id.clone(),
        objective.to_string(),
        decision.passes,
        rationale,
        decision.missing_metrics,
    )
}

/// Compare two runs via run_index.
pub fn compare(
    run_a: &str,
    run_b: &str,
    index_path: &Path,
    artifacts_root: &Path,
) -> Result<CompareReport> {
    compare_runs(run_a, run_b, index_path, artifacts_root)
}

/// Run a suite: summarize, gate, and write artifacts.
///
/// # Errors
/// Returns an error if contracts fail or artifacts cannot be written.
pub fn run_suite(
    suite: &crate::model::BenchmarkSuiteSpec,
    observations: &[BenchmarkObservation],
    policy: &GatePolicy,
    options: &BenchRunOptions,
) -> Result<(BenchmarkSummary, Vec<BenchmarkDecision>)> {
    let mut merged = observations.to_vec();
    if options.resume {
        let out_dir = options
            .output_dir
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("resume requires output_dir"))?;
        let path = out_dir.join("observations.jsonl");
        if path.exists() {
            let existing = read_observations_jsonl(&path)?;
            let mut seen = std::collections::BTreeSet::new();
            for obs in merged.iter() {
                seen.insert((
                    obs.dataset_id.clone(),
                    obs.stage_id.clone(),
                    obs.tool_id.clone(),
                    obs.params_hash.clone(),
                    obs.replicate_id.clone(),
                ));
            }
            for obs in existing {
                let key = (
                    obs.dataset_id.clone(),
                    obs.stage_id.clone(),
                    obs.tool_id.clone(),
                    obs.params_hash.clone(),
                    obs.replicate_id.clone(),
                );
                if !seen.contains(&key) {
                    merged.push(obs);
                }
            }
        }
    }

    let summary = summarize(suite, &merged, options)?;
    validate_summary(&summary)?;
    let mut decisions = Vec::new();
    for obs in &merged {
        decisions.push(gate_with_objective(policy, obs, &options.objective));
    }
    for decision in &decisions {
        validate_decision(decision)?;
    }
    if let Some(out_dir) = &options.output_dir {
        let mode = if options.force {
            WriteMode::Force
        } else if options.resume {
            WriteMode::Resume
        } else {
            WriteMode::Force
        };
        write_observations_jsonl(&out_dir.join("observations.jsonl"), &merged, mode)
            .map_err(|err| BenchError::ArtifactWriteError(err.to_string()))?;
        write_summary_json(&out_dir.join("summary.json"), &summary)
            .map_err(|err| BenchError::ArtifactWriteError(err.to_string()))?;
        if let Some(decision) = decisions.first() {
            write_decision_json(&out_dir.join("decision.json"), decision)
                .map_err(|err| BenchError::ArtifactWriteError(err.to_string()))?;
        }
    }
    Ok((summary, decisions))
}
