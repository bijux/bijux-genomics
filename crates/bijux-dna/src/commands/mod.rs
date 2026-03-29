#![allow(dead_code)]

pub(crate) mod bam;
pub(crate) mod bench;
pub(crate) mod bench_suite;
pub(crate) mod benchmark_config;
pub(crate) mod benchmark_corpus_fastq;
pub(crate) mod benchmark_corpus_metadata;
pub(crate) mod benchmark_publication;
pub(crate) mod benchmark_repo_checks;
pub(crate) mod benchmark_stage_catalog;
pub(crate) mod benchmark_taxonomy_database;
pub(crate) mod benchmark_workspace;
pub mod cli;
mod command_dispatch;
pub(crate) mod command_prelude;
pub(crate) mod corpus;
pub(crate) mod ena;
pub(crate) mod fastq;
pub mod hpc;
mod policies;
pub(crate) mod profile_runtime;
pub(crate) mod repo_root;
pub(crate) mod report_inputs;
mod root;
mod router;
pub(crate) mod run_plan;
mod status;
pub(crate) mod vcf;

pub use command_dispatch::{run_with_args, run_with_cli};
pub use router::argv::{parse_cli_from_argv, parse_process_cli};
pub(crate) use policies::workspace_audit;
