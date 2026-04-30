//! Owner: bijux-dna-bench
//! Benchmark workspace path policy for the repository tree.

use std::path::PathBuf;

#[must_use]
pub fn bench_data_dir() -> PathBuf {
    super::resolve_repo_root()
        .map(|root| bijux_dna_infra::bench_data_dir(&root))
        .unwrap_or_else(|_| PathBuf::from("bench/data"))
}

#[must_use]
pub fn bench_suites_dir() -> PathBuf {
    super::resolve_repo_root()
        .map(|root| bijux_dna_infra::bench_suites_dir(&root))
        .unwrap_or_else(|_| PathBuf::from("bench/data/suites"))
}

#[must_use]
pub fn bench_corpora_dir() -> PathBuf {
    bench_data_dir().join("corpora")
}
