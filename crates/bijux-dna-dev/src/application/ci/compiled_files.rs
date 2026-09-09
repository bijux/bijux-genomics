//! Admission of local compiler products before metadata-only test execution.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use anyhow::{ensure, Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct CargoMetadata {
    workspace_root: PathBuf,
    target_directory: PathBuf,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct BinaryMetadata {
    rust_build_meta: BuildMetadata,
    rust_binaries: BTreeMap<String, TestBinary>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct BuildMetadata {
    target_directory: PathBuf,
    non_test_binaries: BTreeMap<String, Vec<WorkspaceBinary>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct TestBinary {
    binary_id: String,
    binary_path: PathBuf,
}

#[derive(Deserialize)]
struct WorkspaceBinary {
    path: PathBuf,
}

pub(crate) fn admit_compiled_executable_paths(
    cargo: &[u8],
    binaries: &[u8],
    source: &Path,
    target: &Path,
) -> Result<BTreeSet<PathBuf>> {
    let cargo: CargoMetadata =
        serde_json::from_slice(cargo).context("parse local Cargo metadata")?;
    let binaries: BinaryMetadata =
        serde_json::from_slice(binaries).context("parse local compiled binary metadata")?;
    ensure!(source.is_absolute() && target.is_absolute(), "compiler roots must be absolute");
    ensure!(cargo.workspace_root == source, "Cargo metadata belongs to a different source root");
    ensure!(
        cargo.target_directory == target && binaries.rust_build_meta.target_directory == target,
        "compiler metadata belongs to a different target root"
    );
    ensure!(!binaries.rust_binaries.is_empty(), "compiled inventory contains no test binaries");
    let mut paths = BTreeSet::new();
    for (id, binary) in binaries.rust_binaries {
        ensure!(!id.is_empty() && id == binary.binary_id, "ambiguous compiled test identity");
        let relative = binary
            .binary_path
            .strip_prefix(target)
            .context("compiled test executable escapes its target root")?;
        admit_relative_product(relative)?;
        ensure!(paths.insert(binary.binary_path), "multiple test identities share one executable");
    }
    for binary in binaries.rust_build_meta.non_test_binaries.into_values().flatten() {
        admit_relative_product(&binary.path)?;
        paths.insert(target.join(binary.path));
    }
    Ok(paths)
}

fn admit_relative_product(path: &Path) -> Result<()> {
    let value = path.to_str().context("compiled executable path must be UTF-8")?;
    ensure!(
        !value.is_empty()
            && !value.contains(['\\', '\0'])
            && value.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
            && path.components().all(|part| matches!(part, Component::Normal(_))),
        "compiled executable must be a normalized relative path"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn local_compiled_metadata_admits_test_and_workspace_executables() -> Result<()> {
        let source = std::env::current_dir()?;
        let target = source.join("artifacts/compiler");
        let cargo = json!({"workspace_root":source,"target_directory":target});
        let binary = target.join("debug/deps/owner-hash");
        let metadata = json!({
            "rust-build-meta":{"target-directory":target,"non-test-binaries":{
                "owner":[{"path":"debug/owner"}]
            }},
            "rust-binaries":{"owner":{"binary-id":"owner","binary-path":binary}}
        });
        let admitted = admit_compiled_executable_paths(
            &serde_json::to_vec(&cargo)?,
            &serde_json::to_vec(&metadata)?,
            &source,
            &target,
        )?;
        assert_eq!(admitted, [binary, target.join("debug/owner")].into_iter().collect());
        for pointer in ["/workspace_root", "/target_directory"] {
            let mut wrong = cargo.clone();
            wrong[pointer.trim_start_matches('/')] = json!(source.join("foreign"));
            assert!(admit_compiled_executable_paths(
                &serde_json::to_vec(&wrong)?,
                &serde_json::to_vec(&metadata)?,
                &source,
                &target,
            )
            .is_err());
        }
        for path in [source.join("outside"), target.join("../outside"), target.clone()] {
            let mut wrong = metadata.clone();
            wrong["rust-binaries"]["owner"]["binary-path"] = json!(path);
            assert!(admit_compiled_executable_paths(
                &serde_json::to_vec(&cargo)?,
                &serde_json::to_vec(&wrong)?,
                &source,
                &target,
            )
            .is_err());
        }
        let mut wrong = metadata;
        wrong["rust-binaries"]["owner"]["binary-id"] = json!("foreign");
        assert!(admit_compiled_executable_paths(
            &serde_json::to_vec(&cargo)?,
            &serde_json::to_vec(&wrong)?,
            &source,
            &target,
        )
        .is_err());
        Ok(())
    }
}
