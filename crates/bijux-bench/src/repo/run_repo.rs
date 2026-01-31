//! Owner: bijux-bench
//! Run repository abstraction for bench.
//! Owns access to run metadata and metrics via run_index or facts.
//! Must not crawl filesystem trees.
//! Invariants: repository calls are deterministic.

use std::path::PathBuf;

use anyhow::{Context, Result};

use std::collections::BTreeMap;

use crate::error::BenchError;

#[derive(Debug, Clone)]
pub struct RunMetadata {
    pub manifest_path: PathBuf,
    pub metrics_path: PathBuf,
}

pub trait RunRepository {
    fn run_metadata(&self, run_id: &str) -> Result<RunMetadata>;
}

pub fn load_manifest(path: &PathBuf) -> Result<bijux_engine::api::ExecutionManifest> {
    let bytes = std::fs::read(path).with_context(|| format!("read manifest {}", path.display()))?;
    let manifest: bijux_engine::api::ExecutionManifest = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse manifest {}", path.display()))?;
    Ok(manifest)
}

pub fn load_metrics(path: &PathBuf) -> Result<serde_json::Value> {
    if !path.exists() {
        return Err(BenchError::MissingMetrics(format!(
            "metrics file missing: {}",
            path.display()
        ))
        .into());
    }
    let bytes = std::fs::read(path).with_context(|| format!("read metrics {}", path.display()))?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub fn load_metrics_map(path: &PathBuf) -> Result<BTreeMap<String, f64>> {
    let value = load_metrics(path)?;
    let mut map = BTreeMap::new();
    if let serde_json::Value::Object(obj) = value {
        for (key, val) in obj {
            if let Some(num) = val.as_f64() {
                map.insert(key, num);
            }
        }
    }
    Ok(map)
}
