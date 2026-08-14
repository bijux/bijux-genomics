//! Owner: bijux-dna-bench
//! Benchmark workflow facade over summary, evaluation, suite loading, and suite persistence.
#![allow(dead_code)]

mod alias_resolution;
mod appraisal;
mod attempt_migration;
mod bam_alignment;
mod bam_campaign;
mod bam_coverage;
mod bam_duplication_metrics;
mod bam_endogenous_content;
mod bam_genotyping;
mod bam_mapping_summary;
mod bam_observation;
mod bam_picard_cohort;
mod benchmark_matrix;
mod campaign_cases;
mod campaign_engine;
mod campaign_lifecycle;
mod campaign_material;
mod container_report_portfolio;
mod corpus_acquisition;
mod corpus_materialization;
mod cross_campaign;
mod data_portfolio_audit;
mod evaluation;
mod evidence_book;
mod fastq_campaign;
mod fastq_detect_adapters;
mod fastq_expectations;
mod fastq_profile_read_lengths;
mod fastq_profile_reads;
mod fastq_validate_reads;
mod full_benchmark_report;
mod hardening_campaign;
mod options;
mod parameter_authority;
mod publication_registry;
mod reference_materialization;
mod replay;
mod report_load;
mod run_index;
mod run_registry;
mod run_suite;
mod runtime_selection;
mod scientific_readiness;
mod source_authority;
mod stage_coverage;
mod stage_families;
mod stage_report;
mod stage_result;
mod stage_tool_closure;
mod stage_universe;
mod suite_load;
mod summary;
mod summary_fairness;
mod summary_scope;
mod summary_statistics;
mod vcf_campaign;

pub use alias_resolution::{
    choose_canonical_tool_id, legacy_benchmark_stage_alias_target, legacy_benchmark_stage_aliases,
    normalize_tool_id, required_migration_stage_ids,
    stage_set_contains_canonical_or_migration_alias, LegacyBenchmarkStageAlias,
};
pub use appraisal::{
    appraise_matrix_report, hardening_queue_from_appraisal, load_appraisal_report,
    load_hardening_queue_report, publish_appraisal_report, publish_hardening_queue_report,
};
pub use attempt_migration::{
    apply_attempt_migration, plan_attempt_migration, rollback_attempt_migration,
    verify_attempt_migration, write_attempt_migration_plan, BenchmarkAttemptMigrationVerification,
    BenchmarkAttemptMigrationVerificationEntry,
};
pub use bam_alignment::{
    prepare_bam_alignment_material, publish_bam_alignment_stage_report, BamAlignmentMaterial,
    BamAlignmentStageReportPublication,
};
pub use bam_campaign::{
    derive_bam_benchmark_campaign, load_bam_benchmark_campaign_report,
    publish_bam_benchmark_campaign_report,
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
pub use bam_picard_cohort::{
    prepare_bam_picard_cohort_material, publish_bam_gc_bias_stage_report,
    publish_bam_insert_size_stage_report, BamPicardCohortMaterial,
};
pub use benchmark_matrix::{
    current_benchmark_matrix_timestamp, derive_benchmark_matrix, load_benchmark_matrix_report,
    publish_benchmark_matrix, BenchmarkMatrixRequest,
};
pub use bijux_dna_runtime::{
    append_attempt_lifecycle_transition, benchmark_attempt_directory, initialize_attempt_lifecycle,
    load_attempt_lifecycle,
};
pub use campaign_cases::{
    admit_benchmark_cases, aggregate_benchmark_cases, plan_benchmark_cases,
    BenchmarkCaseAdmissionRequest, BenchmarkCaseAggregationRequest, BenchmarkCasePlanningRequest,
};
pub use campaign_engine::{
    aggregate_benchmark_admitted_case_observation_set, aggregate_benchmark_case_observations,
    benchmark_case_matrix_sha256, build_benchmark_admitted_case_observation_set,
    build_benchmark_stage_report_from_case_aggregations,
    ensure_benchmark_admitted_case_observation_set, ensure_benchmark_case_aggregation_set,
    ensure_benchmark_case_matrix, load_benchmark_admitted_case_observation_set,
    load_benchmark_case_aggregation_set, load_benchmark_case_matrix, plan_benchmark_case_matrix,
    validate_benchmark_admitted_case_observation_set, validate_benchmark_case_aggregation_set,
    validate_benchmark_case_aggregations, validate_benchmark_case_matrix,
    BenchmarkAdmittedCaseObservationSetV1, BenchmarkAdmittedCaseObservationV2,
    BenchmarkCaseAggregationSetV1, BenchmarkCaseAggregationV2, BenchmarkCaseDefinitionV1,
    BenchmarkCaseExecutionEvidenceV1, BenchmarkCaseMatrixRequest, BenchmarkCaseMatrixV2,
    BenchmarkCaseMeasurementPlanV1, BenchmarkCaseRepetitionObservationV1,
    BenchmarkCaseSciencePlanV1, BenchmarkCorpusSnapshotV1, BenchmarkDataSnapshotRoleV1,
    BenchmarkDataSnapshotV1, BenchmarkStageNormalization, BenchmarkStagePluginRegistry,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SET_SCHEMA_V1, BENCHMARK_CASE_AGGREGATION_SCHEMA_V2,
    BENCHMARK_CASE_AGGREGATION_SET_SCHEMA_V1, BENCHMARK_CASE_MATRIX_SCHEMA_V2,
};
pub use campaign_lifecycle::{
    admit_checked_in_benchmark_campaign, build_benchmark_stage_plans, build_execution_spec,
    ensure_execution_spec, ensure_stage_campaign_attempts, ensure_stage_plan_manifest,
    load_case_execution_authorities, load_checked_in_benchmark_campaign_plan,
    load_execution_authorities, load_execution_spec, load_stage_plan_manifest,
    plan_benchmark_case_attempts, plan_benchmark_case_retry_attempts,
    plan_checked_in_benchmark_campaign, plan_stage_campaign_attempts,
    plan_stage_campaign_retry_attempts, prepare_checked_in_benchmark_campaign,
    recover_stage_campaign_retry_lineage, retry_checked_in_benchmark_campaign,
    status_checked_in_benchmark_campaign, verify_campaign_plan_source,
    verify_case_campaign_plan_source, BenchmarkCycleAdmission, BenchmarkCycleAdmissionRequest,
    BenchmarkCycleAdmissionState, BenchmarkCycleAdmissionUnit, BenchmarkCyclePlanRequest,
    BenchmarkCyclePreparation, BenchmarkCyclePreparationRequest, BenchmarkCyclePreparationState,
    BenchmarkCyclePreparationUnit, BenchmarkCycleRetryRequest, BenchmarkCycleStatusRequest,
    BenchmarkExecutionAuthorities, BenchmarkExecutionMaterial, BenchmarkStagePlanInputMaterial,
    BenchmarkStagePlanOutputMaterial, BenchmarkStagePlanUnitMaterial,
};
pub use campaign_material::{
    admit_benchmark_campaign_evidence, prepare_benchmark_campaign_material,
    publish_benchmark_campaign_report,
    publish_benchmark_campaign_report_with_presentation_authority, BenchmarkCampaignEvidence,
    BenchmarkCampaignMaterialRequest, BenchmarkCampaignReportPublication,
};
pub use container_report_portfolio::{
    derive_container_report_portfolio_index, render_container_report_portfolio_index_json,
    render_container_report_portfolio_index_markdown,
};
pub use corpus_acquisition::{
    acquire_fastq_corpus, derive_fastq_corpus_acquisition_plan, load_fastq_corpus_source_spec,
    CorpusAcquisitionRequest,
};
pub use corpus_materialization::{
    materialize_fastq_corpus, CorpusMaterializationPublication, CorpusMaterializationRequest,
};
pub use cross_campaign::{
    derive_cross_benchmark_campaign, load_cross_benchmark_campaign_report,
    publish_cross_benchmark_campaign_report,
};
pub use data_portfolio_audit::{
    derive_benchmark_data_portfolio_audit, publish_benchmark_data_portfolio_audit,
    render_benchmark_data_portfolio_audit_markdown, BenchmarkDataPortfolioAuditPublicationRequest,
    BenchmarkDataPortfolioAuditRequest,
};
pub use evaluation::{compare, gate};
pub use evidence_book::{publish_checked_in_benchmark_evidence_book, BenchmarkEvidenceBookRequest};
pub use fastq_campaign::{
    derive_fastq_benchmark_campaign, load_fastq_benchmark_campaign_report,
    publish_fastq_benchmark_campaign_report,
};
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
pub use full_benchmark_report::{
    build_full_benchmark_comparable_metric_rows, build_full_benchmark_corpus_rows,
    build_full_benchmark_failures_section, build_full_benchmark_memory_rows,
    build_full_benchmark_missing_result_rows, build_full_benchmark_pipeline_rows,
    build_full_benchmark_runtime_rows, build_full_benchmark_stage_rows,
    build_full_benchmark_tool_rows, build_full_benchmark_unsupported_pair_rows,
    render_full_benchmark_report_markdown,
};
pub use hardening_campaign::{
    derive_hardening_benchmark_campaign, load_hardening_benchmark_campaign_report,
    publish_hardening_benchmark_campaign_report,
};
pub use options::BenchRunOptions;
pub use publication_registry::{
    derive_benchmark_publication_registry, publish_checked_in_benchmark_publication_registry,
    render_benchmark_publication_registry_markdown, BenchmarkPublicationRegistryWriteMode,
};
pub use reference_materialization::{
    materialize_checked_in_reference_bundle, ReferenceBundleMaterializationRequest,
};
pub use replay::{replay_checked_in_benchmark_publication, BenchmarkReplayRequest};
pub use report_load::{
    load_checked_in_stage_reports, load_stage_report, load_stage_reports_from_dir,
};
pub use run_index::{
    derive_benchmark_run_index, publish_checked_in_benchmark_run_index,
    render_benchmark_run_index_markdown, BenchmarkRunIndexWriteMode,
};
pub use run_registry::{
    derive_benchmark_run_registry, publish_checked_in_benchmark_run_registry,
    render_benchmark_run_registry_markdown, BenchmarkRunRegistryWriteMode,
};
pub use runtime_selection::resolve_benchmark_runtime;
pub use scientific_readiness::{
    derive_scientific_readiness, publish_checked_in_scientific_readiness,
    render_scientific_readiness_markdown,
};
pub use source_authority::load_benchmark_data_source_spec;
pub use stage_coverage::{
    derive_stage_coverage, publish_checked_in_stage_coverage, render_stage_coverage_markdown,
    BenchmarkCoverageWriteMode,
};
pub use stage_families::{
    BenchmarkStageFamily, BAM_STAGE_FAMILIES, FASTQ_STAGE_FAMILIES, VCF_STAGE_FAMILIES,
};
pub use stage_report::{
    publish_stage_report, render_stage_report_markdown, BenchmarkStageReportPublication,
};
pub use stage_result::{
    load_benchmark_stage_result_manifest, validate_benchmark_stage_result_path,
};
pub use stage_tool_closure::{
    derive_benchmark_stage_tool_closure, derive_supported_domain_stage_tool_pairs,
    publish_checked_in_benchmark_stage_tool_closure, render_benchmark_stage_tool_closure_markdown,
    BenchmarkStageToolClosureWriteMode,
};
pub use stage_universe::{
    derive_stage_universe, publish_checked_in_stage_universe, render_stage_universe_markdown,
    BenchmarkUniverseWriteMode,
};
pub use suite_load::{
    load_benchmark_data_fitness_rubric, load_benchmark_portfolio, load_bundle_catalog,
    load_bundle_manifest, load_checked_in_benchmark_contracts,
    load_checked_in_benchmark_planning_contracts, load_corpus_catalog, load_corpus_manifest,
    load_stage_campaign_registry, load_stage_interface_catalog,
    load_stage_interface_catalog_for_publication, load_stage_interface_catalog_from_repo,
    load_suite, BenchmarkPublicationInterfaceCatalog, BenchmarkReportPresentationAuthority,
};
pub use summary::summarize;
pub use vcf_campaign::{
    derive_vcf_benchmark_campaign, load_vcf_benchmark_campaign_report,
    publish_vcf_benchmark_campaign_report,
};
