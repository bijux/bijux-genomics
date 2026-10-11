//! Local compilation transport. The caller holds the compilation cache writer lease.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};

use crate::application::ci::{
    admit_compiled_executable_paths, BinaryDiscoveryPlan, NextestReuseInputs,
};
use crate::runtime::process::{ParallelProcessOutput, ParallelProcessSpec, ProcessRunner};
use crate::runtime::workspace::Workspace;

pub(crate) struct LocalCompiledProducts {
    pub(crate) reuse: NextestReuseInputs,
    pub(crate) receipt: Value,
    fingerprints: BTreeMap<PathBuf, String>,
}

impl LocalCompiledProducts {
    pub(crate) fn build(
        workspace: &Workspace,
        source: &Path,
        target: &Path,
        envs: &[(String, String)],
        reports: &Path,
    ) -> Result<Self> {
        let specifications = [
            (
                "cargo-metadata",
                vec!["metadata", "--all-features", "--format-version", "1", "--locked"],
            ),
            (
                "compiled-binaries",
                vec![
                    "nextest",
                    "list",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--locked",
                    "--config-file",
                    "configs/rust/nextest.toml",
                    "--profile",
                    "full",
                    "--list-type",
                    "binaries-only",
                    "--message-format",
                    "json",
                ],
            ),
        ]
        .into_iter()
        .map(|(label, args)| ParallelProcessSpec {
            label: label.to_owned(),
            program: "cargo".to_owned(),
            args: args.into_iter().map(str::to_owned).collect(),
            envs: envs.to_vec(),
            current_dir: Some(source.to_owned()),
        })
        .collect();
        let outputs = ProcessRunner::new(workspace).run_parallel_logged(specifications, reports)?;
        let mut receipt = json!({
            "schema_version":"bijux.ci.local_compiled_tests.v1",
            "transport":"local_metadata",
            "source_workspace":source,
            "cargo_target_dir":target,
            "archive_created":false,
            "coverage":{"workspace":true,"all_targets":true,"all_features":true},
            "producer_commands": outputs.iter().map(producer_receipt).collect::<Vec<_>>(),
            "all_producers_passed":outputs.iter().all(|output| output.output.status.success()),
        });
        let receipt_path = reports.join("compiled-products.receipt.json");
        bijux_dna_infra::write_bytes(&receipt_path, &serde_json::to_vec_pretty(&receipt)?)?;
        ensure!(
            outputs.iter().all(|output| output.output.status.success()),
            "local compilation failed; see compiled-products.receipt.json"
        );
        let cargo = reports.join("cargo-metadata.stdout.log");
        let binaries = reports.join("compiled-binaries.stdout.log");
        let mut paths = admit_compiled_executable_paths(
            &std::fs::read(&cargo)?,
            &std::fs::read(&binaries)?,
            source,
            target,
        )?;
        paths.extend([cargo.clone(), binaries.clone()]);
        let fingerprints = fingerprint_files(&paths)?;
        receipt["fingerprint_scope"] = json!("test_and_workspace_executables_and_metadata");
        receipt["file_sha256"] = json!(fingerprints);
        bijux_dna_infra::write_bytes(&receipt_path, &serde_json::to_vec_pretty(&receipt)?)?;
        Ok(Self {
            reuse: NextestReuseInputs::new(cargo, binaries, target.to_owned())?,
            receipt,
            fingerprints,
        })
    }

    pub(crate) fn inventory_arguments(&self, source: &Path) -> Result<Vec<String>> {
        inventory_arguments(&self.reuse, source)
    }

    pub(crate) fn discover_inventory(
        &self,
        workspace: &Workspace,
        source: &Path,
        envs: &[(String, String)],
        reports: &Path,
        concurrency: usize,
    ) -> Result<(Vec<u8>, std::time::Duration)> {
        let started = std::time::Instant::now();
        let metadata = std::fs::read(reports.join("compiled-binaries.stdout.log"))?;
        let plan = BinaryDiscoveryPlan::new(&metadata, concurrency)?;
        let mut specs = Vec::with_capacity(plan.groups.len());
        let mut metadata_inputs = BTreeMap::new();
        for group in &plan.groups {
            let path = reports.join(format!("{}.binaries.json", group.label));
            bijux_dna_infra::write_bytes(&path, &group.metadata)?;
            metadata_inputs
                .insert(path.clone(), bijux_dna_infra::hash_bytes_sha256(&group.metadata));
            let reuse = self.reuse.with_binaries_metadata(path)?;
            let args = inventory_arguments(&reuse, source)?;
            specs.push(ParallelProcessSpec {
                label: group.label.clone(),
                program: "cargo".to_owned(),
                args,
                envs: envs.to_vec(),
                current_dir: Some(source.to_path_buf()),
            });
        }
        let outputs = ProcessRunner::new(workspace).run_parallel_logged(specs, reports)?;
        let observations = outputs.iter().map(producer_receipt).collect::<Vec<_>>();
        let metadata_unchanged =
            fingerprint_files(&metadata_inputs.keys().cloned().collect())? == metadata_inputs;
        bijux_dna_infra::write_bytes(
            reports.join("test-inventory-groups.receipt.json"),
            &serde_json::to_vec_pretty(&json!({
                "schema_version":"bijux.ci.parallel_test_discovery.v1",
                "groups": observations,
                "binary_metadata_sha256":metadata_inputs,
                "binary_metadata_unchanged":metadata_unchanged,
                "binary_selection":"disjoint_metadata",
                "duration_milliseconds":started.elapsed().as_millis(),
                "all_processes_passed":outputs.iter().all(|output| output.output.status.success()),
            }))?,
        )?;
        ensure!(
            outputs.iter().all(|output| output.output.status.success()),
            "test inventory group failed; see test-inventory-group logs"
        );
        ensure!(metadata_unchanged, "discovery binary metadata changed during execution");
        let merged = plan.merge(
            outputs.iter().map(|output| (output.label.as_str(), output.output.stdout.as_slice())),
        )?;
        bijux_dna_infra::write_bytes(reports.join("test-inventory.stdout.log"), &merged)?;
        bijux_dna_infra::write_bytes(reports.join("test-inventory.stderr.log"), b"")?;
        Ok((merged, started.elapsed()))
    }

    pub(crate) fn verify_unchanged(&mut self) -> Result<bool> {
        let paths = self.fingerprints.keys().cloned().collect();
        let observed = fingerprint_files(&paths)?;
        let stable = observed == self.fingerprints;
        self.receipt["file_sha256_after_execution"] = json!(observed);
        self.receipt["checksum_stable"] = json!(stable);
        Ok(stable)
    }
}

fn inventory_arguments(reuse: &NextestReuseInputs, source: &Path) -> Result<Vec<String>> {
    let mut args = [
        "nextest",
        "list",
        "--config-file",
        "configs/rust/nextest.toml",
        "--profile",
        "full",
        "--message-format",
        "json",
    ]
    .map(str::to_owned)
    .to_vec();
    args.extend(reuse.arguments(source)?);
    Ok(args)
}

fn producer_receipt(output: &ParallelProcessOutput) -> Value {
    json!({"label":output.label,"exit_code":output.output.status.code(),"duration_milliseconds":output.duration.as_millis()})
}

fn fingerprint_files(paths: &BTreeSet<PathBuf>) -> Result<BTreeMap<PathBuf, String>> {
    paths
        .iter()
        .map(|path| {
            let metadata = path
                .symlink_metadata()
                .with_context(|| format!("inspect compiled product {}", path.display()))?;
            ensure!(
                metadata.is_file() && !metadata.file_type().is_symlink(),
                "compiled product must be an ordinary file: {}",
                path.display()
            );
            ensure!(
                path.canonicalize()? == *path,
                "compiled product traverses a symbolic path alias: {}",
                path.display()
            );
            Ok((path.clone(), bijux_dna_infra::hash_file_sha256(path)?))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_compiled_product_fingerprints_detect_mutation_and_missing_files() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().canonicalize()?.join("binary");
        bijux_dna_infra::write_bytes(&path, b"compiled executable")?;
        let paths = [path.clone()].into_iter().collect();
        let original = fingerprint_files(&paths)?;
        assert_eq!(fingerprint_files(&paths)?, original);
        bijux_dna_infra::write_bytes(&path, b"modified executable")?;
        assert_ne!(fingerprint_files(&paths)?, original);
        std::fs::remove_file(&path)?;
        assert!(fingerprint_files(&paths).is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn local_compiled_product_fingerprints_reject_redirected_files() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let root = directory.path().canonicalize()?;
        bijux_dna_infra::write_bytes(root.join("binary"), b"compiled executable")?;
        std::os::unix::fs::symlink(root.join("binary"), root.join("redirected"))?;
        assert!(fingerprint_files(&[root.join("redirected")].into_iter().collect()).is_err());
        std::os::unix::fs::symlink(&root, root.join("alias"))?;
        assert!(fingerprint_files(&[root.join("alias/binary")].into_iter().collect()).is_err());
        Ok(())
    }
}
