//! Owner: bijux-dna-bench-model
//! Schema versions and contract validators.
//! Owns validation for bench artifacts and inputs.
//! Must not perform IO.
#![allow(dead_code)]

mod compatibility;
mod corpus_expectations;
mod data_catalog_registry;
mod data_index;
mod data_portfolio_audit;
mod execution;
mod interfaces;
mod lifecycle;
mod materialization;
mod planning;
mod protocol;
mod publication;
mod publication_registry;
mod records;
mod replay;
mod report;
mod run_index;
mod schema_versions;
mod source_authority;
mod stage_result;
mod suite;
pub use compatibility::{
    migrate_benchmark_attempt_v1, parse_benchmark_attempt_document, parse_benchmark_cycle_document,
    validate_attempt_record_v1, validate_cycle_status_v1,
};
pub use corpus_expectations::validate_fastq_read_length_expectations;
pub use data_catalog_registry::validate_governed_data_catalog_registry;
pub use data_index::validate_benchmark_data_index;
pub use data_portfolio_audit::validate_benchmark_data_portfolio_audit;
pub use execution::validate_execution_spec;
pub use interfaces::{
    validate_defined_interface_campaign_alignment, validate_interface_parameter_alignment,
    validate_pipeline_parameter_config, validate_stage_interface_catalog,
};
pub use lifecycle::{
    append_attempt_transition, benchmark_attempt_identity_sha256, benchmark_run_directory_name,
    validate_attempt_archive, validate_attempt_identity, validate_attempt_record_v2,
    validate_attempt_transition, validate_campaign_plan, validate_cycle_status_v2,
    validate_scheduler_observation,
};
pub use materialization::{
    validate_corpus_acquisition_plan, validate_fastq_corpus_source_spec,
    validate_governed_data_catalog, validate_governed_data_catalog_document,
    validate_governed_preserved_data_catalog, validate_utc_timestamp,
};
pub use planning::{
    validate_benchmark_portfolio, validate_benchmark_run_registry,
    validate_defined_campaign_portfolio_alignment, validate_portfolio_campaign_alignment,
    validate_scientific_readiness, validate_stage_campaign_registry, validate_stage_coverage,
    validate_stage_universe,
};
pub use protocol::{
    benchmark_run_definition_sha256, validate_benchmark_case_definition,
    validate_benchmark_run_definition, validate_benchmark_run_definition_revision,
};
pub use publication::{
    validate_benchmark_citation_record, validate_benchmark_evidence_book,
    validate_benchmark_publication_receipt,
};
pub use publication_registry::validate_benchmark_publication_registry;
pub use records::{
    validate_bundle_manifest, validate_corpus_manifest, validate_decision, validate_observation,
    validate_summary,
};
pub use replay::validate_benchmark_replay_receipt;
pub use report::{
    validate_report_admission, validate_stage_report, validate_stage_report_alignment,
};
pub use run_index::validate_benchmark_run_index;
pub use schema_versions::{
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V1, BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SET_SCHEMA_V1, BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_LEDGER_SCHEMA_V1, BENCHMARK_ATTEMPT_MIGRATION_PLAN_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_VERIFICATION_SCHEMA_V1, BENCHMARK_ATTEMPT_SCHEMA_V1,
    BENCHMARK_ATTEMPT_SCHEMA_V2, BENCHMARK_CAMPAIGN_PLAN_SCHEMA_V1,
    BENCHMARK_CASE_AGGREGATION_SCHEMA_V1, BENCHMARK_CASE_AGGREGATION_SCHEMA_V2,
    BENCHMARK_CASE_AGGREGATION_SET_SCHEMA_V1, BENCHMARK_CASE_DEFINITION_SCHEMA_V1,
    BENCHMARK_CASE_MATRIX_SCHEMA_V1, BENCHMARK_CASE_MATRIX_SCHEMA_V2,
    BENCHMARK_CITATION_RECORD_SCHEMA_V1, BENCHMARK_CORPUS_MATERIALIZATION_PUBLICATION_SCHEMA_V1,
    BENCHMARK_CYCLE_STATUS_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V2,
    BENCHMARK_DATA_INDEX_SCHEMA_V1, BENCHMARK_DATA_INDEX_SCHEMA_V2,
    BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V1, BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2,
    BENCHMARK_DATA_SOURCE_SCHEMA_V1, BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
    BENCHMARK_EVIDENCE_BOOK_SCHEMA_V1, BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1,
    BENCHMARK_EXECUTION_SPEC_SCHEMA_V1, BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1,
    BENCHMARK_PORTFOLIO_SCHEMA_V1, BENCHMARK_PORTFOLIO_SCHEMA_V2,
    BENCHMARK_PUBLICATION_RECEIPT_SCHEMA_V1, BENCHMARK_PUBLICATION_REGISTRY_SCHEMA_V1,
    BENCHMARK_REPLAY_RECEIPT_SCHEMA_V1, BENCHMARK_REPORT_ADMISSION_SCHEMA_V1,
    BENCHMARK_RUN_DEFINITION_SCHEMA_V1, BENCHMARK_RUN_INDEX_SCHEMA_V1,
    BENCHMARK_RUN_REGISTRY_SCHEMA_V1, BENCHMARK_SCHEDULER_OBSERVATION_SCHEMA_V1,
    BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V1, BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V2,
    BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1, BENCHMARK_STAGE_COVERAGE_SCHEMA_V1,
    BENCHMARK_STAGE_INTERFACES_SCHEMA_V1, BENCHMARK_STAGE_REPORT_SCHEMA_V1,
    BENCHMARK_STAGE_RESULT_SCHEMA_V2, BENCHMARK_STAGE_UNIVERSE_SCHEMA_V1, BUNDLE_SCHEMA_V1,
    CORPUS_ACQUISITION_SCHEMA_V1, CORPUS_ACQUISITION_SCHEMA_V2, CORPUS_SCHEMA_V1,
    DECISION_SCHEMA_V1, FASTQ_CORPUS_SOURCE_SCHEMA_V1, FASTQ_PROFILE_READS_EXPECTATIONS_SCHEMA_V1,
    FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V1, FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V2,
    GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_SCHEMA_V2, OBSERVATION_SCHEMA_V1, SUITE_SCHEMA_V1, SUMMARY_SCHEMA_V1,
};
pub use source_authority::validate_benchmark_data_source_spec;
pub use stage_result::validate_benchmark_stage_result_manifest;
pub use suite::validate_suite;
