//! Owner: bijux-dna-bench
//! Bench summary assembly from validated benchmark observations.
#![allow(dead_code)]

mod grouping;
mod row_metrics;
mod strata;

use anyhow::Result;

use self::grouping::collect_summary_groups;
use self::row_metrics::build_summary_row;
use self::strata::build_summary_strata;
use super::options::BenchRunOptions;
use super::summary_fairness::evaluate_summary_fairness;
use bijux_dna_bench_model::contract::validate_suite;
use bijux_dna_bench_model::{BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary};

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

    let groups = collect_summary_groups(observations);

    let mut rows = Vec::new();
    for ((dataset_id, stage_id, stage_instance_id, lineage_id, tool_id, params_hash), group) in
        groups
    {
        let Some((row, row_warnings)) = build_summary_row(
            suite,
            options,
            dataset_id,
            stage_id,
            stage_instance_id,
            lineage_id,
            tool_id,
            params_hash,
            group,
        )?
        else {
            continue;
        };
        warnings.extend(row_warnings);
        rows.push(row);
    }
    let strata = build_summary_strata(&mut rows);
    let mut summary = BenchmarkSummary::v1(suite.suite_id.clone(), rows, strata, warnings);
    summary.scientifically_invalid = fairness.scientifically_invalid;
    summary.invalid_reasons = fairness.invalid_reasons;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::{summarize, BenchRunOptions};
    use crate::workflow::run_suite::run_suite;
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
            ReplicatePolicy { count: 3, warmup: 0, seeds: vec![1, 2, 3] },
            DiversityRequirements { min_dataset_count: 1, min_classes: 1, min_read_layouts: 1 },
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
        assert!(out_dir.join("decisions.json").exists());
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
            ReplicatePolicy { count: 1, warmup: 0, seeds: vec![1] },
            DiversityRequirements { min_dataset_count: 1, min_classes: 1, min_read_layouts: 1 },
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
