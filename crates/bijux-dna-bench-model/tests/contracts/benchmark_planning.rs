use std::collections::BTreeMap;

use bijux_dna_bench_model::contract::{
    validate_benchmark_portfolio, validate_portfolio_campaign_alignment,
    validate_stage_campaign_registry, BENCHMARK_PORTFOLIO_SCHEMA_V1,
    BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,
};
use bijux_dna_bench_model::{
    BenchmarkCachePolicy, BenchmarkComparisonMetricTolerance, BenchmarkCorpusContract,
    BenchmarkCorpusStatus, BenchmarkEvidenceScope, BenchmarkEvidenceTier,
    BenchmarkExecutionRuntime, BenchmarkExecutionScheduler, BenchmarkMeasurementContract,
    BenchmarkPortfolio, BenchmarkProfileAssignment, BenchmarkScientificProfile,
    BenchmarkStageCampaignRegistry, BenchmarkStageCampaignSpec,
};

fn stage_campaign_registry() -> BenchmarkStageCampaignRegistry {
    BenchmarkStageCampaignRegistry {
        schema_version: BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1.to_string(),
        campaigns: vec![BenchmarkStageCampaignSpec {
            run_id: "run-2042".to_string(),
            slug: "fastq-profile-read-lengths".to_string(),
            domain: "fastq".to_string(),
            stage_id: "fastq.profile_read_lengths".to_string(),
            evidence_tier: BenchmarkEvidenceTier::Comparative,
            profile_id: "fastq-observational-qc".to_string(),
            corpus_ids: vec!["human-short-read-diversity".to_string()],
            scientific_question: "Does the governed cohort preserve exact read lengths?"
                .to_string(),
            comparison_scope: "full cohort exact read-length comparison".to_string(),
            tool_ids: vec!["seqfu".to_string(), "seqkit".to_string()],
            corpus_catalog_ids: vec!["human-short-read-diversity".to_string()],
            reference_catalog_ids: Vec::new(),
            database_catalog_ids: Vec::new(),
            input_size_keys: vec!["fastq_input_bytes".to_string()],
            result_artifact: "read-lengths.json".to_string(),
            runtime: BenchmarkExecutionRuntime::Apptainer,
            scheduler: BenchmarkExecutionScheduler::Slurm,
            container_only: true,
            threads: None,
            threads_by_tool: BTreeMap::from([("seqfu".to_string(), 1), ("seqkit".to_string(), 4)]),
            warmup_repetitions: 1,
            measured_repetitions: 3,
            cache_policy: BenchmarkCachePolicy::Warm,
            result_root_template: "results/benchmarks/{run_id}-{slug}/{tool_id}".to_string(),
            accuracy_claim: "exact_per_record_length_concordance".to_string(),
            interpretation: "Exact agreement is required for qualification.".to_string(),
            performance_metrics: vec![
                "wall_seconds".to_string(),
                "cpu_seconds".to_string(),
                "cpu_utilization_percent".to_string(),
                "peak_rss_kib".to_string(),
                "filesystem_input_operations".to_string(),
                "filesystem_output_operations".to_string(),
                "input_bytes".to_string(),
                "output_bytes".to_string(),
                "throughput_input_bytes_per_second".to_string(),
            ],
            scientific_metrics: vec!["record_count".to_string(), "mean_read_length".to_string()],
            comparison_metrics: vec![BenchmarkComparisonMetricTolerance {
                id: "mean_read_length".to_string(),
                absolute_tolerance: 0.0,
                relative_tolerance: 0.0,
            }],
            limitations: vec![
                "The cohort is operational, not population representative.".to_string()
            ],
        }],
    }
}

fn benchmark_portfolio() -> BenchmarkPortfolio {
    BenchmarkPortfolio {
        schema_version: BENCHMARK_PORTFOLIO_SCHEMA_V1.to_string(),
        measurement: BenchmarkMeasurementContract {
            warmup_repetitions: 1,
            measured_repetitions: 3,
            required_resource_metrics: vec![
                "wall_seconds".to_string(),
                "cpu_seconds".to_string(),
                "cpu_utilization_percent".to_string(),
                "peak_rss_kib".to_string(),
                "filesystem_input_operations".to_string(),
                "filesystem_output_operations".to_string(),
                "input_bytes".to_string(),
                "output_bytes".to_string(),
                "throughput_input_bytes_per_second".to_string(),
            ],
            required_quality_dimensions: vec![
                "scientific_correctness".to_string(),
                "repeatability".to_string(),
                "robustness".to_string(),
                "output_contract".to_string(),
            ],
        },
        stage_runs: BTreeMap::from([(
            "fastq.profile_read_lengths".to_string(),
            "run-2042".to_string(),
        )]),
        corpora: vec![BenchmarkCorpusContract {
            id: "human-short-read-diversity".to_string(),
            status: BenchmarkCorpusStatus::Admitted,
            evidence_scope: BenchmarkEvidenceScope::Comparative,
            real_data: true,
            sample_count: 20,
            minimum_sample_count: Some(10),
            data_types: vec!["fastq".to_string()],
            roles: vec!["short_read_diversity".to_string()],
            source_specs: vec!["benchmarks/corpora/human-short-read-diversity.toml".to_string()],
            catalog_ids: vec!["human-short-read-diversity".to_string()],
            truth_strategy: "observational_only".to_string(),
            scientific_rationale: "The cohort spans multiple real library layouts.".to_string(),
            limitations: vec!["The cohort is not population representative.".to_string()],
        }],
        profiles: vec![BenchmarkScientificProfile {
            id: "fastq-observational-qc".to_string(),
            required_corpus_roles: vec!["short_read_diversity".to_string()],
            minimum_real_samples: 10,
            accuracy_strategy: "cross_tool_consistency".to_string(),
            interchangeability: "canonical_report_contract".to_string(),
            downstream_probe: "consume every tool report through the governed QC surface"
                .to_string(),
        }],
        assignments: vec![BenchmarkProfileAssignment {
            profile_id: "fastq-observational-qc".to_string(),
            stage_ids: vec!["fastq.profile_read_lengths".to_string()],
        }],
    }
}

#[test]
fn benchmark_planning_contracts_accept_aligned_registry_and_portfolio() -> anyhow::Result<()> {
    let registry = stage_campaign_registry();
    let portfolio = benchmark_portfolio();

    validate_stage_campaign_registry(&registry)?;
    validate_benchmark_portfolio(&portfolio)?;
    validate_portfolio_campaign_alignment(&registry, &portfolio)?;
    Ok(())
}

#[test]
fn benchmark_stage_campaign_registry_rejects_slug_drift() {
    let mut registry = stage_campaign_registry();
    registry.campaigns[0].slug = "wrong-slug".to_string();

    let error = validate_stage_campaign_registry(&registry)
        .expect_err("slug drift should fail benchmark campaign validation");

    assert!(error.to_string().contains("must derive from stage"));
}

#[test]
fn benchmark_planning_alignment_rejects_run_id_drift() {
    let registry = stage_campaign_registry();
    let mut portfolio = benchmark_portfolio();
    portfolio.stage_runs.insert("fastq.profile_read_lengths".to_string(), "run-2043".to_string());

    let error = validate_portfolio_campaign_alignment(&registry, &portfolio)
        .expect_err("run-id drift should fail planning alignment");

    assert!(error.to_string().contains("benchmark run id drift"));
}
