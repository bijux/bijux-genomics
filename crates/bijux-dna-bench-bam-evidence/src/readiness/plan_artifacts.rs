use crate::readiness::paths::path_relative_to_repo;
use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
pub(in crate::readiness) fn collect_plan_output_ids(
    payload: &serde_json::Value,
    path: &Path,
) -> Result<Vec<String>> {
    let outputs = payload
        .get("io")
        .and_then(|value| value.get("outputs"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing array `io.outputs`", path.display()))?;
    let mut output_ids = outputs
        .iter()
        .map(|artifact| {
            artifact
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| anyhow!("{} has output without string `name`", path.display()))
        })
        .collect::<Result<Vec<_>>>()?;
    output_ids.sort();
    Ok(output_ids)
}
pub(in crate::readiness) fn collect_plan_artifact_paths(
    repo_root: &Path,
    payload: &serde_json::Value,
    plan_path: &Path,
) -> Result<Vec<String>> {
    let outputs = payload
        .get("io")
        .and_then(|value| value.get("outputs"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing array `io.outputs`", plan_path.display()))?;

    let mut artifact_paths = vec![path_relative_to_repo(repo_root, plan_path)];
    for artifact in outputs {
        let path = artifact
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("{} has output without string `path`", plan_path.display()))?;
        if path.is_empty() {
            return Err(anyhow!("{} has empty output `path`", plan_path.display()));
        }
        artifact_paths.push(path.to_string());
    }
    artifact_paths.sort();
    artifact_paths.dedup();
    Ok(artifact_paths)
}
