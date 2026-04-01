//! Owner: bijux-dna-bench
//! Bench orchestration helpers: summarize, gate, compare, run_suite.
#![allow(dead_code)]

mod evaluation;
mod options;
mod run_suite;
mod suite_load;
mod summary_fairness;
mod summary_scope;
mod summary_statistics;

use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

use bijux_dna_bench_model::contract::validate_suite;
use bijux_dna_bench_model::stats::{mad_outliers, robust_stats};
use bijux_dna_bench_model::{
    BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary, MetricSummary, SummaryRow,
    SummaryStratum,
};
use summary_fairness::evaluate_summary_fairness;
use summary_scope::{SummaryGroupKey, SummaryStratumKey};
use summary_statistics::{bootstrap_if_enabled, indices_to_replicates};

pub use evaluation::{compare, gate};
pub use options::BenchRunOptions;
pub use suite_load::load_suite;

use self::run_suite::run_suite;

/// Summarize observations into a benchmark summary.
///
/// # Errors
/// Returns an error if observations violate contract.
pub fn summarize(
    suite: &BenchmarkSuiteSpec,
    observations: &[BenchmarkObservation],
    options: &BenchRunOptions,
) -> Result<BenchmarkSummary> {
    validate_suite(suite)?;
    let fairness = evaluate_summary_fairness(suite, observations)?;
    let mut warnings = fairness.warnings;

    let mut groups: BTreeMap<SummaryGroupKey, Vec<&BenchmarkObservation>> = BTreeMap::new();
    for obs in observations {
        groups
            .entry((
                obs.dataset_id.clone(),
                obs.stage_id.clone(),
                obs.stage_instance_id.clone(),
                obs.lineage_id.clone(),
                obs.tool_id.clone(),
                obs.params_hash.clone(),
            ))
            .or_default()
            .push(obs);
    }

    let mut rows = Vec::new();
    for ((dataset_id, stage_id, stage_instance_id, lineage_id, tool_id, params_hash), group) in
        groups
    {
        let tool = tool_id.as_str();
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
            let warning = "ci_min_n:runtime_s";
            warnings.push(format!("{warning}:{stage_id}:{tool}"));
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
            let warning = "ci_min_n:memory_mb";
            warnings.push(format!("{warning}:{stage_id}:{tool}"));
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
                let warning = "ci_min_n";
                warnings.push(format!("{warning}:{metric_id}:{stage_id}:{tool}"));
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
            let warning = "low_power";
            warnings.push(format!("{warning}:{stage_id}:{tool}:{dataset_id}"));
        }
        let completeness = if group.is_empty() {
            0.0
        } else {
            n_effective as f64 / group.len() as f64
        };

        let Some(first) = group.first().copied() else {
            continue;
        };
        let dataset_class = first.dataset_class.clone();
        let read_layout = first.read_layout.clone();

        rows.push(SummaryRow {
            dataset_id,
            dataset_class,
            read_layout,
            stage_id,
            stage_instance_id,
            lineage_id,
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
            &a.stage_instance_id,
            &a.lineage_id,
            &a.tool_id,
            &a.params_hash,
        )
            .cmp(&(
                &b.dataset_id,
                &b.stage_id,
                &b.stage_instance_id,
                &b.lineage_id,
                &b.tool_id,
                &b.params_hash,
            ))
    });
    let mut strata_map: BTreeMap<SummaryStratumKey, (usize, usize)> = BTreeMap::new();
    for row in &rows {
        let entry = strata_map
            .entry((
                row.stage_id.clone(),
                row.stage_instance_id.clone(),
                row.lineage_id.clone(),
                row.dataset_class.clone(),
            ))
            .or_insert((0, 0));
        entry.0 += 1;
        if row.low_power {
            entry.1 += 1;
        }
    }
    let mut strata = Vec::new();
    for ((stage_id, stage_instance_id, lineage_id, dataset_class), (row_count, low_power_count)) in
        strata_map
    {
        strata.push(SummaryStratum {
            stage_id,
            stage_instance_id,
            lineage_id,
            dataset_class,
            row_count,
            low_power_count,
        });
    }
    let mut summary = BenchmarkSummary::v1(suite.suite_id.clone(), rows, strata, warnings);
    summary.scientifically_invalid = fairness.scientifically_invalid;
    summary.invalid_reasons = fairness.invalid_reasons;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::{run_suite, summarize, BenchRunOptions};
    use std::collections::BTreeMap;

    use bijux_dna_bench_model::GatePolicy;
    use bijux_dna_bench_model::{
        AnalysisRequirements, BenchmarkObservation, BenchmarkSuiteSpec, DatasetSpec,
        DiversityRequirements, MetricsEnvelope, ReplicatePolicy, StratificationRequirement,
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
            &["fastq.trim_reads".to_string()],
            &["fastp".to_string()],
            &["params-a".to_string()],
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
        let policy = GatePolicy {
            objective: "balanced".to_string(),
            required_metrics: Vec::new(),
            thresholds: BTreeMap::new(),
            allowed_regressions: BTreeMap::new(),
            must_not_regress: Vec::new(),
            semantics_overrides: BTreeMap::new(),
            stage_overrides: BTreeMap::new(),
        };
        let temp = tempfile::tempdir()?;
        let out_dir = temp.path().join("bench_bundle");
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

    #[test]
    fn summary_keeps_distinct_stage_instances_separate() -> anyhow::Result<()> {
        let suite = BenchmarkSuiteSpec::v1(
            "suite-branching".to_string(),
            vec![DatasetSpec {
                id: "dataset-1".to_string(),
                hash: "hash-1".to_string(),
                size: 100,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            &["fastq.trim_reads".to_string()],
            &["fastp".to_string()],
            &["params-a".to_string()],
            ReplicatePolicy {
                count: 1,
                warmup: 0,
                seeds: vec![1],
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
                require_outlier_detection: false,
                min_replicates_for_bootstrap: 5,
            },
        );
        let base = BenchmarkObservation {
            schema_version: "bijux.bench.observation.v1".to_string(),
            run_id: "run-1".to_string(),
            dataset_id: "dataset-1".to_string(),
            dataset_class: "trueseq".to_string(),
            read_layout: "paired".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: None,
            lineage_id: Some("branch-a".to_string()),
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
        let other = BenchmarkObservation {
            run_id: "run-2".to_string(),
            stage_instance_id: Some("fastq.trim_reads.tool.fastp.alt".to_string()),
            lineage_id: Some("branch-b".to_string()),
            ..base.clone()
        };

        let summary = summarize(&suite, &[base, other], &BenchRunOptions::default())?;
        assert_eq!(summary.rows.len(), 2);
        assert_ne!(summary.rows[0].lineage_id, summary.rows[1].lineage_id);
        Ok(())
    }
}
