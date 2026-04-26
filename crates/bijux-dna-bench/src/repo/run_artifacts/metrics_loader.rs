//! Owner: bijux-dna-bench
//! Loader for finished benchmark metric payloads.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result};

use bijux_dna_bench_model::BenchError;

#[allow(dead_code)]
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

#[allow(dead_code)]
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
