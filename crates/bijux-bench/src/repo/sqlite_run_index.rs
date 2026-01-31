//! Owner: bijux-bench
//! Run repository backed by bijux-core run_index.

use std::path::PathBuf;

use anyhow::{anyhow, Result};

use bijux_core::run_index::{query_run, RunIndexEntry};

use crate::repo::run_repo::{RunMetadata, RunRepository};

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
        let manifest_path = self.artifacts_root.join(&run.run_id).join("manifest.json");
        let metrics_path = self.artifacts_root.join(&run.run_id).join("metrics.json");
        RunMetadata {
            manifest_path,
            metrics_path,
        }
    }
}

impl RunRepository for RunIndexRepository {
    fn run_metadata(&self, run_id: &str) -> Result<RunMetadata> {
        let run = query_run(&self.index_path, run_id)?
            .ok_or_else(|| anyhow!("run_id {run_id} not found in run_index"))?;
        Ok(self.resolve_run(&run))
    }
}
