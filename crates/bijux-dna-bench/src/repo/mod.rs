//! Owner: bijux-dna-bench
//! Run repositories for bench.

mod bam_corpus_authority;
mod catalog_registry_authority;
mod catalog_snapshot_authority;
mod corpus_authority;
mod data_index_authority;
mod repo_root;
mod repository;
mod run_artifacts;
mod run_metadata;
pub mod sqlite;
mod workspace_paths;

pub use bam_corpus_authority::{
    admit_bam_benchmark_corpus, admit_bam_genotyping_corpus, AdmittedBamGenotypingCorpus,
};
pub use catalog_registry_authority::{
    publish_governed_data_catalog_registry, GovernedDataCatalogRegistryPublication,
    GovernedDataCatalogRegistryPublicationRequest,
};
pub use catalog_snapshot_authority::{
    publish_governed_data_catalog_snapshots, GovernedDataCatalogSnapshotPublication,
    GovernedDataCatalogSnapshotRequest,
};
pub use corpus_authority::admit_fastq_benchmark_corpus;
pub use data_index_authority::{
    build_benchmark_data_index, publish_benchmark_data_index, render_benchmark_data_index_markdown,
    BenchmarkDataIndexPublicationRequest, BenchmarkDataIndexRequest,
};
pub use repo_root::resolve_repo_root;
pub use repository::RunRepository;
pub(crate) use run_artifacts::load_observations;
pub use run_metadata::RunMetadata;
pub use workspace_paths::{
    bench_bam_local_stage_matrix_path, bench_bundles_dir, bench_contracts_dir, bench_corpora_dir,
    bench_data_dir, bench_fastq_local_stage_matrix_path, bench_local_config_dir,
    bench_portfolio_path, bench_reports_dir, bench_stage_campaigns_path,
    bench_stage_interfaces_path, bench_suites_dir,
};
