#![cfg(test)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use cargo_metadata::{MetadataCommand, Package};

/// One guard's exact-manifest Cargo observation, reused for dependencies and test targets.
pub(crate) struct CargoPackageSnapshot {
    packages: BTreeMap<PathBuf, Package>,
}

impl CargoPackageSnapshot {
    pub(crate) fn capture(manifest: &Path) -> Self {
        let requested = std::path::absolute(manifest).unwrap_or_else(|error| {
            panic!("resolve Cargo dependency manifest {}: {error}", manifest.display())
        });
        let directory = requested.parent().unwrap_or_else(|| {
            panic!("Cargo dependency manifest has no parent: {}", manifest.display())
        });
        let metadata = MetadataCommand::new()
            .manifest_path(&requested)
            .current_dir(directory)
            .no_deps()
            .other_options(vec!["--locked".to_string()])
            .exec()
            .unwrap_or_else(|error| {
                panic!("read Cargo dependency authority {}: {error}", manifest.display())
            });
        let expected = resolve_manifest(manifest);
        let workspace_manifest =
            resolve_manifest(&metadata.workspace_root.as_std_path().join("Cargo.toml"));
        let mut packages = BTreeMap::new();
        for package in metadata.packages {
            let path = resolve_manifest(package.manifest_path.as_std_path());
            assert!(
                packages.insert(path, package).is_none(),
                "Cargo returned duplicate physical package manifests"
            );
        }
        assert!(
            packages.contains_key(&expected) || expected == workspace_manifest,
            "Cargo did not return requested manifest {}",
            manifest.display()
        );
        Self { packages }
    }

    pub(crate) fn package(&self, manifest: &Path) -> &Package {
        self.packages.get(&resolve_manifest(manifest)).unwrap_or_else(|| {
            panic!("Cargo observation has no requested manifest {}", manifest.display())
        })
    }
}

fn resolve_manifest(manifest: &Path) -> PathBuf {
    manifest.canonicalize().unwrap_or_else(|error| {
        panic!("resolve Cargo dependency manifest {}: {error}", manifest.display())
    })
}
