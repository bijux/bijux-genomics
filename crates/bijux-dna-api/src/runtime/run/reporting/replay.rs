use super::Result;
use anyhow::anyhow;
use bijux_dna_core::contract::ExecutionGraph;
use bijux_dna_engine::Engine;
use bijux_dna_environment::api::RuntimeKind;
use bijux_dna_runner::{ApptainerRunner, DockerRunner, LocalRunner};
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
    let mut graph_value: serde_json::Value =
        serde_json::from_str(&graph_raw).map_err(|err| anyhow!("parse graph.json: {err}"))?;
    absolutize_graph_paths(base_dir, &mut graph_value)?;
    let graph: ExecutionGraph =
        serde_json::from_value(graph_value).map_err(|err| anyhow!("parse graph.json: {err}"))?;
    let runner = load_runner(base_dir)?;
    let layout = bijux_dna_runtime::run_layout::RunLayout::from_run_dir(base_dir.to_path_buf());
    Engine::default().execute(&graph, runner.as_ref(), &layout, None, None)?;
    Ok(())
}

fn absolutize_graph_paths(base_dir: &Path, graph: &mut serde_json::Value) -> Result<()> {
    let steps = graph
        .get_mut("steps")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| anyhow!("graph.json missing steps array"))?;
    for step in steps {
        absolutize_path_field(base_dir, step, "out_dir", false)?;
        let io = step
            .get_mut("io")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(|| anyhow!("graph.json step missing io object"))?;
        for artifact_key in ["inputs", "outputs"] {
            let artifacts = io
                .get_mut(artifact_key)
                .and_then(serde_json::Value::as_array_mut)
                .ok_or_else(|| anyhow!("graph.json io missing {artifact_key} array"))?;
            for artifact in artifacts {
                absolutize_path_field(base_dir, artifact, "path", artifact_key == "inputs")?;
            }
        }
    }
    Ok(())
}

fn absolutize_path_field(
    base_dir: &Path,
    object: &mut serde_json::Value,
    field: &str,
    search_ancestors_for_existing: bool,
) -> Result<()> {
    let value =
        object.get_mut(field).ok_or_else(|| anyhow!("graph.json object missing {field}"))?;
    let path_value =
        value.as_str().ok_or_else(|| anyhow!("graph.json field {field} must be a string path"))?;
    let path = Path::new(path_value);
    if path.is_relative() {
        let resolved =
            resolve_relative_path_from_ancestors(base_dir, path, search_ancestors_for_existing)
                .unwrap_or_else(|| base_dir.join(path));
        *value = serde_json::Value::String(resolved.to_string_lossy().into_owned());
    }
    Ok(())
}

fn resolve_relative_path_from_ancestors(
    base_dir: &Path,
    relative_path: &Path,
    require_existing_path: bool,
) -> Option<PathBuf> {
    if relative_path.as_os_str().is_empty() {
        return None;
    }
    if let Some(candidate) =
        resolve_anchored_relative_path(base_dir, relative_path, require_existing_path)
    {
        return Some(candidate);
    }
    if !require_existing_path {
        return None;
    }
    resolve_existing_relative_path(base_dir, relative_path)
}

fn resolve_existing_relative_path(base_dir: &Path, relative_path: &Path) -> Option<PathBuf> {
    let candidates = suffix_path_candidates(relative_path);
    base_dir.ancestors().find_map(|ancestor| {
        candidates.iter().find_map(|candidate| {
            let resolved = ancestor.join(candidate);
            resolved.exists().then_some(resolved)
        })
    })
}

fn resolve_anchored_relative_path(
    base_dir: &Path,
    relative_path: &Path,
    require_existing_path: bool,
) -> Option<PathBuf> {
    let components: Vec<Component<'_>> = relative_path.components().collect();
    if components.len() < 2 {
        return None;
    }
    for prefix_len in (1..components.len()).rev() {
        let prefix = &components[..prefix_len];
        let suffix = &components[prefix_len..];
        for ancestor in base_dir.ancestors() {
            if !path_ends_with_components(ancestor, prefix) {
                continue;
            }
            let mut candidate = ancestor.to_path_buf();
            for component in suffix {
                candidate.push(component.as_os_str());
            }
            if !require_existing_path || candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

fn path_ends_with_components(path: &Path, suffix: &[Component<'_>]) -> bool {
    let path_components: Vec<Component<'_>> = path.components().collect();
    if suffix.len() > path_components.len() {
        return false;
    }
    let offset = path_components.len() - suffix.len();
    path_components[offset..]
        .iter()
        .zip(suffix.iter())
        .all(|(left, right)| left.as_os_str() == right.as_os_str())
}

fn suffix_path_candidates(relative_path: &Path) -> Vec<PathBuf> {
    let components: Vec<Component<'_>> = relative_path.components().collect();
    let mut candidates = Vec::new();
    for start in 0..components.len() {
        let mut candidate = PathBuf::new();
        for component in components.iter().skip(start) {
            candidate.push(component.as_os_str());
        }
        if !candidate.as_os_str().is_empty() {
            candidates.push(candidate);
        }
    }
    if candidates.is_empty() {
        candidates.push(relative_path.to_path_buf());
    }
    candidates
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
        serde_json::from_str(&raw)
            .map_err(|err| anyhow!("parse executor_descriptor.json: {err}"))?;
    match descriptor.descriptor {
        bijux_dna_runtime::run_layout::ExecutorDescriptorV1::Local { .. } => {
            Ok(Box::new(LocalRunner::new(None)))
        }
        bijux_dna_runtime::run_layout::ExecutorDescriptorV1::Container { runtime, .. } => {
            match runtime.as_str() {
                "docker" => Ok(Box::new(DockerRunner::new(None))),
                "apptainer" => Ok(Box::new(ApptainerRunner::new(RuntimeKind::Apptainer, None))),
                "singularity" => Ok(Box::new(ApptainerRunner::new(RuntimeKind::Singularity, None))),
                _ => Err(anyhow!("replay does not support container runtime {runtime}")),
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
    use std::path::Path;

    use anyhow::anyhow;
    use bijux_dna_infra::{ensure_dir, write_bytes};

    use super::{
        resolve_graph_path, resolve_manifest_relative_path, resolve_relative_path_from_ancestors,
    };

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

    #[test]
    fn resolve_existing_relative_path_recovers_workspace_relative_inputs() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let workspace_root = tempfile::tempdir_in(temp.path())?;
        let declared_input = tempfile::Builder::new()
            .prefix("reads-")
            .suffix(".fastq")
            .tempfile_in(workspace_root.path())?;
        let declared_input_path = declared_input.path().to_path_buf();
        let relative_input = declared_input_path
            .strip_prefix(workspace_root.path())
            .map_err(|err| anyhow!("strip workspace prefix: {err}"))?;
        let run_dir = workspace_root.path().join("runs/run-123/artifacts");

        let recovered =
            resolve_relative_path_from_ancestors(run_dir.as_path(), relative_input, true)
                .ok_or_else(|| anyhow!("expected resolver to recover existing input path"))?;

        assert_eq!(recovered, declared_input_path);
        Ok(())
    }

    #[test]
    fn resolve_existing_relative_path_recovers_canonicalized_artifacts_tmp_tail(
    ) -> anyhow::Result<()> {
        let temp_root = std::env::current_dir()?;
        let temp = tempfile::tempdir_in(&temp_root)?;
        let workspace_root = tempfile::tempdir_in(temp.path())?;
        let runtime_root = workspace_root.path().join("artifacts/runtime-temp/.tmp123");
        ensure_dir(&runtime_root)?;
        let declared_input_path = runtime_root.join("reads.fastq");
        write_bytes(&declared_input_path, b"@read\nACGT\n+\n!!!!\n")?;
        let run_dir = runtime_root.join("runs/run-123");
        ensure_dir(&run_dir)?;
        let canonicalized_relative = Path::new("artifacts/runtime-temp/.tmp123/reads.fastq");

        let recovered =
            resolve_relative_path_from_ancestors(run_dir.as_path(), canonicalized_relative, true)
                .ok_or_else(|| anyhow!("expected resolver to recover canonicalized input path"))?;

        assert_eq!(recovered, declared_input_path);
        Ok(())
    }

    #[test]
    fn resolve_relative_path_from_ancestors_maps_output_tails_without_existing_target(
    ) -> anyhow::Result<()> {
        let temp_root = std::env::current_dir()?;
        let temp = tempfile::tempdir_in(&temp_root)?;
        let workspace_root = tempfile::tempdir_in(temp.path())?;
        let runtime_root = workspace_root.path().join("artifacts/runtime-temp/.tmp123");
        ensure_dir(&runtime_root)?;
        let run_dir = runtime_root.join("runs/run-123");
        ensure_dir(&run_dir)?;
        let output_tail = Path::new("artifacts/runtime-temp/.tmp123/validated.fastq");

        let resolved = resolve_relative_path_from_ancestors(run_dir.as_path(), output_tail, false)
            .ok_or_else(|| anyhow!("expected resolver to map canonicalized output tail"))?;

        assert_eq!(resolved, runtime_root.join("validated.fastq"));
        Ok(())
    }
}
