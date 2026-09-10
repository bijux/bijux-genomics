//! Owner: bijux-dna-bench
//! Run repositories for bench.

mod fastq_score_evidence;
pub(crate) use fastq_score_evidence::load_optional_fastq_score_report;

mod case_data_authority;
mod catalog_registry_authority;
mod catalog_snapshot_authority;
mod corpus;
mod data_index_authority;
mod pipeline_config;
mod pipeline_execution;
mod pipeline_inventory;
mod pipeline_reference;
mod reference_fasta;
mod repo_root;
mod repository;
mod run_artifacts;
mod run_metadata;
pub mod sqlite;
mod stage_inventory;
mod workspace_paths;

pub use case_data_authority::derive_benchmark_data_snapshots;
pub use catalog_registry_authority::{
    publish_governed_data_catalog_registry, GovernedDataCatalogRegistryPublication,
    GovernedDataCatalogRegistryPublicationRequest,
};
pub use catalog_snapshot_authority::{
    admit_governed_derived_data_catalog, publish_governed_data_catalog_snapshots,
    publish_governed_materialized_data_catalog_snapshots, GovernedDataCatalogSnapshotPublication,
    GovernedDataCatalogSnapshotRequest, GovernedDerivedDataCatalogAdmission,
    GovernedDerivedDataCatalogAdmissionRequest, GovernedMaterializedDataCatalogSnapshotRequest,
};
pub use corpus::{
    admit_alignment_truth_set, admit_bam_benchmark_cohort, admit_bam_benchmark_corpus,
    admit_bam_genotyping_corpus, admit_benchmark_indexed_reference, admit_benchmark_reference,
    admit_fastq_benchmark_corpus, discover_normalized_fastq_samples,
    expected_fastq_publication_cohort_counts, load_fastq_corpus_source_for_id,
    match_normalized_fastq_corpus_metadata, normalized_fastq_expected_sample_total,
    select_normalized_fastq_paired_samples, AdmittedBamGenotypingCorpus, NormalizedFastqSample,
};
pub use data_index_authority::{
    build_benchmark_data_index, publish_benchmark_data_index, render_benchmark_data_index_markdown,
    BenchmarkDataIndexPublicationRequest, BenchmarkDataIndexRequest,
};
pub use pipeline_config::{
    benchmark_local_pipeline_config_path, load_benchmark_local_pipeline_dag_config,
};
pub use pipeline_reference::validate_benchmark_pipeline_reference_context;
pub use repo_root::resolve_repo_root;
pub use repository::RunRepository;
pub(crate) use run_artifacts::load_observations;
pub use run_metadata::RunMetadata;
pub use stage_inventory::{
    load_benchmark_local_stage_inventory, BenchmarkLocalStageInventoryDomain,
};
pub use workspace_paths::{
    bench_bam_local_stage_matrix_path, bench_bundles_dir, bench_contracts_dir, bench_corpora_dir,
    bench_data_dir, bench_data_fitness_path, bench_fastq_local_stage_matrix_path,
    bench_local_config_dir, bench_portfolio_path, bench_reports_dir, bench_stage_campaigns_path,
    bench_stage_interfaces_path, bench_suites_dir, DEFAULT_VCF_STAGE_MATRIX_PATH,
};

pub use pipeline_execution::validate_benchmark_pipeline_execution_context;
pub use pipeline_inventory::{
    load_benchmark_pipeline_inventory_index, load_benchmark_pipeline_vcf_stage_matrix_index,
};

pub use reference_fasta::materialize_benchmark_reference_fasta;

mod bam_score_evidence;
pub use bam_score_evidence::load_bam_tool_score_evidence;

mod vcf_score_evidence;
pub use vcf_score_evidence::load_vcf_tool_score_evidence;

mod vcf_execution_proof;
pub use vcf_execution_proof::load_vcf_execution_blocker;

#[cfg(feature = "container-qualification")]
pub(crate) mod container_qualification;

pub(crate) mod hpc_campaign;

pub(crate) mod simulation;

pub(crate) mod command_audit;

pub(crate) mod command_artifacts;

pub(crate) mod micro_run;

pub(crate) mod micro_report;

pub(crate) mod stage_result;

pub(crate) mod hpc_array;

pub(crate) mod hpc_resources;

pub(crate) mod hardening_campaign;

pub(crate) mod hpc_inputs;

pub(crate) mod hpc_asset_staging;

pub(crate) mod source_assets;

pub(crate) mod hpc_scratch;

pub(crate) mod runtime_probe;

pub(crate) mod tool_execution_modes;

pub mod alignment_fixture;
