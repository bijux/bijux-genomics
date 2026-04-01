//! Owner: bijux-dna-bench
//! Run repository abstraction for bench.
//! Owns access to run metadata and metrics via run_index or facts.
//! Must not crawl filesystem trees.
//! Invariants: repository calls are deterministic.
#![allow(dead_code)]

use std::path::PathBuf;

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct RunMetadata {
    pub manifest_path: PathBuf,
    pub metrics_path: PathBuf,
}

pub trait RunRepository {
    fn list_runs(&self) -> Result<Vec<String>>;
    fn run_metadata(&self, run_id: &str) -> Result<RunMetadata>;
    fn load_observations(
        &self,
        run_id: &str,
    ) -> Result<Vec<bijux_dna_bench_model::BenchmarkObservation>>;
}
