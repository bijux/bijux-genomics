//! Owner: bijux-dna-bench-model
//! Schema versions and contract validators.
//! Owns validation for bench artifacts and inputs.
//! Must not perform IO.
#![allow(dead_code)]

mod corpus_expectations;
mod execution;
mod interfaces;
mod lifecycle;
mod planning;
mod records;
mod report;
mod schema_versions;
mod suite;
pub use corpus_expectations::validate_fastq_read_length_expectations;
pub use execution::validate_execution_spec;
pub use interfaces::{
    validate_defined_interface_campaign_alignment, validate_interface_parameter_alignment,
    validate_pipeline_parameter_config, validate_stage_interface_catalog,
};
pub use lifecycle::{
    append_attempt_transition, benchmark_run_directory_name, validate_attempt_archive,
    validate_attempt_identity, validate_attempt_record_v2, validate_attempt_transition,
    validate_campaign_plan, validate_cycle_status_v2,
};
pub use planning::{
    validate_benchmark_portfolio, validate_defined_campaign_portfolio_alignment,
    validate_portfolio_campaign_alignment, validate_scientific_readiness,
    validate_stage_campaign_registry, validate_stage_coverage,
};
pub use records::{
    validate_bundle_manifest, validate_corpus_manifest, validate_decision, validate_observation,
    validate_summary,
};
pub use report::{
    validate_report_admission, validate_stage_report, validate_stage_report_alignment,
};
pub use schema_versions::{
    BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1, BENCHMARK_ATTEMPT_MIGRATION_LEDGER_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_PLAN_SCHEMA_V1, BENCHMARK_ATTEMPT_SCHEMA_V1,
    BENCHMARK_ATTEMPT_SCHEMA_V2, BENCHMARK_CAMPAIGN_PLAN_SCHEMA_V1,
    BENCHMARK_CYCLE_STATUS_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V2,
    BENCHMARK_EXECUTION_SPEC_SCHEMA_V1, BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1,
    BENCHMARK_PORTFOLIO_SCHEMA_V1, BENCHMARK_REPORT_ADMISSION_SCHEMA_V1,
    BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V1, BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,
    BENCHMARK_STAGE_COVERAGE_SCHEMA_V1, BENCHMARK_STAGE_INTERFACES_SCHEMA_V1,
    BENCHMARK_STAGE_REPORT_SCHEMA_V1, BUNDLE_SCHEMA_V1, CORPUS_SCHEMA_V1, DECISION_SCHEMA_V1,
    FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V1, FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V2,
    OBSERVATION_SCHEMA_V1, SUITE_SCHEMA_V1, SUMMARY_SCHEMA_V1,
};
pub use suite::validate_suite;
