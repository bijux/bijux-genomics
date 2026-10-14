//! Local compilation transport. The caller holds the compilation cache writer lease.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{ensure, Result};
use serde_json::{json, Value};

use crate::application::ci::{admit_compiled_executable_paths, NextestReuseInputs};
use crate::runtime::process::{ParallelProcessOutput, ParallelProcessSpec, ProcessRunner};
use crate::runtime::workspace::Workspace;

mod executable_seal;
mod native_inventory;
pub(crate) use executable_seal::ExecutableSeal;
pub(crate) use native_inventory::{
    discover_native_inventory, native_child_records, publish_native_metadata, MetadataInput,
};

pub(crate) struct LocalCompiledProducts {
    pub(crate) reuse: NextestReuseInputs,
    pub(crate) receipt: Value,
    seal: ExecutableSeal,
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
        let mut seal = ExecutableSeal::capture(&paths)?;
        receipt["fingerprint_scope"] = json!("test_and_workspace_executables_and_metadata");
        receipt["file_sha256"] = json!(seal.digest_manifest());
        receipt["input_manifest"] =
            seal.publish_manifest(&reports.join("compiled-inputs.manifest.json"))?;
        bijux_dna_infra::write_bytes(&receipt_path, &serde_json::to_vec_pretty(&receipt)?)?;
        Ok(Self {
            reuse: NextestReuseInputs::new(cargo, binaries, target.to_owned())?,
            receipt,
            seal,
        })
    }

    pub(crate) fn discover_inventory(
        &self,
        workspace: &Workspace,
        source: &Path,
        envs: &[(String, String)],
        reports: &Path,
        native_worker_budget: usize,
    ) -> Result<(Vec<u8>, std::time::Duration)> {
        discover_native_inventory(
            workspace,
            source,
            &self.reuse,
            envs,
            reports,
            native_worker_budget,
        )
    }

    pub(crate) fn verify_unchanged(&mut self) -> bool {
        let observed = self.seal.verify();
        self.receipt["file_sha256_after_execution"] = json!(observed.file_sha256);
        self.receipt["input_verification"] = observed.receipt;
        self.receipt["checksum_stable"] = json!(observed.unchanged);
        observed.unchanged
    }
}

pub(crate) fn inventory_arguments(
    reuse: &NextestReuseInputs,
    source: &Path,
) -> Result<Vec<String>> {
    let mut args = [
        "nextest",
        "list",
        "--config-file",
        "configs/rust/nextest.toml",
        "--profile",
        "full",
        "--message-format",
        "json",
        "--run-ignored",
        "all",
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
    Ok(ExecutableSeal::capture(paths)?.digest_manifest())
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
