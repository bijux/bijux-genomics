//! Owner: bijux-dna-bench
//! Stable public benchmark surface re-exported by the crate root.

pub use crate::repo::{
    bench_bam_local_stage_matrix_path, bench_bundles_dir, bench_contracts_dir, bench_corpora_dir,
    bench_data_dir, bench_fastq_local_stage_matrix_path, bench_local_config_dir,
    bench_portfolio_path, bench_reports_dir, bench_stage_campaigns_path,
    bench_stage_interfaces_path, bench_suites_dir,
};
pub use crate::workflow::{
    compare, gate, load_benchmark_portfolio, load_bundle_catalog, load_bundle_manifest,
    load_checked_in_benchmark_contracts, load_checked_in_benchmark_planning_contracts,
    load_checked_in_stage_reports, load_corpus_catalog, load_corpus_manifest,
    load_stage_campaign_registry, load_stage_interface_catalog,
    load_stage_interface_catalog_from_repo, load_stage_report, load_stage_reports_from_dir,
    load_suite, summarize, BenchRunOptions,
};
pub use bijux_dna_bench_model::{
    AdmittedBenchmarkStageReport, AnalysisRequirements, BackendComparisonSpec, BenchError,
    BenchmarkAttemptRecord, BenchmarkBundleManifest, BenchmarkCachePolicy,
    BenchmarkComparisonMetricTolerance, BenchmarkCorpusContract, BenchmarkCorpusManifest,
    BenchmarkCorpusStatus, BenchmarkCycleStatus, BenchmarkDecision, BenchmarkEvidenceScope,
    BenchmarkEvidenceTier, BenchmarkExecutionRuntime, BenchmarkExecutionScheduler,
    BenchmarkJobReference, BenchmarkLifecycleStatus, BenchmarkMeasurementContract,
    BenchmarkObservation, BenchmarkParameterValue, BenchmarkPipelineParameterConfig,
    BenchmarkPortfolio, BenchmarkProfileAssignment, BenchmarkReportVerdict,
    BenchmarkScientificProfile, BenchmarkStageCampaignRegistry, BenchmarkStageCampaignSpec,
    BenchmarkStageDownstreamProbe, BenchmarkStageInterfaceCatalog, BenchmarkStageInterfaceSpec,
    BenchmarkStageParameterConfig, BenchmarkStageReport, BenchmarkStageReportDesign,
    BenchmarkSuiteSpec, BenchmarkSummary, BenchmarkToolReport, CorpusDatasetSpec, CorpusDomain,
    CorpusScale, DatasetSpec, DecisionRationale, DiversityRequirements, DriftScenarioSpec,
    GateDecision, GatePolicy, GatePolicyOverrides, GateViolation, MetricSummary, MetricsEnvelope,
    ReplicatePolicy, StratificationRequirement, SummaryRow, TruthSetHook, TruthSetStatus,
    BENCHMARK_ATTEMPT_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V1,
    BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1, BENCHMARK_PORTFOLIO_SCHEMA_V1,
    BENCHMARK_REPORT_ADMISSION_SCHEMA_V1, BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,
    BENCHMARK_STAGE_INTERFACES_SCHEMA_V1, BENCHMARK_STAGE_REPORT_SCHEMA_V1,
};
