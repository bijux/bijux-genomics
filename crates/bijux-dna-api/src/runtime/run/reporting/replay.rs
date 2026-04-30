use super::Result;
use anyhow::anyhow;
use bijux_dna_core::contract::ExecutionGraph;
use bijux_dna_engine::Engine;
use bijux_dna_runner::{DockerRunner, LocalRunner};
use std::path::{Component, Path, PathBuf};

/// Replay or verify a run from a run manifest.
///
/// # Errors
/// Returns an error if manifest parsing, graph loading, execution, or verification fails.
pub fn replay_manifest(manifest_path: &Path, verify_only: bool) -> Result<()> {
    let raw = std::fs::read_to_string(manifest_path)
        .map_err(|err| anyhow!("read run_manifest.json: {err}"))?;
    let manifest: serde_json::Value =
        serde_json::from_str(&raw).map_err(|err| anyhow!("parse run_manifest.json: {err}"))?;
    let base_dir =
        manifest_path.parent().ok_or_else(|| anyhow!("run_manifest.json missing parent"))?;
    let artifacts = manifest
        .get("output_artifacts")
        .and_then(|v| v.as_array())
        .cloned()
        .ok_or_else(|| anyhow!("run_manifest.json missing output_artifacts array"))?;
    if verify_only {
        for entry in artifacts {
            let Some(path_value) = entry.get("path") else {
                continue;
            };
            let Some(path_str) = path_value.as_str() else {
                continue;
            };
            let path = resolve_manifest_relative_path(base_dir, path_str)?;
            if !path.exists() {
                return Err(anyhow!("missing output artifact {}", path.display()));
            }
            if let Some(expected) = entry.get("sha256").and_then(|v| v.as_str()) {
                let actual = bijux_dna_infra::hash_file_sha256(&path)?;
                if actual != expected {
                    return Err(anyhow!(
                        "artifact hash mismatch for {} (expected {}, got {})",
                        path.display(),
                        expected,
                        actual
                    ));
                }
            }
        }
        return Ok(());
    }
    let graph_path = resolve_graph_path(base_dir, &artifacts)?;
    let graph_raw =
        std::fs::read_to_string(&graph_path).map_err(|err| anyhow!("read graph.json: {err}"))?;
    let graph: ExecutionGraph =
        serde_json::from_str(&graph_raw).map_err(|err| anyhow!("parse graph.json: {err}"))?;
    let runner = load_runner(base_dir)?;
    let layout = bijux_dna_runtime::run_layout::RunLayout::from_run_dir(base_dir.to_path_buf());
    Engine::default().execute(&graph, runner.as_ref(), &layout, None, None)?;
    Ok(())
}

fn resolve_graph_path(base_dir: &Path, artifacts: &[serde_json::Value]) -> Result<PathBuf> {
    artifacts
        .iter()
        .find_map(|entry| {
            let is_graph = entry.get("kind").and_then(|value| value.as_str()) == Some("graph");
            let path = entry.get("path").and_then(|value| value.as_str())?;
            is_graph.then_some(path)
        })
        .map_or_else(
            || {
                Ok(bijux_dna_runtime::recording::run_artifacts_dir_for_out(base_dir)
                    .join("graph.json"))
            },
            |path| resolve_manifest_relative_path(base_dir, path),
        )
}

fn load_runner(base_dir: &Path) -> Result<Box<dyn bijux_dna_runtime::Runner>> {
    let descriptor_path = base_dir.join("executor_descriptor.json");
    if !descriptor_path.exists() {
        return Ok(Box::new(DockerRunner::new(None)));
    }
    let raw = std::fs::read_to_string(&descriptor_path)
        .map_err(|err| anyhow!("read executor_descriptor.json: {err}"))?;
    let descriptor: bijux_dna_runtime::run_layout::RunExecutorDescriptorV1 =
        serde_json::from_str(&raw).map_err(|err| anyhow!("parse executor_descriptor.json: {err}"))?;
    match descriptor.descriptor {
        bijux_dna_runtime::run_layout::ExecutorDescriptorV1::Local { .. } => {
            Ok(Box::new(LocalRunner::new(None)))
        }
        bijux_dna_runtime::run_layout::ExecutorDescriptorV1::Container { runtime, .. } => {
            if runtime == "docker" {
                Ok(Box::new(DockerRunner::new(None)))
            } else {
                Err(anyhow!("replay does not support container runtime {runtime}"))
            }
        }
        bijux_dna_runtime::run_layout::ExecutorDescriptorV1::Hpc { scheduler, .. } => {
            Err(anyhow!("replay does not support hpc scheduler {scheduler}"))
        }
    }
}

fn resolve_manifest_relative_path(base_dir: &Path, path: &str) -> Result<PathBuf> {
    if path.is_empty() {
        return Err(anyhow!("output artifact path must not be empty"));
    }
    let relative_path = Path::new(path);
    if relative_path.is_absolute() {
        return Err(anyhow!("output artifact path must be relative: {path}"));
    }
    if relative_path.components().any(|component| {
        matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
    }) {
        return Err(anyhow!("output artifact path must stay within run directory: {path}"));
    }
    Ok(base_dir.join(relative_path))
}

#[cfg(test)]
mod tests {
    use super::{resolve_graph_path, resolve_manifest_relative_path};

    #[test]
    fn resolve_graph_path_uses_manifest_graph_artifact() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let artifacts = vec![serde_json::json!({
            "kind": "graph",
            "path": "graph.json",
            "schema": "bijux.execution_graph.v1"
        })];

        let graph_path = resolve_graph_path(temp.path(), &artifacts)?;

        assert_eq!(graph_path, temp.path().join("graph.json"));
        Ok(())
    }

    #[test]
    fn resolve_graph_path_keeps_legacy_artifact_location() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;

        let graph_path = resolve_graph_path(temp.path(), &[])?;

        assert_eq!(
            graph_path,
            bijux_dna_runtime::recording::run_artifacts_dir_for_out(temp.path()).join("graph.json")
        );
        Ok(())
    }

    #[test]
    fn resolve_manifest_relative_path_rejects_parent_traversal() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;

        let err = match resolve_manifest_relative_path(temp.path(), "../outside.json") {
            Ok(path) => panic!("parent traversal should fail: {}", path.display()),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains("must stay within run directory"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn resolve_manifest_relative_path_rejects_absolute_paths() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;

        let err = match resolve_manifest_relative_path(temp.path(), "/outside.json") {
            Ok(path) => panic!("absolute path should fail: {}", path.display()),
            Err(err) => err,
        };

        assert!(err.to_string().contains("must be relative"), "unexpected error: {err}");
        Ok(())
    }
}
