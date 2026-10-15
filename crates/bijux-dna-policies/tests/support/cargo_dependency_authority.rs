#![cfg(test)]

use std::collections::BTreeSet;
use std::path::Path;

use cargo_metadata::Dependency;

#[path = "cargo_metadata_authority.rs"]
mod metadata;

pub(crate) use metadata::CargoPackageSnapshot as CargoDependencySnapshot;

impl CargoDependencySnapshot {
    pub(crate) fn dependencies(&self, manifest: &Path) -> &[Dependency] {
        &self.package(manifest).dependencies
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

// Fixture readers intentionally take a fresh observation after each authored manifest change.
// Real workspace guards instead retain one snapshot for their complete guard request.
pub(crate) fn read_dependency_names(manifest: &Path) -> Vec<String> {
    CargoDependencySnapshot::capture(manifest).dependency_names(manifest)
}
