use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use anyhow::{ensure, Result};

use super::RepoSandbox;

// These commands publish all-domain declarations before filtering BAM rows and acquire
// downstream VCF proofs. Keep their governed catalogs, native fixtures and providers;
// unrelated source, documentation and generated products are excluded.
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
    "benchmarks/configs/pipelines/local",
    "benchmarks/corpora",
    "benchmarks/tests/fixtures/corpora",
    "benchmarks/tests/fixtures/bench/parsers/vcf",
    "assets/reference",
];

#[allow(dead_code)]
impl RepoSandbox {
    pub fn new_bam_genotype(label: &str) -> Result<Self> {
        Self::new_selected(label, Self::bam_genotype_inputs)
    }

    pub(crate) fn bam_genotype_inputs(
        indexed: &[PathBuf],
        read: &dyn Fn(&Path) -> Result<Vec<u8>>,
    ) -> Result<Vec<PathBuf>> {
        let mut components = COMPONENTS.iter().map(PathBuf::from).collect::<BTreeSet<_>>();
        let mut manifests = 0;
        for path in indexed {
            if path.starts_with("benchmarks/tests/fixtures/corpora")
                && path.file_name().is_some_and(|name| name == "manifest.toml")
            {
                let manifest: toml::Value = toml::from_str(std::str::from_utf8(&read(path)?)?)?;
                collect_assets(&manifest, indexed, &mut components)?;
                manifests += 1;
            }
        }
        ensure!(manifests > 0, "genotype fixture corpus declarations are absent");
        for component in &components {
            ensure!(
                indexed.iter().any(|path| path.starts_with(component)),
                "genotype fixture component is not indexed: {}",
                component.display()
            );
        }
        Ok(indexed
            .iter()
            .filter(|path| components.iter().any(|component| path.starts_with(component)))
            .cloned()
            .collect())
    }
}

fn collect_assets(
    value: &toml::Value,
    indexed: &[PathBuf],
    components: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    match value {
        toml::Value::String(value) if value.starts_with("assets/") => {
            let path = Path::new(value);
            ensure!(
                path.components().all(|part| matches!(part, Component::Normal(_))),
                "genotype fixture asset escapes repository: {value}"
            );
            ensure!(
                indexed.iter().any(|entry| entry.starts_with(path)),
                "genotype fixture declared asset is not indexed: {value}"
            );
            let family = path.components().take(3).collect::<PathBuf>();
            ensure!(
                family.components().count() == 3,
                "genotype fixture asset lacks an owned family: {value}"
            );
            // Retain the declared provider family, including adjacent indexes and metadata.
            components.insert(family);
        }
        toml::Value::Array(values) => {
            for value in values {
                collect_assets(value, indexed, components)?;
            }
        }
        toml::Value::Table(values) => {
            for value in values.values() {
                collect_assets(value, indexed, components)?;
            }
        }
        _ => {}
    }
    Ok(())
}
