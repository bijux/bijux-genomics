//! Owner: bijux-dna-bench
//! Benchmark workflow facade over summary, evaluation, suite loading, and suite persistence.
#![allow(dead_code)]

mod evaluation;
mod options;
mod run_suite;
mod suite_load;
mod summary;
mod summary_fairness;
mod summary_scope;
mod summary_statistics;

pub use evaluation::{compare, gate};
pub use options::BenchRunOptions;
pub use suite_load::{load_corpus_catalog, load_corpus_manifest, load_suite};
pub use summary::summarize;
