//! Owner: bijux-dna-bench
//! Benchmark workflow facade over summary, evaluation, suite loading, and suite persistence.
#![allow(dead_code)]

mod attempt_migration;
mod bam_coverage;
mod bam_duplication_metrics;
mod bam_endogenous_content;
mod bam_genotyping;
mod bam_mapping_summary;
mod bam_observation;
mod campaign_engine;
mod campaign_lifecycle;
mod campaign_material;
mod corpus_acquisition;
mod corpus_materialization;
mod data_portfolio_audit;
mod evaluation;
mod fastq_detect_adapters;
mod fastq_expectations;
mod fastq_profile_read_lengths;
mod fastq_profile_reads;
mod fastq_validate_reads;
mod options;
mod parameter_authority;
mod publication_registry;
mod report_load;
mod run_registry;
mod run_suite;
mod scientific_readiness;
mod source_authority;
mod stage_coverage;
mod stage_report;
mod stage_universe;
mod suite_load;
mod summary;
mod summary_fairness;
mod summary_scope;
mod summary_statistics;

pub use attempt_migration::{
    apply_attempt_migration, plan_attempt_migration, rollback_attempt_migration,
    verify_attempt_migration, write_attempt_migration_plan, BenchmarkAttemptMigrationVerification,
    BenchmarkAttemptMigrationVerificationEntry,
};
pub use bam_coverage::{
    prepare_bam_coverage_material, prepare_bam_coverage_material_from_corpus,
    publish_bam_coverage_stage_report, render_bam_coverage_markdown, BamCoverageMaterial,
    BamCoverageStageReportPublication,
};
pub use bam_duplication_metrics::{
    prepare_bam_duplication_metrics_material, prepare_bam_duplication_metrics_material_from_corpus,
    publish_bam_duplication_metrics_stage_report, render_bam_duplication_metrics_markdown,
    BamDuplicationMetricsMaterial, BamDuplicationMetricsStageReportPublication,
};
pub use bam_endogenous_content::{
    prepare_bam_endogenous_content_material, prepare_bam_endogenous_content_material_from_corpus,
    publish_bam_endogenous_content_stage_report, render_bam_endogenous_content_markdown,
    BamEndogenousContentMaterial, BamEndogenousContentStageReportPublication,
};
pub use bam_genotyping::{
    prepare_bam_genotyping_material, prepare_bam_genotyping_material_from_corpus,
    publish_bam_genotyping_stage_report, render_bam_genotyping_markdown, BamGenotypingMaterial,
    BamGenotypingStageReportPublication,
};
pub use bam_mapping_summary::{
    prepare_bam_mapping_summary_material, prepare_bam_mapping_summary_material_from_corpus,
    publish_bam_mapping_summary_stage_report, render_bam_mapping_summary_markdown,
    BamMappingSummaryMaterial, BamMappingSummaryStageReportPublication,
};
pub use bijux_dna_runtime::{
    append_attempt_lifecycle_transition, benchmark_attempt_directory, initialize_attempt_lifecycle,
    load_attempt_lifecycle,
};
pub use campaign_engine::{
    aggregate_benchmark_case_observations, build_benchmark_stage_report_from_case_aggregations,
    plan_benchmark_case_matrix, validate_benchmark_case_aggregations,
    validate_benchmark_case_matrix, BenchmarkAdmittedCaseObservationV2, BenchmarkCaseAggregationV2,
    BenchmarkCaseDefinitionV1, BenchmarkCaseExecutionEvidenceV1, BenchmarkCaseMatrixRequest,
    BenchmarkCaseMatrixV1, BenchmarkCaseMeasurementPlanV1, BenchmarkCaseRepetitionObservationV1,
    BenchmarkCaseSciencePlanV1, BenchmarkCorpusSnapshotV1,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2, BENCHMARK_CASE_AGGREGATION_SCHEMA_V2,
    BENCHMARK_CASE_MATRIX_SCHEMA_V1,
};
pub use campaign_lifecycle::{
    build_execution_spec, ensure_execution_spec, ensure_stage_campaign_attempts,
    load_execution_authorities, load_execution_spec, plan_stage_campaign_attempts,
    plan_stage_campaign_retry_attempts, recover_stage_campaign_retry_lineage,
    verify_campaign_plan_source, BenchmarkExecutionAuthorities, BenchmarkExecutionMaterial,
};
pub use campaign_material::{
    admit_benchmark_campaign_evidence, prepare_benchmark_campaign_material,
    publish_benchmark_campaign_report,
    publish_benchmark_campaign_report_with_presentation_authority, BenchmarkCampaignEvidence,
    BenchmarkCampaignMaterialRequest, BenchmarkCampaignReportPublication,
};
pub use corpus_acquisition::{
    acquire_fastq_corpus, derive_fastq_corpus_acquisition_plan, load_fastq_corpus_source_spec,
    CorpusAcquisitionRequest,
};
pub use corpus_materialization::{
    materialize_fastq_corpus, CorpusMaterializationPublication, CorpusMaterializationRequest,
};
pub use data_portfolio_audit::{
    derive_benchmark_data_portfolio_audit, publish_benchmark_data_portfolio_audit,
    render_benchmark_data_portfolio_audit_markdown, BenchmarkDataPortfolioAuditPublicationRequest,
    BenchmarkDataPortfolioAuditRequest,
};
pub use evaluation::{compare, gate};
pub use fastq_detect_adapters::{
    prepare_fastq_detect_adapters_material_from_expectations,
    publish_fastq_detect_adapters_stage_report, render_fastq_detect_adapters_markdown,
    FastqDetectAdaptersMaterial, FastqDetectAdaptersStageReportPublication,
};
pub use fastq_expectations::{
    ensure_fastq_profile_reads_expectations, ensure_fastq_read_length_expectations,
    FastqProfileReadsExpectationPublication, FastqProfileReadsExpectationSample,
    FastqReadLengthExpectationPublication,
};
pub use fastq_profile_read_lengths::{
    prepare_fastq_profile_read_lengths_material,
    prepare_fastq_profile_read_lengths_material_from_expectations,
    publish_fastq_profile_read_lengths_stage_report, render_fastq_profile_read_lengths_markdown,
    FastqProfileReadLengthsMaterial, FastqProfileReadLengthsStageReportPublication,
};
pub use fastq_profile_reads::{
    prepare_fastq_profile_reads_material, prepare_fastq_profile_reads_material_from_expectations,
    publish_fastq_profile_reads_stage_report, render_fastq_profile_reads_markdown,
    FastqProfileReadsMaterial, FastqProfileReadsStageReportPublication,
};
pub use fastq_validate_reads::{
    prepare_fastq_validate_reads_material_from_expectations,
    publish_fastq_validate_reads_stage_report, render_fastq_validate_reads_markdown,
    FastqValidateReadsMaterial, FastqValidateReadsStageReportPublication,
};
pub use options::BenchRunOptions;
pub use publication_registry::{
    derive_benchmark_publication_registry, publish_checked_in_benchmark_publication_registry,
    render_benchmark_publication_registry_markdown, BenchmarkPublicationRegistryWriteMode,
};
pub use report_load::{
    load_checked_in_stage_reports, load_stage_report, load_stage_reports_from_dir,
};
pub use run_registry::{
    derive_benchmark_run_registry, publish_checked_in_benchmark_run_registry,
    render_benchmark_run_registry_markdown, BenchmarkRunRegistryWriteMode,
};
pub use scientific_readiness::{
    derive_scientific_readiness, publish_checked_in_scientific_readiness,
    render_scientific_readiness_markdown,
};
pub use source_authority::load_benchmark_data_source_spec;
pub use stage_coverage::{
    derive_stage_coverage, publish_checked_in_stage_coverage, render_stage_coverage_markdown,
    BenchmarkCoverageWriteMode,
};
pub use stage_report::{
    publish_stage_report, render_stage_report_markdown, BenchmarkStageReportPublication,
};
pub use stage_universe::{
    derive_stage_universe, publish_checked_in_stage_universe, render_stage_universe_markdown,
    BenchmarkUniverseWriteMode,
};
pub use suite_load::{
    load_benchmark_portfolio, load_bundle_catalog, load_bundle_manifest,
    load_checked_in_benchmark_contracts, load_checked_in_benchmark_planning_contracts,
    load_corpus_catalog, load_corpus_manifest, load_stage_campaign_registry,
    load_stage_interface_catalog, load_stage_interface_catalog_for_publication,
    load_stage_interface_catalog_from_repo, load_suite, BenchmarkPublicationInterfaceCatalog,
    BenchmarkReportPresentationAuthority,
};
pub use summary::summarize;
