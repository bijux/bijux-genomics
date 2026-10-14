//! Native test discovery subsets with explicit process demand and original metadata authority.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{ensure, Result};
use serde::Serialize;
use serde_json::json;

use crate::application::ci::{BinaryDiscoveryPlan, NextestReuseInputs};
use crate::runtime::process::{
    DependentProcessSpec, ParallelProcessSpec, ProcessRunner, WeightedProcessSpec,
};
use crate::runtime::source_snapshot::{IsolatedSourceIdentity, TrackedSourceSnapshot};
use crate::runtime::workspace::Workspace;

use super::{fingerprint_files, inventory_arguments};

#[derive(Serialize)]
pub(crate) struct MetadataInput {
    pub(crate) path: PathBuf,
    pub(crate) sha256: String,
    pub(crate) bytes: u64,
    pub(crate) mode: u32,
}

fn metadata_mode(metadata: &std::fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    }
    #[cfg(not(unix))]
    {
        u32::from(metadata.permissions().readonly())
    }
}

impl MetadataInput {
    pub(crate) fn verify_unchanged(&self) -> Result<()> {
        ensure!(
            self.path.symlink_metadata()?.is_file()
                && self.path.canonicalize()? == self.path
                && self.path.metadata()?.permissions().readonly(),
            "native subset physical ownership or readonly state changed"
        );
        let metadata = self.path.metadata()?;
        ensure!(
            metadata.len() == self.bytes && metadata_mode(&metadata) == self.mode,
            "native subset bytes/mode identity changed"
        );
        let fingerprints = fingerprint_files(&[self.path.clone()].into_iter().collect())?;
        ensure!(fingerprints.get(&self.path) == Some(&self.sha256), "native subset bytes changed");
        Ok(())
    }
}

pub(crate) fn publish_native_metadata(path: &Path, bytes: &[u8]) -> Result<MetadataInput> {
    if path.try_exists()? {
        ensure!(
            path.symlink_metadata()?.is_file() && path.canonicalize()? == path,
            "native subset output owner is redirected"
        );
        bijux_dna_infra::remove_file(path)?;
    }
    bijux_dna_infra::write_bytes(path, bytes)?;
    let mut permissions = path.metadata()?.permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions)?;
    let seal = MetadataInput {
        path: path.to_owned(),
        sha256: bijux_dna_infra::hash_bytes_sha256(bytes),
        bytes: u64::try_from(bytes.len())?,
        mode: metadata_mode(&path.metadata()?),
    };
    seal.verify_unchanged()?;
    Ok(seal)
}

/// Read freshly initialized child states, retaining completed peers after admission/launch errors.
pub(crate) fn native_child_records<'a>(
    labels: impl IntoIterator<Item = &'a str>,
    reports: &Path,
) -> Result<Vec<serde_json::Value>> {
    labels
        .into_iter()
        .map(|label| {
            let path = reports.join(format!("{label}.receipt.json"));
            let status = if path.try_exists()? {
                ensure!(
                    path.symlink_metadata()?.is_file() && path.canonicalize()? == path,
                    "native child receipt physical owner changed"
                );
                let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
                ensure!(value["label"] == label, "native child receipt changed identity");
                value
            } else {
                json!({"label":label,"state":"not_started","terminal":false,"all_passed":false})
            };
            Ok(json!({"label":label,"receipt":path,"status":status}))
        })
        .collect()
}

pub(crate) fn discover_native_inventory(
    workspace: &Workspace,
    source: &Path,
    reuse: &NextestReuseInputs,
    envs: &[(String, String)],
    reports: &Path,
    native_worker_budget: usize,
) -> Result<(Vec<u8>, std::time::Duration)> {
    let started = Instant::now();
    let source_workspace = Workspace { root: source.to_owned() };
    let expected_source = IsolatedSourceIdentity::capture(workspace)?;
    expected_source.require_snapshot(&TrackedSourceSnapshot::capture(workspace)?)?;
    expected_source.verify(&source_workspace)?;
    let source_before = TrackedSourceSnapshot::capture(&source_workspace)?;
    let metadata = std::fs::read(reuse.binaries_metadata())?;
    let producer_sha256 = bijux_dna_infra::hash_bytes_sha256(&metadata);
    let plan = BinaryDiscoveryPlan::new(&metadata, native_worker_budget)?;
    let mut specs = Vec::new();
    let mut inputs = BTreeMap::new();
    for group in &plan.groups {
        let input = publish_native_metadata(
            &reports.join(format!("{}.binaries.json", group.label)),
            &group.metadata,
        )?;
        let subset = reuse.with_binaries_metadata(input.path.clone())?;
        specs.push(WeightedProcessSpec {
            process: DependentProcessSpec {
                process: ParallelProcessSpec {
                    label: group.label.clone(),
                    program: "cargo".into(),
                    args: inventory_arguments(&subset, source)?,
                    envs: envs.to_vec(),
                    current_dir: Some(source.to_owned()),
                },
                dependencies: vec![],
            },
            resource_units: group.worker_demand(0, native_worker_budget)?,
            terminal_dependencies: vec![],
        });
        inputs.insert(group.label.clone(), input);
    }
    let path = reports.join("test-inventory-groups.receipt.json");
    let mut receipt = json!({"schema_version":"bijux.ci.bounded_native_discovery.v1", "all_processes_passed":false,
        "source_condition":expected_source.descriptor(),
        "source_commit":source_before.commit(),"source_patch_sha256":source_before.patch_sha256(),
        "producer_binary_metadata_sha256":producer_sha256,"native_worker_budget":native_worker_budget,
        "worker_scope":"ordinary/ignored native harness listing; parent runtimes and nested tools have no hard physical CPU/process cap",
        "metadata_inputs":inputs,
        "expected_groups":plan.groups.iter().map(|group|json!({"label":group.label,"binaries":group.binaries,"native_listing_demand":group.binaries.len()*2})).collect::<Vec<_>>()});
    bijux_dna_infra::write_bytes(&path, &serde_json::to_vec_pretty(&receipt)?)?;
    let outputs = ProcessRunner::new(workspace).run_prepared_weighted_parallel_logged(
        specs,
        native_worker_budget / 2,
        native_worker_budget,
        reports,
        |spec| Ok(spec.clone()),
    );
    let source_postcondition = expected_source.postcondition(&source_workspace);
    let verified = (|| -> Result<()> {
        for input in inputs.values() {
            input.verify_unchanged()?;
        }
        ensure!(
            bijux_dna_infra::hash_file_sha256(reuse.binaries_metadata())? == producer_sha256,
            "producer binary metadata changed during native discovery"
        );
        ensure!(
            source_postcondition["unchanged"] == true,
            "native discovery source changed: {}",
            source_postcondition["error"]
        );
        Ok(())
    })();
    receipt["source_postcondition"] = source_postcondition;
    receipt["terminal_groups"] =
        json!(native_child_records(plan.groups.iter().map(|group| group.label.as_str()), reports)?);
    receipt["physical_inputs_unchanged"] = json!(verified.is_ok());
    receipt["duration_milliseconds"] = json!(started.elapsed().as_millis());
    receipt["execution_error"] = json!(outputs.as_ref().err().map(|error| format!("{error:#}")));
    receipt["input_error"] = json!(verified.as_ref().err().map(|error| format!("{error:#}")));
    if let Ok(outputs) = &outputs {
        receipt["groups"] = json!(outputs.iter().map(|output|json!({"label":output.label,"exit_code":output.output.status.code(),"duration_milliseconds":output.duration.as_millis()})).collect::<Vec<_>>());
        receipt["all_processes_passed"] =
            json!(verified.is_ok() && outputs.iter().all(|output| output.output.status.success()));
    }
    bijux_dna_infra::write_bytes(&path, &serde_json::to_vec_pretty(&receipt)?)?;
    verified?;
    let outputs = outputs?;
    ensure!(
        outputs.iter().all(|output| output.output.status.success()),
        "bounded native inventory group failed; see test-inventory-groups.receipt.json"
    );
    let plan = plan.admit_target_remapping(reuse.target_root())?;
    let merged = plan.merge(
        outputs.iter().map(|output| (output.label.as_str(), output.output.stdout.as_slice())),
    )?;
    bijux_dna_infra::write_bytes(reports.join("test-inventory.stdout.log"), &merged)?;
    bijux_dna_infra::write_bytes(reports.join("test-inventory.stderr.log"), b"")?;
    Ok((merged, started.elapsed()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_subset_bytes_modes_and_redirected_identity_cannot_be_resealed() -> Result<()> {
        let owner = tempfile::tempdir()?;
        let path = owner.path().canonicalize()?.join("native-metadata.json");
        let seal = publish_native_metadata(&path, b"original admitted native subset")?;
        seal.verify_unchanged()?;
        std::fs::remove_file(&path)?;
        std::fs::write(&path, b"changed producer bytes")?;
        assert!(seal.verify_unchanged().is_err());
        std::fs::remove_file(&path)?;
        std::fs::write(&path, b"original admitted native subset")?;
        assert!(seal.verify_unchanged().is_err(), "readonly mode belongs to physical admission");
        #[cfg(unix)]
        {
            std::fs::remove_file(&path)?;
            let alternate = path.with_file_name("alternate.json");
            std::fs::write(&alternate, b"original admitted native subset")?;
            let mut permissions = alternate.metadata()?.permissions();
            permissions.set_readonly(true);
            std::fs::set_permissions(&alternate, permissions)?;
            std::os::unix::fs::symlink(&alternate, &path)?;
            assert!(seal.verify_unchanged().is_err());
        }
        Ok(())
    }
}
