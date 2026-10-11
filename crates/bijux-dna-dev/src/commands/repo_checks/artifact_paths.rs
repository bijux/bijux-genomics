use std::path::{Component, Path};

use anyhow::{ensure, Context, Result};

use crate::runtime::workspace::Workspace;

const DESTINATIONS: &[(&str, &[&str])] = &[
    (
        "configs/ci/build-authority.json",
        &["/targets/shared_prebuild_root", "/targets/concurrent_target_pattern"],
    ),
    (
        "configs/ci/fast-groups.json",
        &["/report_root", "/timing_report", "/timing_history", "/baseline_report"],
    ),
    (
        "configs/ci/full-shards.json",
        &["/archive_root", "/shard_artifact_root", "/report_root", "/runtime_dependency_report"],
    ),
];

fn validate_destination(root: &Path, declared: &str) -> Result<()> {
    let mut components = Path::new(declared).components();
    ensure!(
        components.next() == Some(Component::Normal("artifacts".as_ref()))
            && components.clone().next().is_some()
            && components.all(|component| matches!(component, Component::Normal(_))),
        "destination must name a relative child of artifacts without parent traversal"
    );
    Workspace { root: root.to_path_buf() }.admit_artifact_path(Path::new(declared))?;
    Ok(())
}

pub(super) fn configured_artifact_path_offenders(workspace: &Workspace) -> Result<Vec<String>> {
    let mut offenders = Vec::new();
    for (source, pointers) in DESTINATIONS {
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(workspace.path(source))?)
                .with_context(|| format!("parse artifact destinations in {source}"))?;
        for pointer in *pointers {
            let Some(destination) = value.pointer(pointer).and_then(serde_json::Value::as_str)
            else {
                offenders.push(format!("{source}{pointer}: missing string destination"));
                continue;
            };
            if let Err(error) = validate_destination(&workspace.root, destination) {
                offenders.push(format!("{source}{pointer} ({destination}): {error:#}"));
            }
        }
    }
    Ok(offenders)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_relative_outputs_and_rejects_traversal_and_similar_prefixes() -> Result<()> {
        let root = tempfile::tempdir()?;
        for path in ["artifacts/ci/report.json", "artifacts/targets/{gate}"] {
            validate_destination(root.path(), path)?;
        }
        for path in [
            "",
            "artifacts",
            "artifacts-elsewhere/report",
            "artifacts/../report",
            "/artifacts/report",
            "runs/report",
        ] {
            assert!(validate_destination(root.path(), path).is_err(), "{path}");
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn rejects_existing_and_dangling_escape_links_and_preserves_them() -> Result<()> {
        let root = tempfile::tempdir()?;
        let outside = tempfile::tempdir()?;
        std::fs::create_dir(root.path().join("artifacts"))?;
        std::os::unix::fs::symlink(outside.path(), root.path().join("artifacts/escape"))?;
        std::os::unix::fs::symlink(
            outside.path().join("absent"),
            root.path().join("artifacts/dangling"),
        )?;
        for path in ["artifacts/escape/future/report.json", "artifacts/dangling/report.json"] {
            assert!(validate_destination(root.path(), path).is_err(), "{path}");
        }
        assert!(root.path().join("artifacts/escape").is_symlink());
        std::fs::create_dir(root.path().join("artifacts/reports"))?;
        std::os::unix::fs::symlink("reports", root.path().join("artifacts/local"))?;
        validate_destination(root.path(), "artifacts/local/future.json")?;
        Ok(())
    }

    #[test]
    fn reads_all_configured_destinations_and_refuses_missing_or_malformed_fields() -> Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        std::fs::create_dir_all(workspace.path("configs/ci"))?;
        for (source, pointers) in DESTINATIONS {
            let mut value = serde_json::json!({});
            for pointer in *pointers {
                let mut cursor = &mut value;
                let parts = pointer.trim_start_matches('/').split('/').collect::<Vec<_>>();
                for parent in &parts[..parts.len() - 1] {
                    if cursor[*parent].is_null() {
                        cursor[*parent] = serde_json::json!({});
                    }
                    cursor = &mut cursor[*parent];
                }
                cursor[parts[parts.len() - 1]] = serde_json::json!("artifacts/ci/result");
            }
            std::fs::write(workspace.path(source), serde_json::to_vec(&value)?)?;
        }
        assert!(configured_artifact_path_offenders(&workspace)?.is_empty());
        std::fs::write(
            workspace.path("configs/ci/fast-groups.json"),
            br#"{"report_root":"artifacts/../escape","timing_report":42}"#,
        )?;
        assert_eq!(configured_artifact_path_offenders(&workspace)?.len(), 4);
        Ok(())
    }
}
