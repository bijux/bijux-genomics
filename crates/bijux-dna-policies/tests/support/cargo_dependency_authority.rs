#![cfg(test)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use cargo_metadata::{Dependency, MetadataCommand};

/// One guard's exact-manifest Cargo observation, including all dependency kinds and targets.
pub(crate) struct CargoDependencySnapshot {
    packages: BTreeMap<PathBuf, Vec<Dependency>>,
}

impl CargoDependencySnapshot {
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
                packages.insert(path, package.dependencies).is_none(),
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

    pub(crate) fn dependencies(&self, manifest: &Path) -> &[Dependency] {
        self.packages.get(&resolve_manifest(manifest)).unwrap_or_else(|| {
            panic!("Cargo observation has no requested manifest {}", manifest.display())
        })
    }

    pub(crate) fn dependency_names(&self, manifest: &Path) -> Vec<String> {
        self.dependencies(manifest).iter().map(|dependency| dependency.name.clone()).collect()
    }

    pub(crate) fn known_names(
        &self,
        manifest: &Path,
        known: &BTreeSet<String>,
    ) -> BTreeSet<String> {
        self.dependencies(manifest)
            .iter()
            .filter(|dependency| known.contains(&dependency.name))
            .map(|dependency| dependency.name.clone())
            .collect()
    }
}

fn resolve_manifest(manifest: &Path) -> PathBuf {
    manifest.canonicalize().unwrap_or_else(|error| {
        panic!("resolve Cargo dependency manifest {}: {error}", manifest.display())
    })
}

// Fixture readers intentionally take a fresh observation after each authored manifest change.
// Real workspace guards instead retain one snapshot for their complete guard request.
pub(crate) fn read_dependency_names(manifest: &Path) -> Vec<String> {
    CargoDependencySnapshot::capture(manifest).dependency_names(manifest)
}
