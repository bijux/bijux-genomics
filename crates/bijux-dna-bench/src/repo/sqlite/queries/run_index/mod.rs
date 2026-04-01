//! Owner: bijux-dna-bench
//! Run repository backed by bijux-dna-core run_index.
#![allow(dead_code)]

use std::path::PathBuf;

use anyhow::{anyhow, Result};

use bijux_dna_core::contract::{list_runs, query_run, RunIndexEntry};

use crate::repo::{load_observations, RunMetadata, RunRepository};

mod metadata_paths;

#[derive(Debug, Clone)]
pub struct RunIndexRepository {
    index_path: PathBuf,
    artifacts_root: PathBuf,
}

impl RunIndexRepository {
    #[must_use]
    pub fn new(index_path: PathBuf, artifacts_root: PathBuf) -> Self {
        Self {
            index_path,
            artifacts_root,
        }
    }

    fn resolve_run(&self, run: &RunIndexEntry) -> RunMetadata {
        metadata_paths::resolve_run_metadata(&self.artifacts_root, run)
    }
}

impl RunRepository for RunIndexRepository {
    fn list_runs(&self) -> Result<Vec<String>> {
        let entries = list_runs(&self.index_path)?;
        Ok(entries
            .into_iter()
            .map(|entry| entry.run_id.to_string())
            .collect())
    }

    fn run_metadata(&self, run_id: &str) -> Result<RunMetadata> {
        let run = query_run(&self.index_path, run_id)?
            .ok_or_else(|| anyhow!("run_id {run_id} not found in run_index"))?;
        Ok(self.resolve_run(&run))
    }

    fn load_observations(
        &self,
        run_id: &str,
    ) -> Result<Vec<bijux_dna_bench_model::BenchmarkObservation>> {
        let observations_path = self.artifacts_root.join(run_id).join("observations.jsonl");
        load_observations(&observations_path)
    }
}
