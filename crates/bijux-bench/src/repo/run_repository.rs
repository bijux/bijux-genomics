//! Owner: bijux-bench
//! Run repository abstraction for bench.
//! Owns access to run metadata and metrics via run_index or facts.
//! Must not crawl filesystem trees.
//! Invariants: repository calls are deterministic.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use bijux_core::run_index::{query_run, RunIndexEntry};

#[derive(Debug, Clone)]
pub struct RunMetadata {
    pub run_id: String,
    pub manifest_path: PathBuf,
    pub metrics_path: PathBuf,
}

pub trait RunRepository {
    fn run_metadata(&self, run_id: &str) -> Result<RunMetadata>;
}

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
            run_id: run.run_id.clone(),
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

#[derive(Debug, Clone)]
pub struct FactsJsonlRepository {
    facts_path: PathBuf,
}

impl FactsJsonlRepository {
    #[must_use]
    pub fn new(facts_path: PathBuf) -> Self {
        Self { facts_path }
    }
}

impl RunRepository for FactsJsonlRepository {
    fn run_metadata(&self, run_id: &str) -> Result<RunMetadata> {
        let report_path = self
            .facts_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(""))
            .join("report.json");
        let manifest_path = self
            .facts_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(""))
            .join("manifest.json");
        if !report_path.exists() {
            return Err(anyhow!(
                "facts repository missing report.json for run {run_id}"
            ));
        }
        Ok(RunMetadata {
            run_id: run_id.to_string(),
            manifest_path,
            metrics_path: report_path,
        })
    }
}

pub fn load_manifest(path: &PathBuf) -> Result<bijux_engine::api::ExecutionManifest> {
    let bytes = std::fs::read(path).with_context(|| format!("read manifest {}", path.display()))?;
    let manifest: bijux_engine::api::ExecutionManifest = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse manifest {}", path.display()))?;
    Ok(manifest)
}

pub fn load_metrics(path: &PathBuf) -> Result<serde_json::Value> {
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let bytes = std::fs::read(path).with_context(|| format!("read metrics {}", path.display()))?;
    Ok(serde_json::from_slice(&bytes)?)
}
