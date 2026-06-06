#![allow(dead_code)]

pub(crate) mod bam;
pub(crate) mod benchmark;
pub mod cli;
pub(crate) mod corpus;
pub(crate) mod ena;
pub(crate) mod fastq;
pub(crate) mod fixtures;
pub mod hpc;
pub(crate) mod planning;
mod router;
mod status;
pub(crate) mod support;
pub(crate) mod vcf;

pub(crate) use benchmark::config as benchmark_config;
pub(crate) use benchmark::corpus_fastq as benchmark_corpus_fastq;
pub(crate) use benchmark::corpus_metadata as benchmark_corpus_metadata;
pub(crate) use benchmark::fastq_bench as bench;
pub(crate) use benchmark::publication as benchmark_publication;
pub(crate) use benchmark::repo_checks as benchmark_repo_checks;
pub(crate) use benchmark::stage_catalog as benchmark_stage_catalog;
pub(crate) use benchmark::suite as bench_suite;
pub(crate) use benchmark::taxonomy_database as benchmark_taxonomy_database;
pub(crate) use benchmark::workspace as benchmark_workspace;
pub use router::argv::parse_process_cli;
pub use router::entrypoint::{run_with_args, run_with_cli};
pub(crate) use support::workspace_audit::workspace_audit;
