//! Owner: bijux-dna-bench-model
//! Schema versions and contract validators.
//! Owns validation for bench artifacts and inputs.
//! Must not perform IO.
#![allow(dead_code)]

mod active_scope;
mod all_domain_parser_collector;
mod appraisal;
mod benchmark_matrix;
mod comparable_metrics;
mod compatibility;
mod container_report_portfolio;
mod corpus_dossier;
mod corpus_expectations;
mod cross_campaign;
mod data_catalog_registry;
mod data_fitness;
mod data_index;
mod data_portfolio_audit;
mod domain_campaign;
mod dossier_index;
mod essential_pipeline_readiness;
mod execution;
mod interfaces;
mod lifecycle;
mod local_benchmark_summary;
mod local_corpus_run;
mod local_corpus_skip;
mod local_corpus_stage_compatibility;
mod local_hpc_asset_staging;
mod local_hpc_result_collection_simulation;
mod local_hpc_scratch_layout;
mod local_pipeline_dag;
mod local_runtime_metrics;
mod local_slurm_dependency;
mod local_slurm_submit;
mod local_stage_commands;
mod local_stage_completion;
mod local_stage_inventory;
mod local_vcf_smoke;
mod materialization;
mod planning;
mod protocol;
mod publication;
mod publication_findings;
mod publication_outcome;
mod publication_registry;
mod publication_status;
mod published_results_status;
mod readiness_views;
mod records;
mod remediation_queue;
mod replay;
mod report;
mod run_index;
mod schema_versions;
mod scope_views;
mod source_authority;
mod stage_result;
mod stage_tool_closure;
mod suite;
pub use active_scope::validate_active_scope_fast_validation_report;
pub use all_domain_parser_collector::validate_all_domain_parser_collector;
pub use appraisal::{validate_appraisal_report, validate_hardening_queue_report};
pub use benchmark_matrix::validate_benchmark_matrix_report;
pub use comparable_metrics::{
    validate_bam_comparable_metrics_report, validate_fastq_comparable_metrics_report,
    validate_vcf_comparable_metrics_report,
};
pub use compatibility::{
    migrate_benchmark_attempt_v1, parse_benchmark_attempt_document, parse_benchmark_cycle_document,
    validate_attempt_record_v1, validate_cycle_status_v1,
};
pub use container_report_portfolio::validate_container_report_portfolio_index;
pub use corpus_dossier::validate_benchmark_corpus_dossier_summary;
pub use corpus_expectations::validate_fastq_read_length_expectations;
pub use cross_campaign::validate_cross_benchmark_campaign_report;
pub use data_catalog_registry::validate_governed_data_catalog_registry;
pub use data_fitness::{validate_data_fitness_portfolio_alignment, validate_data_fitness_rubric};
pub use data_index::validate_benchmark_data_index;
pub use data_portfolio_audit::validate_benchmark_data_portfolio_audit;
pub use domain_campaign::{
    validate_bam_benchmark_campaign_report, validate_fastq_benchmark_campaign_report,
    validate_hardening_benchmark_campaign_report, validate_vcf_benchmark_campaign_report,
};
pub use dossier_index::validate_benchmark_dossier_index;
pub use essential_pipeline_readiness::validate_benchmark_essential_pipeline_readiness;
pub use execution::validate_execution_spec;
pub use interfaces::{
    validate_defined_interface_campaign_alignment, validate_interface_parameter_alignment,
    validate_pipeline_parameter_config, validate_stage_interface_catalog,
};
pub use lifecycle::{
    append_attempt_transition, benchmark_attempt_identity_sha256, benchmark_run_directory_name,
    validate_attempt_archive, validate_attempt_identity, validate_attempt_record_v2,
    validate_attempt_transition, validate_campaign_plan, validate_cycle_admission,
    validate_cycle_preparation, validate_cycle_status_v2, validate_scheduler_observation,
};
pub use local_benchmark_summary::validate_benchmark_local_summary;
pub use local_corpus_run::validate_benchmark_local_corpus_run;
pub use local_corpus_skip::validate_benchmark_local_corpus_skip_report;
pub use local_corpus_stage_compatibility::validate_benchmark_local_corpus_stage_compatibility_report;
pub use local_hpc_asset_staging::validate_benchmark_local_hpc_asset_staging_manifest;
pub use local_hpc_result_collection_simulation::validate_benchmark_local_hpc_result_collection_simulation_report;
pub use local_hpc_scratch_layout::validate_benchmark_local_hpc_scratch_layout;
pub use local_pipeline_dag::{
    validate_benchmark_local_pipeline_dag_validation,
    validate_benchmark_local_pipeline_dag_validation_set,
};
pub use local_runtime_metrics::validate_benchmark_local_runtime_metrics;
pub use local_slurm_dependency::{
    validate_benchmark_local_slurm_dependency_check, validate_benchmark_local_slurm_dry_run,
    validate_benchmark_local_slurm_shell_syntax,
};
pub use local_slurm_submit::validate_benchmark_local_slurm_submit_manifest;
pub use local_stage_commands::validate_benchmark_local_stage_commands;
pub use local_stage_completion::{
    validate_benchmark_local_stage_manifest_completion,
    validate_benchmark_local_stage_output_completion,
};
pub use local_stage_inventory::{
    validate_benchmark_local_all_domain_stage_inventory, validate_benchmark_local_stage_inventory,
};
pub use local_vcf_smoke::validate_benchmark_local_vcf_smoke_suite;
pub use materialization::{
    validate_corpus_acquisition_plan, validate_fastq_corpus_source_spec,
    validate_governed_data_catalog, validate_governed_data_catalog_document,
    validate_governed_preserved_data_catalog, validate_utc_timestamp,
};
pub use planning::{
    validate_benchmark_local_hpc_job_graph, validate_benchmark_portfolio,
    validate_benchmark_run_registry, validate_defined_campaign_portfolio_alignment,
    validate_portfolio_campaign_alignment, validate_scientific_readiness,
    validate_stage_campaign_registry, validate_stage_coverage, validate_stage_universe,
};
pub use protocol::{
    benchmark_run_definition_sha256, validate_benchmark_case_definition,
    validate_benchmark_run_definition, validate_benchmark_run_definition_revision,
};
pub use publication::{
    validate_benchmark_citation_record, validate_benchmark_evidence_book,
    validate_benchmark_publication_receipt,
};
pub use publication_findings::validate_benchmark_publication_findings;
pub use publication_outcome::{
    validate_benchmark_campaign_report_publication,
    validate_governed_data_catalog_registry_publication,
    validate_governed_data_catalog_snapshot_publication,
};
pub use publication_registry::validate_benchmark_publication_registry;
pub use publication_status::validate_benchmark_publication_status;
pub use published_results_status::validate_benchmark_published_results_status;
pub use readiness_views::{
    validate_corpus_centric_report, validate_stage_centric_report, validate_tool_centric_report,
};
pub use records::{
    validate_bundle_manifest, validate_corpus_manifest, validate_decision, validate_observation,
    validate_summary,
};
pub use remediation_queue::validate_benchmark_remediation_queue;
pub use replay::validate_benchmark_replay_receipt;
pub use report::{
    validate_report_admission, validate_stage_report, validate_stage_report_alignment,
};
pub use run_index::{
    validate_benchmark_protocol_citation, validate_benchmark_run_case_index,
    validate_benchmark_run_index, validate_benchmark_run_publication_pointer,
};
pub use schema_versions::{
    ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_V1, ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_V1,
    ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_V1, BAM_COMPARABLE_METRICS_SCHEMA_V1,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V1, BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SET_SCHEMA_V1, BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_LEDGER_SCHEMA_V1, BENCHMARK_ATTEMPT_MIGRATION_PLAN_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_VERIFICATION_SCHEMA_V1, BENCHMARK_ATTEMPT_SCHEMA_V1,
    BENCHMARK_ATTEMPT_SCHEMA_V2, BENCHMARK_CAMPAIGN_PLAN_SCHEMA_V1,
    BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1, BENCHMARK_CASE_AGGREGATION_SCHEMA_V1,
    BENCHMARK_CASE_AGGREGATION_SCHEMA_V2, BENCHMARK_CASE_AGGREGATION_SET_SCHEMA_V1,
    BENCHMARK_CASE_DEFINITION_SCHEMA_V1, BENCHMARK_CASE_MATRIX_SCHEMA_V1,
    BENCHMARK_CASE_MATRIX_SCHEMA_V2, BENCHMARK_CITATION_RECORD_SCHEMA_V1,
    BENCHMARK_CORPUS_MATERIALIZATION_PUBLICATION_SCHEMA_V1, BENCHMARK_CYCLE_ADMISSION_SCHEMA_V1,
    BENCHMARK_CYCLE_PREPARATION_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V1,
    BENCHMARK_CYCLE_STATUS_SCHEMA_V2, BENCHMARK_DATA_FITNESS_SCHEMA_V1,
    BENCHMARK_DATA_INDEX_SCHEMA_V1, BENCHMARK_DATA_INDEX_SCHEMA_V2,
    BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V1, BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2,
    BENCHMARK_DATA_SOURCE_SCHEMA_V1, BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
    BENCHMARK_ESSENTIAL_PIPELINE_READINESS_SCHEMA_V1, BENCHMARK_EVIDENCE_BOOK_SCHEMA_V1,
    BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1, BENCHMARK_EXECUTION_SPEC_SCHEMA_V1,
    BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1, BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1,
    BENCHMARK_LOCAL_CORPUS_SKIP_REPORT_SCHEMA_V1,
    BENCHMARK_LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1, BENCHMARK_LOCAL_HPC_JOB_GRAPH_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_RESULT_COLLECTION_SIMULATION_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_V1,
    BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1,
    BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SET_SCHEMA_V1,
    BENCHMARK_LOCAL_RUNTIME_METRICS_SCHEMA_V1, BENCHMARK_LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_V1,
    BENCHMARK_LOCAL_SLURM_DRY_RUN_SCHEMA_V1, BENCHMARK_LOCAL_SLURM_SHELL_SYNTAX_SCHEMA_V1,
    BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1, BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3,
    BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1, BENCHMARK_LOCAL_STAGE_MANIFEST_COMPLETION_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_OUTPUT_COMPLETION_SCHEMA_V1, BENCHMARK_LOCAL_SUMMARY_SCHEMA_V1,
    BENCHMARK_LOCAL_VCF_SMOKE_SUITE_SCHEMA_V1, BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1,
    BENCHMARK_PORTFOLIO_SCHEMA_V1, BENCHMARK_PORTFOLIO_SCHEMA_V2,
    BENCHMARK_PROTOCOL_CITATION_SCHEMA_V1, BENCHMARK_PUBLICATION_RECEIPT_SCHEMA_V1,
    BENCHMARK_PUBLICATION_REGISTRY_SCHEMA_V1, BENCHMARK_REPLAY_RECEIPT_SCHEMA_V1,
    BENCHMARK_REPORT_ADMISSION_SCHEMA_V1, BENCHMARK_RUN_CASE_INDEX_SCHEMA_V1,
    BENCHMARK_RUN_DEFINITION_SCHEMA_V1, BENCHMARK_RUN_INDEX_SCHEMA_V1,
    BENCHMARK_RUN_PUBLICATION_POINTER_SCHEMA_V1, BENCHMARK_RUN_REGISTRY_SCHEMA_V1,
    BENCHMARK_SCHEDULER_OBSERVATION_SCHEMA_V1, BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V1,
    BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V2, BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,
    BENCHMARK_STAGE_COVERAGE_SCHEMA_V1, BENCHMARK_STAGE_INTERFACES_SCHEMA_V1,
    BENCHMARK_STAGE_REPORT_SCHEMA_V1, BENCHMARK_STAGE_RESULT_SCHEMA_V2,
    BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1, BENCHMARK_STAGE_UNIVERSE_SCHEMA_V1,
    BENCHMARK_SUITE_RUN_MANIFEST_SCHEMA_V2, BUNDLE_SCHEMA_V1, CORPUS_ACQUISITION_SCHEMA_V1,
    CORPUS_ACQUISITION_SCHEMA_V2, CORPUS_CENTRIC_REPORT_SCHEMA_V1, CORPUS_SCHEMA_V1,
    DECISION_SCHEMA_V1, FASTQ_COMPARABLE_METRICS_SCHEMA_V1, FASTQ_CORPUS_SOURCE_SCHEMA_V1,
    FASTQ_PROFILE_READS_EXPECTATIONS_SCHEMA_V1, FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V1,
    FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V2, GOVERNED_DATA_CATALOG_REGISTRY_PUBLICATION_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_SCHEMA_V2, GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1,
    OBSERVATION_SCHEMA_V1, STAGE_CENTRIC_REPORT_SCHEMA_V1, STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_V1,
    SUITE_SCHEMA_V1, SUMMARY_SCHEMA_V1, TOOL_CENTRIC_REPORT_SCHEMA_V1,
    VCF_COMPARABLE_METRICS_SCHEMA_V1,
};
pub use scope_views::{
    validate_all_domain_active_stage_tool_matrix, validate_all_domain_retained_tools,
    validate_stage_registry_extra_pairs,
};
pub use source_authority::validate_benchmark_data_source_spec;
pub use stage_result::{
    validate_benchmark_stage_result_manifest, validate_benchmark_stage_result_validation_report,
};
pub use stage_tool_closure::validate_benchmark_stage_tool_closure;
pub use suite::validate_suite;
