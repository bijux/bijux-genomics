use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use bijux_dna_infra::bench_tools_dir;

use super::super::io::{hash_file_sha256, write_canonical_json};
use super::{RunArtifactInput, RunDirs};
use crate::recording::manifests::manifest_identity::manifest_sort_key;

pub(super) fn collect_all_run_artifacts(
    run_dirs: &RunDirs,
    extra_artifacts: &[RunArtifactInput],
) -> Result<Vec<serde_json::Value>> {
    let mut out = Vec::new();
    out.push(make_artifact_record(
        "execution_manifest",
        &run_dirs.manifest_path,
    )?);
    out.push(make_artifact_record("metrics", &run_dirs.metrics_path)?);
    for artifact in extra_artifacts {
        out.push(make_artifact_record(artifact.name, &artifact.path)?);
    }
    let run_artifacts = run_artifacts_dir(run_dirs)?;
    if run_artifacts.exists() {
        for path in collect_files_sorted(&run_artifacts)? {
            let rel = path
                .strip_prefix(&run_artifacts)
                .ok()
                .and_then(|p| p.to_str())
                .unwrap_or("artifact");
            let name = format!("run_artifacts/{rel}");
            out.push(make_artifact_record(&name, &path)?);
        }
    }
    out.sort_by_key(|artifact| manifest_sort_key(artifact, "name"));
    Ok(out)
}

fn make_artifact_record(name: &str, path: &Path) -> Result<serde_json::Value> {
    let hash = hash_file_sha256(path)?;
    Ok(serde_json::json!({
        "name": name,
        "path": path,
        "sha256": hash
    }))
}

fn collect_files_sorted(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root).with_context(|| format!("read dir {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_files_sorted(&path)?);
        } else if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// # Errors
/// Returns an error if JSON serialization or writing fails.
pub fn write_stage_plan_json<T: Serialize>(
    run_dirs: &RunDirs,
    file_name: &str,
    plan: &T,
) -> Result<PathBuf> {
    let root = run_artifacts_dir(run_dirs)?;
    let plans_dir = root.join("plans");
    bijux_dna_infra::ensure_dir(&plans_dir).context("create plans artifact dir")?;
    let path = plans_dir.join(file_name);
    bijux_dna_infra::ensure_dir(path.parent().unwrap_or(&plans_dir))
        .context("create plan parent dir")?;
    write_canonical_json(&path, plan).context("write stage plan json")?;
    Ok(path)
}

#[must_use]
pub fn run_artifacts_dir_for_out(out_dir: &Path) -> PathBuf {
    out_dir.join("run_artifacts")
}

#[allow(dead_code)]
#[must_use]
pub fn tool_run_artifacts_dir(
    out: &Path,
    stage: &str,
    sample_id: &str,
    tool: &str,
    run_id: &str,
) -> PathBuf {
    bench_tools_dir(out, stage, sample_id)
        .join(tool)
        .join("run")
        .join(run_id)
        .join("artifacts")
}

pub(super) fn run_artifacts_dir(run_dirs: &RunDirs) -> Result<PathBuf> {
    let run_dir = run_dirs
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("run dir missing for manifest"))?;
    Ok(run_dir.join("run_artifacts"))
}
