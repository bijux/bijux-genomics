//! Owner: bijux-bench
//! Bench orchestration helpers: summarize, gate, compare, run_suite.
#![allow(dead_code)]

use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

use crate::artifacts::{
    read_observations_jsonl, write_decision_json, write_observations_jsonl, write_summary_json,
    WriteMode,
};
use crate::compare::{compare_summaries, CompareReport};
use crate::contract::{validate_decision, validate_observation, validate_suite, validate_summary};
use crate::error::BenchError;
use crate::model::{
    BenchmarkObservation, BenchmarkSummary, MetricSummary, SummaryRow, SummaryStratum,
};
use crate::policy::{GateDecision, GatePolicy};
use crate::repo::RunRepository;
use crate::stats::{bootstrap_ci, mad_outliers, robust_stats, seed_from_ids};

#[derive(Debug, Clone)]
pub struct BenchRunOptions {
    pub output_dir: Option<std::path::PathBuf>,
    pub resume: bool,
    pub force: bool,
    pub ci_bootstrap: Option<usize>,
    pub objective: String,
}

/// Load observations for a suite from a repository.
///
/// # Errors
/// Returns an error if the repository cannot load observations.
pub fn load_suite(
    repo: &dyn RunRepository,
    run_ids: Option<&[String]>,
) -> Result<Vec<BenchmarkObservation>> {
    let ids = match run_ids {
        Some(ids) => ids.to_vec(),
        None => repo.list_runs()?,
    };
    let mut observations = Vec::new();
    for run_id in ids {
        observations.extend(repo.load_observations(&run_id)?);
    }
    Ok(observations)
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
    let mut scientifically_invalid = false;
    let mut invalid_reasons = Vec::new();
    for obs in observations {
        if let Err(err) = validate_observation(obs) {
            match err {
                BenchError::MissingConfounder { field } => {
                    scientifically_invalid = true;
                    invalid_reasons.push(format!("missing_confounder:{field}"));
                }
                BenchError::InvalidObservation { reason } => {
                    scientifically_invalid = true;
                    invalid_reasons.push(format!("invalid_observation:{reason}"));
                }
                other => return Err(other.into()),
            }
        }
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
        if options.ci_bootstrap.is_some() && runtimes.len() < 5 {
            warnings.push(format!("ci_min_n:runtime_s:{stage_id}:{tool_id}"));
        }
        let memory_ci = bootstrap_if_enabled(
            suite,
            &stage_id,
            &tool_id,
            "memory_mb",
            &memories,
            options.ci_bootstrap,
        );
        if options.ci_bootstrap.is_some() && memories.len() < 5 {
            warnings.push(format!("ci_min_n:memory_mb:{stage_id}:{tool_id}"));
        }

        let runtime_summary = MetricSummary {
            metric_id: "runtime_s".to_string(),
            n: runtimes.len(),
            stats: runtime_stats,
            ci_low: runtime_ci.map(|ci| ci.0),
            ci_high: runtime_ci.map(|ci| ci.1),
            outlier_count: runtime_outliers.outlier_count,
            outlier_replicates: indices_to_replicates(&runtime_outliers.outlier_indices, &group),
            practical_threshold: Some(0.05),
            power_warning: runtimes.len() < 5,
        };
        let memory_summary = MetricSummary {
            metric_id: "memory_mb".to_string(),
            n: memories.len(),
            stats: memory_stats,
            ci_low: memory_ci.map(|ci| ci.0),
            ci_high: memory_ci.map(|ci| ci.1),
            outlier_count: memory_outliers.outlier_count,
            outlier_replicates: indices_to_replicates(&memory_outliers.outlier_indices, &group),
            practical_threshold: Some(0.05),
            power_warning: memories.len() < 5,
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
            if options.ci_bootstrap.is_some() && values.len() < 5 {
                warnings.push(format!("ci_min_n:{metric_id}:{stage_id}:{tool_id}"));
            }
            metric_summaries.push(MetricSummary {
                metric_id,
                n: values.len(),
                stats,
                ci_low: ci.map(|c| c.0),
                ci_high: ci.map(|c| c.1),
                outlier_count: outliers.outlier_count,
                outlier_replicates: indices_to_replicates(&outliers.outlier_indices, &group),
                practical_threshold: Some(0.05),
                power_warning: values.len() < 5,
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
        if low_power {
            warnings.push(format!("low_power:{stage_id}:{tool_id}:{dataset_id}"));
        }
        let completeness = if group.is_empty() {
            0.0
        } else {
            n_effective as f64 / group.len() as f64
        };

        let first = group.first().copied();
        let (dataset_class, read_layout) = match first {
            Some(obs) => (obs.dataset_class.clone(), obs.read_layout.clone()),
            None => ("unknown".to_string(), "unknown".to_string()),
        };

        rows.push(SummaryRow {
            dataset_id,
            dataset_class,
            read_layout,
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
        (&a.dataset_id, &a.stage_id, &a.tool_id, &a.params_hash).cmp(&(
            &b.dataset_id,
            &b.stage_id,
            &b.tool_id,
            &b.params_hash,
        ))
    });
    let mut strata_map: BTreeMap<(String, String), (usize, usize)> = BTreeMap::new();
    for row in &rows {
        let entry = strata_map
            .entry((row.stage_id.clone(), row.dataset_class.clone()))
            .or_insert((0, 0));
        entry.0 += 1;
        if row.low_power {
            entry.1 += 1;
        }
    }
    let mut strata = Vec::new();
    for ((stage_id, dataset_class), (row_count, low_power_count)) in strata_map {
        strata.push(SummaryStratum {
            stage_id,
            dataset_class,
            row_count,
            low_power_count,
        });
    }
    let mut summary = BenchmarkSummary::v1(suite.suite_id.clone(), rows, strata, warnings);
    summary.scientifically_invalid = scientifically_invalid;
    summary.invalid_reasons = invalid_reasons;
    Ok(summary)
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
    if samples == 0 || values.len() < 5 {
        return None;
    }
    let seed = seed_from_ids(&suite.suite_id, metric_id, stage_id, tool_id);
    let result = bootstrap_ci(values, samples, seed);
    Some((result.ci_low, result.ci_high))
}

fn indices_to_replicates(indices: &[usize], group: &[&BenchmarkObservation]) -> Vec<String> {
    let mut replicates = Vec::new();
    for idx in indices {
        if let Some(obs) = group.get(*idx) {
            replicates.push(obs.replicate_id.clone());
        }
    }
    replicates
}

/// Gate summary rows using a policy.
#[must_use]
pub fn gate(policy: &GatePolicy, summary: &BenchmarkSummary) -> Vec<GateDecision> {
    let mut decisions = Vec::new();
    for row in &summary.rows {
        let mut metrics = BTreeMap::new();
        metrics.insert("runtime_s".to_string(), row.runtime.stats.median);
        metrics.insert("memory_mb".to_string(), row.memory.stats.median);
        for metric in &row.metrics {
            metrics.insert(metric.metric_id.clone(), metric.stats.median);
        }
        decisions.push(policy.decide(
            &row.dataset_id,
            &row.stage_id,
            &row.tool_id,
            &row.params_hash,
            &metrics,
        ));
    }
    decisions
}

/// Compare two summaries.
pub fn compare(
    summary_a: &BenchmarkSummary,
    summary_b: &BenchmarkSummary,
) -> Result<CompareReport> {
    compare_summaries(summary_a, summary_b)
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
) -> Result<(BenchmarkSummary, Vec<GateDecision>)> {
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
    let decisions = gate(policy, &summary);
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

#[cfg(test)]
mod tests {
    use super::{run_suite, BenchRunOptions};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use crate::model::{BenchmarkObservation, BenchmarkSuiteSpec};
    use crate::policy::GatePolicy;
    use crate::{
        AnalysisRequirements, DatasetSpec, DiversityRequirements, MetricsEnvelope, ReplicatePolicy,
        StratificationRequirement,
    };

    #[test]
    fn artifact_bundle_is_stable() -> anyhow::Result<()> {
        let suite = BenchmarkSuiteSpec::v1(
            "suite-bundle".to_string(),
            vec![DatasetSpec {
                id: "dataset-1".to_string(),
                hash: "hash-1".to_string(),
                size: 100,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            vec!["fastq.trim".to_string()],
            vec!["fastp".to_string()],
            vec!["params-a".to_string()],
            ReplicatePolicy {
                count: 3,
                warmup: 0,
                seeds: vec![1, 2, 3],
            },
            DiversityRequirements {
                min_dataset_count: 1,
                min_classes: 1,
                min_read_layouts: 1,
            },
            vec![StratificationRequirement {
                key: "dataset_class".to_string(),
                required_values: vec!["trueseq".to_string()],
            }],
            AnalysisRequirements {
                require_bootstrap: false,
                require_outlier_detection: true,
                min_replicates_for_bootstrap: 5,
            },
        );
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
            container_digest: "sha256:abc".to_string(),
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
            warmup_policy: "none".to_string(),
            seed_policy: "default".to_string(),
            runner: "docker".to_string(),
            platform: "linux".to_string(),
            cpu: "x86_64".to_string(),
            threads: 4,
            io_mode: "local".to_string(),
        };
        let policy = GatePolicy {
            objective: "balanced".to_string(),
            required_metrics: Vec::new(),
            thresholds: BTreeMap::new(),
            allowed_regressions: BTreeMap::new(),
            must_not_regress: Vec::new(),
            semantics_overrides: BTreeMap::new(),
            stage_overrides: BTreeMap::new(),
        };
        let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("bench_bundle");
        let options = BenchRunOptions {
            output_dir: Some(out_dir.clone()),
            resume: false,
            force: true,
            ci_bootstrap: None,
            objective: "balanced".to_string(),
        };
        let _ = run_suite(&suite, &[obs], &policy, &options)?;
        assert!(out_dir.join("observations.jsonl").exists());
        assert!(out_dir.join("summary.json").exists());
        assert!(out_dir.join("decision.json").exists());
        Ok(())
    }
}
