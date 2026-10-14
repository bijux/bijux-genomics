use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use anyhow::{ensure, Result};

use crate::support::RepoSandbox;

/// Component owners consumed by all-domain command rendering. Source crates and generated
/// readiness products are excluded; reference providers include metadata-only directories.
const COMPONENTS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "crates/bijux-dna/Cargo.toml",
    "configs/bench",
    "configs/runtime",
    "configs/ci/registry",
    "configs/ci/tools",
    "configs/vcf",
    "domain/fastq",
    "domain/bam",
    "domain/vcf",
    "containers/contracts",
    "containers/docker",
    "containers/apptainer",
    "benchmarks/contracts",
    "benchmarks/corpora",
    "benchmarks/tests/fixtures/corpora",
    "assets/reference",
];

pub(super) fn new(label: &str) -> Result<RepoSandbox> {
    RepoSandbox::new_selected(label, select_inputs)
}

fn select_inputs(
    indexed: &[PathBuf],
    read: &dyn Fn(&Path) -> Result<Vec<u8>>,
) -> Result<Vec<PathBuf>> {
    let mut components = COMPONENTS.iter().map(PathBuf::from).collect::<BTreeSet<_>>();
    for path in indexed {
        if path.file_name().is_some_and(|name| name == "manifest.toml")
            && path.starts_with("benchmarks/tests/fixtures/corpora")
        {
            let manifest: toml::Value = toml::from_str(std::str::from_utf8(&read(path)?)?)?;
            collect_asset_components(&manifest, &mut components)?;
        }
    }
    for component in &components {
        ensure!(
            indexed.iter().any(|path| path.starts_with(component)),
            "adapter fixture component is absent from indexed inputs: {}",
            component.display()
        );
    }
    Ok(indexed
        .iter()
        .filter(|path| components.iter().any(|component| path.starts_with(component)))
        .cloned()
        .collect())
}

fn collect_asset_components(value: &toml::Value, components: &mut BTreeSet<PathBuf>) -> Result<()> {
    match value {
        toml::Value::String(value) if value.starts_with("assets/") => {
            let path = Path::new(value);
            ensure!(
                path.components().all(|part| matches!(part, Component::Normal(_))),
                "adapter fixture asset declaration escapes repository: {value}"
            );
            // Retain the declared asset family so sidecars and directory providers share
            // their governed identity without guessing which bytes a loader will open.
            let component = path.components().take(3).collect::<PathBuf>();
            ensure!(
                component.components().count() == 3,
                "adapter fixture asset declaration lacks an owned family: {value}"
            );
            components.insert(component);
        }
        toml::Value::Array(values) => {
            for value in values {
                collect_asset_components(value, components)?;
            }
        }
        toml::Value::Table(values) => {
            for value in values.values() {
                collect_asset_components(value, components)?;
            }
        }
        _ => {}
    }
    Ok(())
}
