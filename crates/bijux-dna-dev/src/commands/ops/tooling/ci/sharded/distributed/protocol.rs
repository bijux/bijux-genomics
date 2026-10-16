//! Source-bound archive compatibility and exact distributed partition evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::application::ci::{CoverageAttestation, ExpectedTestInventory, ShardDiscoveryPlan};
use crate::runtime::compiled_products::ExecutableSeal;

pub(super) const SCHEMA: &str = "bijux.ci.distributed_producer.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Partition {
    pub partition: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Matrix {
    pub include: Vec<Partition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FileInput {
    pub sha256: String,
    pub bytes: u64,
    pub mode: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Producer {
    pub schema_version: String,
    pub source_commit: String,
    pub source: Value,
    pub compatibility: Value,
    pub archive_sha256: String,
    pub inventory_sha256: String,
    pub matrix_sha256: String,
    pub runner_sha256: String,
    pub source_root: PathBuf,
    pub target_root: PathBuf,
    pub inputs: BTreeMap<PathBuf, FileInput>,
    pub matrix: Matrix,
}

pub(super) fn matrix(count: usize) -> Result<Matrix> {
    ensure!((2..=256).contains(&count), "distributed partition count must be 2..256");
    Ok(Matrix {
        include: (1..=count)
            .map(|index| Partition {
                partition: format!("hash:{index}/{count}"),
                id: format!("hash-{index}-of-{count}"),
            })
            .collect(),
    })
}

pub(super) fn validate_matrix(value: &Matrix) -> Result<()> {
    ensure!(
        *value == matrix(value.include.len())?,
        "distributed matrix is not a complete canonical hash roster"
    );
    Ok(())
}

pub(super) fn validate_relative(path: &Path) -> Result<()> {
    ensure!(
        !path.as_os_str().is_empty()
            && path.components().all(|part| matches!(part, Component::Normal(_))),
        "distributed input path must be a confined relative path"
    );
    Ok(())
}

pub(super) fn fingerprint(path: &Path) -> Result<FileInput> {
    let metadata = path.symlink_metadata()?;
    ensure!(metadata.is_file() && path.canonicalize()? == path, "artifact input is redirected");
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o777
    };
    #[cfg(not(unix))]
    let mode = u32::from(metadata.permissions().readonly());
    Ok(FileInput { sha256: bijux_dna_infra::hash_file_sha256(path)?, bytes: metadata.len(), mode })
}

pub(super) fn extracted_inputs(root: &Path) -> Result<BTreeMap<PathBuf, FileInput>> {
    ensure!(root.is_absolute() && root.canonicalize()? == root, "archive root is redirected");
    let mut inputs = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        ensure!(!entry.file_type().is_symlink(), "archive input is a symbolic link");
        if entry.file_type().is_file() {
            let relative = entry.path().strip_prefix(root)?.to_path_buf();
            validate_relative(&relative)?;
            inputs.insert(relative, fingerprint(entry.path())?);
        }
    }
    ensure!(!inputs.is_empty(), "archive input manifest is empty");
    Ok(inputs)
}

pub(super) fn imported_seal(
    root: &Path,
    expected: &BTreeMap<PathBuf, FileInput>,
) -> Result<ExecutableSeal> {
    for path in expected.keys() {
        validate_relative(path)?;
    }
    ensure!(
        extracted_inputs(root)? == *expected,
        "extracted archive differs from producer manifest"
    );
    let paths = expected.keys().map(|path| root.join(path)).collect();
    let seal = ExecutableSeal::capture(&paths)?;
    seal.require_readonly()?;
    Ok(seal)
}

pub(super) fn verify_compatibility(expected: &Value, actual: &Value) -> Result<()> {
    ensure!(
        expected.is_object() && expected == actual,
        "distributed archive compatibility changed"
    );
    Ok(())
}

pub(super) fn admit_producer(
    producer: &Producer,
    source: &Value,
    compatibility: &Value,
    expected_matrix: &Matrix,
) -> Result<()> {
    validate_matrix(&producer.matrix)?;
    ensure!(producer.schema_version == SCHEMA, "unknown distributed producer schema");
    ensure!(producer.source == *source, "distributed source identity changed");
    ensure!(
        producer.source["expected"]["sha"] == producer.source_commit,
        "producer commit disagrees with its source condition"
    );
    ensure!(producer.matrix == *expected_matrix, "distributed partition roster changed");
    ensure!(
        producer.source_root.is_absolute() && producer.target_root.is_absolute(),
        "producer path roots are not absolute"
    );
    verify_compatibility(&producer.compatibility, compatibility)
}

/// Rebase only producer-owned paths needed by the retained crate-proof interface.
/// Nextest itself remaps the original metadata through `NextestReuseInputs`.
pub(super) fn remap_inventory(
    bytes: &[u8],
    producer: &Producer,
    source: &Path,
    target: &Path,
) -> Result<Vec<u8>> {
    let mut value: Value = serde_json::from_slice(bytes)?;
    for suite in
        value["rust-suites"].as_object_mut().context("inventory suites absent")?.values_mut()
    {
        for (field, original, replacement) in
            [("cwd", &producer.source_root, source), ("binary-path", &producer.target_root, target)]
        {
            let path = Path::new(suite[field].as_str().context("inventory path absent")?);
            let relative =
                path.strip_prefix(original).context("inventory escaped producer root")?;
            if !relative.as_os_str().is_empty() {
                validate_relative(relative)?;
            }
            suite[field] = json!(replacement.join(relative));
        }
    }
    value["rust-build-meta"]["target-directory"] = json!(target);
    Ok(serde_json::to_vec_pretty(&value)?)
}

pub(super) fn aggregate_records(
    producer_sha: &str,
    matrix: &Matrix,
    inventory: &ExpectedTestInventory,
    records: &[(String, Value, Option<Vec<u8>>)],
) -> Result<Value> {
    validate_matrix(matrix)?;
    for (_, receipt, _) in records {
        ensure!(
            receipt["extracted_input_postcondition"]["checksum_stable"] == true
                && receipt["transport_inputs_unchanged"] == true,
            "distributed shard physical input postcondition failed"
        );
    }

    let expected = matrix.include.iter().map(|row| row.id.as_str()).collect::<BTreeSet<_>>();
    let actual = records.iter().map(|(id, _, _)| id.as_str()).collect::<BTreeSet<_>>();
    ensure!(
        records.len() == expected.len() && actual == expected,
        "distributed shard set is missing, duplicate or foreign"
    );
    let mut plans = Vec::new();
    for (id, receipt, bytes) in records {
        let partition = matrix.include.iter().find(|row| row.id == *id).context("unknown shard")?;
        ensure!(
            receipt["state"] == "complete"
                && receipt["all_passed"] == true
                && receipt["producer_sha256"] == producer_sha
                && receipt["partition"] == partition.partition
                && receipt["id"] == *id,
            "distributed shard is failed, cancelled, stale or incomplete: {id}"
        );
        ensure!(
            receipt["source_postcondition"]["unchanged"] == true,
            "shard source postcondition failed"
        );
        ensure!(receipt["cleanup"]["completed"] == true, "shard cleanup failed");
        let plan = ShardDiscoveryPlan::from_inventory(inventory, &partition.partition, "0.9.100")?;
        ensure!(
            receipt["expected_case_count"] == plan.expected_case_count,
            "distributed selected case count changed"
        );
        if plan.expected_case_count == 0 {
            ensure!(
                bytes.is_none()
                    && receipt["native_execution_observed"] == false
                    && receipt.get("native_exit_code").is_some_and(Value::is_null),
                "expected-empty shard invented execution"
            );
        } else {
            let bytes = bytes.as_ref().context("nonempty shard lacks events")?;
            ensure!(
                receipt["native_execution_observed"] == true
                    && receipt.get("native_exit_code").is_some_and(|code| code == 0)
                    && receipt["events_sha256"] == bijux_dna_infra::hash_bytes_sha256(bytes),
                "distributed event stream changed"
            );
        }
        plans.push(plan);
    }
    let coverage = CoverageAttestation::reconcile_terminals(
        inventory,
        records
            .iter()
            .zip(&plans)
            .map(|((id, _, bytes), plan)| (id.as_str(), bytes.as_deref(), plan)),
    );
    ensure!(
        coverage.complete && coverage.selected_shards_valid,
        "distributed native union is incomplete or failed"
    );
    Ok(serde_json::to_value(coverage)?)
}
