//! Transport-independent paths for executing an admitted compiled test inventory.

use anyhow::{ensure, Context, Result};
use std::path::{Component, Path, PathBuf};

pub(crate) struct NextestReuseInputs {
    cargo_metadata: PathBuf,
    binaries_metadata: PathBuf,
    target_root: PathBuf,
}

impl NextestReuseInputs {
    pub(crate) fn new(
        cargo_metadata: PathBuf,
        binaries_metadata: PathBuf,
        target_root: PathBuf,
    ) -> Result<Self> {
        for path in [&cargo_metadata, &binaries_metadata, &target_root] {
            validate_path(path)?;
        }
        ensure!(
            cargo_metadata != binaries_metadata,
            "Cargo and executable metadata must have distinct identities"
        );
        Ok(Self { cargo_metadata, binaries_metadata, target_root })
    }

    pub(crate) fn arguments(&self, workspace: &Path) -> Result<Vec<String>> {
        validate_path(workspace)?;
        let mut arguments = Vec::with_capacity(8);
        for (flag, path) in [
            ("--cargo-metadata", self.cargo_metadata.as_path()),
            ("--binaries-metadata", self.binaries_metadata.as_path()),
            ("--target-dir-remap", self.target_root.as_path()),
            ("--workspace-remap", workspace),
        ] {
            arguments.push(flag.to_owned());
            arguments.push(path.to_str().context("test reuse path must be UTF-8")?.to_owned());
        }
        Ok(arguments)
    }
}

fn validate_path(path: &Path) -> Result<()> {
    ensure!(
        path.is_absolute()
            && !path
                .components()
                .any(|part| matches!(part, Component::ParentDir | Component::CurDir)),
        "test reuse paths must be absolute and normalized: {}",
        path.display()
    );
    ensure!(
        path.to_str().is_some_and(|path| !path.contains('\0')),
        "test reuse path must be UTF-8 without NUL bytes"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nextest_reuse_accepts_local_and_extracted_products_without_transport_flags() -> Result<()> {
        let root = std::env::current_dir()?.join("artifacts/test products");
        for target in [root.join("compile-cache/cargo"), root.join("extracted/target")] {
            let inputs = NextestReuseInputs::new(
                root.join("cargo.json"),
                root.join("binaries.json"),
                target.clone(),
            )?;
            let args = inputs.arguments(&root.join("source"))?;
            assert_eq!(args.len(), 8);
            assert_eq!(args[5], target.display().to_string());
            assert!(!args.iter().any(|arg| [
                "--archive-file",
                "--extract-to",
                "--workspace",
                "--all-targets"
            ]
            .contains(&arg.as_str())));
        }
        Ok(())
    }

    #[test]
    fn nextest_reuse_rejects_ambiguous_or_escaping_input_paths() -> Result<()> {
        let root = std::env::current_dir()?;
        assert!(NextestReuseInputs::new(
            root.join("same.json"),
            root.join("same.json"),
            root.join("target")
        )
        .is_err());
        for invalid in [PathBuf::from("relative"), root.join("../outside")] {
            assert!(NextestReuseInputs::new(
                invalid.clone(),
                root.join("binaries.json"),
                root.join("target")
            )
            .is_err());
            let inputs = NextestReuseInputs::new(
                root.join("cargo.json"),
                root.join("binaries.json"),
                root.join("target"),
            )?;
            assert!(inputs.arguments(&invalid).is_err());
        }
        Ok(())
    }
}
