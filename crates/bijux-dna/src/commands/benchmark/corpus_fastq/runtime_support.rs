use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub(super) fn path_display(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(super) fn benchmark_sample_root(out_root: &Path, report_dir: &str, sample_id: &str) -> PathBuf {
    out_root.join("bench").join(report_dir).join(sample_id)
}

pub(super) fn reset_sample_payload(sample_root: &Path) -> Result<()> {
    if sample_root.is_dir() {
        fs::remove_dir_all(sample_root)
            .with_context(|| format!("remove stale sample payload {}", sample_root.display()))?;
    }
    Ok(())
}

pub(super) fn sample_report_is_resume_ready(sample_report: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(sample_report) else {
        return false;
    };
    let Ok(payload) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    if payload
        .get("failures")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|row| !row.is_empty())
    {
        return false;
    }
    if payload.get("gate").and_then(|row| row.get("passes")).and_then(serde_json::Value::as_bool)
        == Some(false)
    {
        return false;
    }
    payload.get("records").and_then(serde_json::Value::as_array).is_some_and(|row| !row.is_empty())
}

pub(super) fn benchmark_runtime_env(out_root: &Path) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    let Some(cache_root) = workspace_cache_root_for_output(out_root) else {
        return env;
    };
    env.insert("BIJUX_CACHE_ROOT".to_string(), cache_root.display().to_string());
    env.insert("XDG_CACHE_HOME".to_string(), cache_root.display().to_string());
    env
}

pub(super) fn workspace_cache_root_for_output(out_root: &Path) -> Option<PathBuf> {
    let resolved = out_root.canonicalize().unwrap_or_else(|_| out_root.to_path_buf());
    for candidate in resolved.ancestors() {
        if candidate.file_name().and_then(|row| row.to_str()) == Some(".cache") {
            return Some(candidate.to_path_buf());
        }
    }
    None
}

pub(super) fn absolutize(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

pub(super) fn current_timestamp_utc() -> Result<String> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("resolve benchmark timestamp")?;
    Ok(format!("unix:{}", elapsed.as_secs()))
}
