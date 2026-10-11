use std::collections::BTreeMap;

use bijux_dna_bench_model::contract::{
    validate_benchmark_local_hpc_job_graph, validate_benchmark_portfolio,
    validate_benchmark_publication_registry, validate_executable_stage_campaign_registry,
    validate_portfolio_campaign_alignment, validate_scientific_readiness,
    validate_stage_campaign_registry, validate_stage_coverage, validate_stage_universe,
    BENCHMARK_PORTFOLIO_SCHEMA_V2, BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,
};
use bijux_dna_bench_model::{
    BenchmarkCachePolicy, BenchmarkCampaignMaterialDriver, BenchmarkComparisonMetricTolerance,
    BenchmarkCorpusClass, BenchmarkCorpusContract, BenchmarkCorpusStatus,
    BenchmarkCurrentPublication, BenchmarkDisposition, BenchmarkEvidenceScope,
    BenchmarkEvidenceTier, BenchmarkExecutionRuntime, BenchmarkExecutionScheduler,
    BenchmarkLocalHpcJobGraph, BenchmarkLocalHpcJobGraphNode, BenchmarkMeasurementContract,
    BenchmarkPortfolio, BenchmarkProfileAssignment, BenchmarkPublicationRegistry,
    BenchmarkPublicationRegistrySummary, BenchmarkQuarantinedPublicationSurface,
    BenchmarkRunReportDisposition, BenchmarkRunReportIndexEntry, BenchmarkScientificProfile,
    BenchmarkScientificReadinessCorpus, BenchmarkScientificReadinessDomain,
    BenchmarkScientificReadinessReport, BenchmarkScientificReadinessStage,
    BenchmarkScientificReadinessStatus, BenchmarkScientificReadinessSummary,
    BenchmarkStageCampaignRegistry, BenchmarkStageCampaignSpec, BenchmarkStageCoverage,
    BenchmarkStageCoverageRow, BenchmarkStageCoverageSummary, BenchmarkStageDefinitionStatus,
    BenchmarkStageExecutionStatus, BenchmarkStageInterfaceStatus, BenchmarkStagePublication,
    BenchmarkStagePublicationDisposition, BenchmarkStageUniverse, BenchmarkStageUniverseRow,
    BenchmarkStageUniverseSummary, BENCHMARK_LOCAL_HPC_JOB_GRAPH_SCHEMA_V1,
};

const TEST_SHA256: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TEST_COMMIT: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[test]
fn benchmark_publication_registry_rejects_summary_drift() -> anyhow::Result<()> {
    let mut registry = BenchmarkPublicationRegistry::v2(
        BenchmarkPublicationRegistrySummary {
            current_product_count: 1,
            current_run_report_count: 0,
            missing_run_report_count: 1,
            stale_run_report_count: 0,
            failed_run_report_count: 0,
            superseded_run_report_count: 0,
            active_stage_report_count: 0,
            historical_stage_report_count: 1,
            quarantined_surface_count: 1,
            quarantined_file_count: 3,
        },
        vec![BenchmarkCurrentPublication {
            product_id: "coverage".into(),
            json_path: "benchmarks/reports/coverage.json".into(),
            json_sha256: TEST_SHA256.into(),
            markdown_path: "benchmarks/reports/coverage.md".into(),
            markdown_sha256: TEST_SHA256.into(),
        }],
        vec![BenchmarkRunReportIndexEntry {
            run_id: "run-2001".into(),
            stage_id: "bam.genotyping".into(),
            disposition: BenchmarkRunReportDisposition::Missing,
            report: None,
            publication_receipt: None,
            limitations: vec!["No admitted publication exists.".into()],
        }],
        vec![BenchmarkStagePublication {
            run_id: "run-2001".into(),
            stage_id: "bam.genotyping".into(),
            source_commit: TEST_COMMIT.into(),
            disposition: BenchmarkStagePublicationDisposition::Historical,
            json_path: format!("benchmarks/reports/historical/{TEST_COMMIT}/run-2001.json"),
            json_sha256: TEST_SHA256.into(),
            markdown_path: format!("benchmarks/reports/historical/{TEST_COMMIT}/run-2001.md"),
            markdown_sha256: TEST_SHA256.into(),
        }],
        vec![BenchmarkQuarantinedPublicationSurface {
            path_prefix: "benchmarks/readiness/".into(),
            disposition: "quarantined_legacy_readiness".into(),
            tracked_file_count: 3,
            tree_sha256: TEST_SHA256.into(),
            authority_policy: "migration evidence only".into(),
        }],
    );
    validate_benchmark_publication_registry(&registry)?;
    let mut invalid_missing = registry.clone();
    invalid_missing.run_reports[0].limitations.clear();
    assert!(validate_benchmark_publication_registry(&invalid_missing).is_err());
    let mut invalid_current = registry.clone();
    invalid_current.summary.current_run_report_count = 1;
    invalid_current.summary.missing_run_report_count = 0;
    invalid_current.run_reports[0].disposition = BenchmarkRunReportDisposition::Current;
    assert!(validate_benchmark_publication_registry(&invalid_current).is_err());
    registry.summary.historical_stage_report_count = 0;
    assert!(validate_benchmark_publication_registry(&registry).is_err());
    Ok(())
}

fn stage_campaign_registry() -> BenchmarkStageCampaignRegistry {
    BenchmarkStageCampaignRegistry {
        schema_version: BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1.to_string(),
        campaigns: vec![BenchmarkStageCampaignSpec {
            run_id: "run-2042".to_string(),
            slug: "fastq-profile-read-lengths".to_string(),
            domain: "fastq".to_string(),
            stage_id: "fastq.profile_read_lengths".to_string(),
            material_driver: Some(BenchmarkCampaignMaterialDriver::FastqProfileReadLengths),
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
        schema_version: BENCHMARK_PORTFOLIO_SCHEMA_V2.to_string(),
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
            class: BenchmarkCorpusClass::ProductionReal,
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
    validate_executable_stage_campaign_registry(&registry)?;
    validate_benchmark_portfolio(&portfolio)?;
    validate_portfolio_campaign_alignment(&registry, &portfolio)?;
    Ok(())
}

#[test]
fn benchmark_portfolio_rejects_fixture_class_from_production_namespace() {
    let mut portfolio = benchmark_portfolio();
    portfolio.corpora[0].class = BenchmarkCorpusClass::TestFixture;

    let error = validate_benchmark_portfolio(&portfolio).err().map(|error| error.to_string());
    assert!(error
        .as_deref()
        .is_some_and(|message| message.contains("class and evidence scope disagree")));
}

#[test]
fn benchmark_stage_campaign_registry_rejects_slug_drift() {
    let mut registry = stage_campaign_registry();
    registry.campaigns[0].slug = "wrong-slug".to_string();

    let error = validate_stage_campaign_registry(&registry).err().map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("must derive from stage")));
}

#[test]
fn benchmark_stage_campaign_registry_rejects_material_driver_drift() {
    let mut registry = stage_campaign_registry();
    registry.campaigns[0].material_driver = Some(BenchmarkCampaignMaterialDriver::BamCoverage);

    let error = validate_stage_campaign_registry(&registry).err().map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("material driver")));
}

#[test]
fn canonical_campaign_records_without_material_drivers_use_generic_bindings() -> anyhow::Result<()>
{
    let mut value = serde_json::to_value(stage_campaign_registry())?;
    let campaign = value["campaigns"][0]
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("test campaign is not an object"))?;
    campaign.remove("material_driver");

    let historical: BenchmarkStageCampaignRegistry = serde_json::from_value(value)?;
    assert!(historical.campaigns[0].material_driver.is_none());
    validate_stage_campaign_registry(&historical)?;
    validate_executable_stage_campaign_registry(&historical)?;

    let mut drifted = historical;
    drifted.campaigns[0].tool_ids = vec!["seqfu".to_string()];
    assert!(validate_executable_stage_campaign_registry(&drifted).is_err());
    Ok(())
}

#[test]
fn checked_planning_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let schemas = [
        (
            "stage-campaigns.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_stage_campaign_registry_schema())?,
        ),
        (
            "portfolio.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_portfolio_schema())?,
        ),
        (
            "run-registry.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_run_registry_schema())?,
        ),
        (
            "publication-registry.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_publication_registry_schema())?,
        ),
        (
            "stage-universe.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_stage_universe_schema())?,
        ),
        (
            "local-hpc-job-graph.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_local_hpc_job_graph_schema())?,
        ),
    ];
    let schema_root = repository_root.join("benchmarks/contracts/schemas");
    for (name, generated) in schemas {
        let path = schema_root.join(name);
        write_generated_schema(&path, &generated)?;
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "planning schema drifted: {name}");
    }
    Ok(())
}

fn write_generated_schema(
    path: &std::path::Path,
    generated: &serde_json::Value,
) -> anyhow::Result<()> {
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(generated)?;
        bytes.push(b'\n');
        std::fs::write(path, bytes)?;
    }
    Ok(())
}

#[test]
fn benchmark_planning_alignment_rejects_run_id_drift() {
    let registry = stage_campaign_registry();
    let mut portfolio = benchmark_portfolio();
    portfolio.stage_runs.insert("fastq.profile_read_lengths".to_string(), "run-2043".to_string());

    let error = validate_portfolio_campaign_alignment(&registry, &portfolio)
        .err()
        .map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("benchmark run id drift")));
}

fn scientific_readiness() -> BenchmarkScientificReadinessReport {
    let portfolio = benchmark_portfolio();
    BenchmarkScientificReadinessReport::v2(
        BenchmarkScientificReadinessSummary {
            stage_count: 1,
            reserved_run_count: 1,
            ready_count: 1,
            blocked_count: 0,
            corpus_count: 1,
            admitted_comparative_corpus_count: 1,
            qualification_campaign_count: 0,
            comparative_campaign_count: 1,
        },
        BTreeMap::from([
            (
                "bam".to_string(),
                BenchmarkScientificReadinessDomain {
                    stage_count: 0,
                    ready_count: 0,
                    blocked_count: 0,
                },
            ),
            (
                "fastq".to_string(),
                BenchmarkScientificReadinessDomain {
                    stage_count: 1,
                    ready_count: 1,
                    blocked_count: 0,
                },
            ),
            (
                "vcf".to_string(),
                BenchmarkScientificReadinessDomain {
                    stage_count: 0,
                    ready_count: 0,
                    blocked_count: 0,
                },
            ),
        ]),
        portfolio.measurement,
        vec![BenchmarkScientificReadinessCorpus {
            id: "human-short-read-diversity".to_string(),
            status: BenchmarkCorpusStatus::Admitted,
            evidence_scope: BenchmarkEvidenceScope::Comparative,
            class: BenchmarkCorpusClass::ProductionReal,
            sample_count: 20,
            truth_strategy: "observational_only".to_string(),
            roles: vec!["short_read_diversity".to_string()],
        }],
        vec![BenchmarkScientificReadinessStage {
            stage_id: "fastq.profile_read_lengths".to_string(),
            domain: "fastq".to_string(),
            profile_id: "fastq-observational-qc".to_string(),
            retained_tools: vec!["seqfu".to_string(), "seqkit".to_string()],
            available_comparative_samples: 20,
            campaign_run_id: Some("run-2042".to_string()),
            reserved_run_id: "run-2042".to_string(),
            campaign_tier: Some(BenchmarkEvidenceTier::Comparative),
            interface_status: BenchmarkStageInterfaceStatus::Closed,
            readiness: BenchmarkScientificReadinessStatus::Ready,
            blockers: Vec::new(),
        }],
    )
}

#[test]
fn scientific_readiness_requires_exact_status_and_summary_closure() -> anyhow::Result<()> {
    let report = scientific_readiness();
    validate_scientific_readiness(&report)?;

    let mut status_drift = report.clone();
    status_drift.stages[0].blockers.push("missing evidence".to_string());
    assert!(validate_scientific_readiness(&status_drift).is_err());

    let mut summary_drift = report;
    summary_drift.summary.ready_count = 0;
    assert!(validate_scientific_readiness(&summary_drift).is_err());
    Ok(())
}

fn stage_coverage() -> BenchmarkStageCoverage {
    BenchmarkStageCoverage::v1(
        BenchmarkStageCoverageSummary {
            retained_stage_count: 2,
            defined_stage_count: 1,
            qualification_stage_count: 0,
            comparative_stage_count: 1,
            admitted_stage_count: 1,
            stale_stage_count: 0,
            failed_stage_count: 0,
            missing_stage_count: 1,
        },
        vec![
            BenchmarkStageCoverageRow {
                stage_id: "fastq.profile_read_lengths".to_string(),
                retained_tools: vec!["seqfu".to_string(), "seqkit".to_string()],
                campaign_run_id: Some("run-2042".to_string()),
                report_repository_commit: Some("a".repeat(40)),
                report_sha256: Some("b".repeat(64)),
                definition_status: BenchmarkStageDefinitionStatus::Comparative,
                execution_status: BenchmarkStageExecutionStatus::Admitted,
            },
            BenchmarkStageCoverageRow {
                stage_id: "vcf.stats".to_string(),
                retained_tools: vec!["bcftools".to_string()],
                campaign_run_id: None,
                report_repository_commit: None,
                report_sha256: None,
                definition_status: BenchmarkStageDefinitionStatus::Missing,
                execution_status: BenchmarkStageExecutionStatus::NotDefined,
            },
        ],
    )
}

#[test]
fn benchmark_stage_coverage_accepts_exact_sorted_closure() -> anyhow::Result<()> {
    validate_stage_coverage(&stage_coverage())?;
    Ok(())
}

#[test]
fn benchmark_stage_coverage_rejects_summary_drift() {
    let mut coverage = stage_coverage();
    coverage.summary.admitted_stage_count = 0;

    let error = validate_stage_coverage(&coverage).err().map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("summary disagrees")));
}

#[test]
fn benchmark_stage_coverage_rejects_incoherent_missing_state() {
    let mut coverage = stage_coverage();
    coverage.stages[1].execution_status = BenchmarkStageExecutionStatus::NotAdmitted;

    let error = validate_stage_coverage(&coverage).err().map(|error| error.to_string());
    assert!(error
        .as_deref()
        .is_some_and(|message| message.contains("definition, execution, and run identity")));
}

fn stage_universe() -> BenchmarkStageUniverse {
    BenchmarkStageUniverse::v1(
        BenchmarkStageUniverseSummary {
            stage_count: 2,
            benchmark_required_count: 1,
            qualification_only_count: 1,
            orchestration_only_count: 0,
            test_only_count: 0,
            retired_count: 0,
            excluded_count: 0,
        },
        vec![
            BenchmarkStageUniverseRow {
                stage_id: "fastq.profile_read_lengths".to_string(),
                domain: "fastq".to_string(),
                status: "supported".to_string(),
                benchmark_disposition: BenchmarkDisposition::BenchmarkRequired,
                benchmark_disposition_reason: "Reserved comparative benchmark.".to_string(),
                reserved_run_id: Some("run-2001".to_string()),
            },
            BenchmarkStageUniverseRow {
                stage_id: "fastq.repair_pairs".to_string(),
                domain: "fastq".to_string(),
                status: "planned".to_string(),
                benchmark_disposition: BenchmarkDisposition::QualificationOnly,
                benchmark_disposition_reason: "Utility contract qualification.".to_string(),
                reserved_run_id: None,
            },
        ],
    )
}

#[test]
fn benchmark_stage_universe_accepts_explicit_disposition_closure() -> anyhow::Result<()> {
    validate_stage_universe(&stage_universe())?;
    Ok(())
}

#[test]
fn benchmark_stage_universe_rejects_run_on_qualification_utility() {
    let mut universe = stage_universe();
    universe.stages[1].reserved_run_id = Some("run-2002".to_string());
    assert!(validate_stage_universe(&universe).is_err());
}

#[test]
fn benchmark_stage_universe_rejects_non_monotonic_run_reservation() {
    let mut universe = stage_universe();
    universe.stages[0].reserved_run_id = Some("run-2002".to_string());

    let error = validate_stage_universe(&universe).err().map(|error| error.to_string());
    assert!(error
        .as_deref()
        .is_some_and(|message| message.contains("monotonic and contiguous from run-2001")));
}

fn local_hpc_job_graph() -> BenchmarkLocalHpcJobGraph {
    BenchmarkLocalHpcJobGraph {
        schema_version: BENCHMARK_LOCAL_HPC_JOB_GRAPH_SCHEMA_V1.to_string(),
        manifest_path: "runs/bench/hpc-dry-run/all-domain-slurm-submit-manifest.json".to_string(),
        job_count: 2,
        benchmark_job_count: 1,
        essential_pipeline_job_count: 1,
        dependency_count: 1,
        root_job_ids: vec!["benchmark:fastq:profile".to_string()],
        topological_order: vec![
            "benchmark:fastq:profile".to_string(),
            "pipeline:fastq:report".to_string(),
        ],
        nodes: vec![
            BenchmarkLocalHpcJobGraphNode {
                job_id_local: "benchmark:fastq:profile".to_string(),
                job_kind: "benchmark_result".to_string(),
                result_id: Some("fastq-profile".to_string()),
                pipeline_id: None,
                node_id: None,
                domain: "fastq".to_string(),
                stage_id: "fastq.profile_reads".to_string(),
                tool_id: "seqkit".to_string(),
                corpus_id: "fastq-cohort".to_string(),
                asset_profile_id: "fastq-assets".to_string(),
                dependency_count: 0,
                dependent_count: 1,
                dependencies: Vec::new(),
                dependents: vec!["pipeline:fastq:report".to_string()],
            },
            BenchmarkLocalHpcJobGraphNode {
                job_id_local: "pipeline:fastq:report".to_string(),
                job_kind: "essential_pipeline_node".to_string(),
                result_id: None,
                pipeline_id: Some("fastq-qc".to_string()),
                node_id: Some("report".to_string()),
                domain: "fastq".to_string(),
                stage_id: "fastq.report_qc".to_string(),
                tool_id: "multiqc".to_string(),
                corpus_id: "fastq-cohort".to_string(),
                asset_profile_id: "fastq-assets".to_string(),
                dependency_count: 1,
                dependent_count: 0,
                dependencies: vec!["benchmark:fastq:profile".to_string()],
                dependents: Vec::new(),
            },
        ],
    }
}

#[test]
fn local_hpc_job_graph_validates_exact_identity_and_edge_closure() -> anyhow::Result<()> {
    let graph = local_hpc_job_graph();
    validate_benchmark_local_hpc_job_graph(&graph)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcJobGraph>(&serde_json::to_vec(&graph)?)?,
        graph
    );
    Ok(())
}

#[test]
fn local_hpc_job_graph_rejects_nonreciprocal_and_misordered_edges() {
    let mut nonreciprocal = local_hpc_job_graph();
    nonreciprocal.nodes[0].dependents.clear();
    nonreciprocal.nodes[0].dependent_count = 0;
    assert!(validate_benchmark_local_hpc_job_graph(&nonreciprocal).is_err());

    let mut misordered = local_hpc_job_graph();
    misordered.topological_order.reverse();
    assert!(validate_benchmark_local_hpc_job_graph(&misordered).is_err());
}
