#![cfg(test)]

//! Distributed transport admission exercised through the native inventory and event owners.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{json, Value};

use super::protocol::{
    admit_producer, aggregate_records, extracted_inputs, imported_seal, matrix, remap_inventory,
    validate_matrix, validate_relative, verify_compatibility, Matrix, Producer, SCHEMA,
};
use crate::application::ci::{ExpectedTestInventory, ShardDiscoveryPlan};

type Records = Vec<(String, Value, Option<Vec<u8>>)>;

fn inventory_value() -> Value {
    json!({"test-count":4,"rust-suites":{"owner::contracts":{
        "package-name":"owner","binary-id":"owner::contracts","binary-name":"contracts",
        "kind":"test","status":"listed","cwd":"/producer/source/crates/owner",
        "binary-path":"/producer/target/debug/contracts",
        "testcases":{
            "first":{"ignored":false,"filter-match":{"status":"matches"}},
            "second":{"ignored":false,"filter-match":{"status":"matches"}},
            "third":{"ignored":false,"filter-match":{"status":"matches"}},
            "ignored":{"ignored":true,"filter-match":{"status":"matches"}}
        }
    }},"rust-build-meta":{"target-directory":"/producer/target"}})
}

fn inventory() -> Result<ExpectedTestInventory> {
    ExpectedTestInventory::parse(&serde_json::to_vec(&inventory_value())?)
}

fn compatibility() -> Value {
    json!({"os":"linux","target":"x86_64-unknown-linux-gnu","rust":"1.95.0",
        "nextest":"0.9.100","profile":"ci","features":"workspace-all",
        "lock":"lock-digest","config":"config-digest","bcftools":"bcftools 1.20"})
}

fn producer_fixture() -> Result<Producer> {
    Ok(Producer {
        schema_version: SCHEMA.to_owned(),
        source_commit: "source-commit".to_owned(),
        source: json!({"expected":{"sha":"source-commit","tree":"source-tree"}}),
        compatibility: compatibility(),
        archive_sha256: "archive-digest".to_owned(),
        inventory_sha256: "inventory-digest".to_owned(),
        matrix_sha256: "matrix-digest".to_owned(),
        runner_sha256: "runner-digest".to_owned(),
        source_root: PathBuf::from("/producer/source"),
        target_root: PathBuf::from("/producer/target"),
        inputs: BTreeMap::new(),
        matrix: matrix(8)?,
    })
}

fn events(names: impl IntoIterator<Item = String>, passed: bool) -> Vec<u8> {
    let names = names.into_iter().collect::<Vec<_>>();
    let identity = json!({"crate":"owner","test_binary":"contracts","kind":"test"});
    let mut lines = vec![json!({"type":"suite","event":"started",
        "test_count":names.len(),"nextest":identity})];
    for name in &names {
        let name = format!("owner::contracts${name}");
        lines.push(json!({"type":"test","event":"started","name":name}));
        lines.push(json!({"type":"test","event":if passed {"ok"} else {"failed"},"name":name}));
    }
    lines.push(json!({"type":"suite","event":if passed {"ok"} else {"failed"},
        "passed":if passed {names.len()} else {0},"failed":if passed {0} else {names.len()},
        "ignored":0,"measured":0,"nextest":identity}));
    lines.into_iter().map(|line| line.to_string()).collect::<Vec<_>>().join("\n").into_bytes()
}

fn selected_names(partition: &str) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for name in ["first", "second", "third", "ignored"] {
        let mut value = inventory_value();
        value["test-count"] = json!(1);
        value["rust-suites"]["owner::contracts"]["testcases"]
            .as_object_mut()
            .context("fixture cases")?
            .retain(|key, _| key == name);
        let single = ExpectedTestInventory::parse(&serde_json::to_vec(&value)?)?;
        let plan = ShardDiscoveryPlan::from_inventory(&single, partition, "0.9.100")?;
        if plan.expected_case_count == 1 {
            names.push(name.to_owned());
        }
    }
    Ok(names)
}

fn records(matrix: &Matrix, inventory: &ExpectedTestInventory) -> Result<Records> {
    matrix.include.iter().map(|partition| {
        let plan = ShardDiscoveryPlan::from_inventory(inventory, &partition.partition, "0.9.100")?;
        let bytes = (plan.expected_case_count != 0)
            .then(|| selected_names(&partition.partition).map(|names| events(names, true)))
            .transpose()?;
        let receipt = json!({"state":"complete","all_passed":true,
            "producer_sha256":"producer-digest","partition":partition.partition,"id":partition.id,
            "source_postcondition":{"unchanged":true},"cleanup":{"completed":true},
            "extracted_input_postcondition":{"checksum_stable":true},
            "transport_inputs_unchanged":true,
            "native_exit_code":bytes.as_ref().map(|_| 0),
            "expected_case_count":plan.expected_case_count,"native_execution_observed":bytes.is_some(),
            "events_sha256":bytes.as_deref().map(bijux_dna_infra::hash_bytes_sha256)});
        Ok((partition.id.clone(), receipt, bytes))
    }).collect()
}

#[test]
fn distributed_source_and_build_compatibility_are_required_before_consumption() -> Result<()> {
    let producer = producer_fixture()?;
    admit_producer(&producer, &producer.source, &compatibility(), &producer.matrix)?;
    for field in
        ["os", "target", "rust", "nextest", "profile", "features", "lock", "config", "bcftools"]
    {
        let mut changed = compatibility();
        changed[field] = json!("different");
        assert!(
            admit_producer(&producer, &producer.source, &changed, &producer.matrix).is_err(),
            "admitted changed {field}"
        );
    }
    let mut source = producer.source.clone();
    source["expected"]["tree"] = json!("other-tree");
    assert!(admit_producer(&producer, &source, &compatibility(), &producer.matrix).is_err());
    assert!(admit_producer(&producer, &producer.source, &compatibility(), &matrix(2)?).is_err());
    assert!(verify_compatibility(&json!(null), &json!(null)).is_err());
    let mut changed = producer_fixture()?;
    changed.source_commit = "other-commit".to_owned();
    assert!(admit_producer(&changed, &changed.source, &compatibility(), &changed.matrix).is_err());
    Ok(())
}

#[test]
fn distributed_inventory_remaps_only_declared_producer_roots() -> Result<()> {
    let producer = producer_fixture()?;
    let original = inventory_value();
    let bytes = remap_inventory(
        &serde_json::to_vec(&original)?,
        &producer,
        Path::new("/consumer/source"),
        Path::new("/consumer/products"),
    )?;
    let mapped: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(mapped["rust-suites"]["owner::contracts"]["cwd"], "/consumer/source/crates/owner");
    assert_eq!(
        mapped["rust-suites"]["owner::contracts"]["binary-path"],
        "/consumer/products/debug/contracts"
    );
    assert_eq!(
        mapped["rust-suites"]["owner::contracts"]["testcases"],
        original["rust-suites"]["owner::contracts"]["testcases"]
    );
    assert_eq!(ExpectedTestInventory::parse(&bytes)?.ignored_case_count(), 1);
    for (field, path) in [
        ("cwd", "/producer/source-neighbor/crates/owner"),
        ("binary-path", "/producer/target/../other/program"),
    ] {
        let mut escaped = original.clone();
        escaped["rust-suites"]["owner::contracts"][field] = json!(path);
        assert!(remap_inventory(
            &serde_json::to_vec(&escaped)?,
            &producer,
            Path::new("/consumer/source"),
            Path::new("/consumer/products")
        )
        .is_err());
    }
    for path in ["", "/absolute", "../outside", "native/../../outside"] {
        assert!(validate_relative(Path::new(path)).is_err());
    }
    validate_relative(Path::new("debug/deps/owner"))?;
    assert!(matrix(0).is_err());
    assert!(matrix(1).is_err());
    assert!(matrix(257).is_err());
    Ok(())
}

#[test]
fn distributed_matrix_cannot_omit_repeat_or_relabel_hash_partitions() -> Result<()> {
    let valid = matrix(20)?;
    validate_matrix(&valid)?;
    let mut omitted = valid.clone();
    omitted.include.pop();
    assert!(validate_matrix(&omitted).is_err());
    let mut repeated = valid.clone();
    repeated.include[1] = repeated.include[0].clone();
    assert!(validate_matrix(&repeated).is_err());
    let mut wrong_denominator = valid.clone();
    wrong_denominator.include[0].partition = "hash:1/5".to_owned();
    assert!(validate_matrix(&wrong_denominator).is_err());
    let mut relabeled = valid;
    relabeled.include[0].id = "other-id".to_owned();
    assert!(validate_matrix(&relabeled).is_err());
    Ok(())
}

#[test]
fn distributed_union_includes_ignored_cases_and_explicit_empty_partitions() -> Result<()> {
    let matrix = matrix(8)?;
    let inventory = inventory()?;
    let records = records(&matrix, &inventory)?;
    assert!(records.iter().any(|(_, _, bytes)| bytes.is_none()));
    let result = aggregate_records("producer-digest", &matrix, &inventory, &records)?;
    assert_eq!(result["complete"], true);
    assert_eq!(result["expected_count"], 4);
    assert_eq!(result["observed_count"], 4);
    assert_eq!(inventory.ignored_case_count(), 1);
    let mut reordered = records.clone();
    reordered.reverse();
    assert_eq!(aggregate_records("producer-digest", &matrix, &inventory, &reordered)?, result);
    Ok(())
}

#[test]
fn distributed_union_refuses_missing_duplicate_foreign_and_unhealthy_shards() -> Result<()> {
    let matrix = matrix(8)?;
    let inventory = inventory()?;
    let valid = records(&matrix, &inventory)?;
    assert!(aggregate_records("producer-digest", &matrix, &inventory, &valid[1..]).is_err());
    let mut duplicate = valid.clone();
    duplicate[1] = duplicate[0].clone();
    assert!(aggregate_records("producer-digest", &matrix, &inventory, &duplicate).is_err());
    let mut foreign = valid.clone();
    foreign[0].0 = "foreign-shard".to_owned();
    assert!(aggregate_records("producer-digest", &matrix, &inventory, &foreign).is_err());
    for (pointer, replacement) in [
        ("/state", json!("cancelled")),
        ("/state", json!("failed")),
        ("/all_passed", json!(false)),
        ("/producer_sha256", json!("stale-producer")),
        ("/partition", json!("hash:1/2")),
        ("/id", json!("foreign-shard")),
        ("/source_postcondition/unchanged", json!(false)),
        ("/extracted_input_postcondition/checksum_stable", json!(false)),
        ("/transport_inputs_unchanged", json!(false)),
        ("/cleanup/completed", json!(false)),
        ("/expected_case_count", json!(999)),
    ] {
        let mut changed = valid.clone();
        *changed[0].1.pointer_mut(pointer).context("receipt fixture pointer")? = replacement;
        assert!(
            aggregate_records("producer-digest", &matrix, &inventory, &changed).is_err(),
            "admitted {pointer}"
        );
    }
    Ok(())
}

#[test]
fn distributed_union_cannot_replace_native_events_with_green_counters() -> Result<()> {
    let matrix = matrix(8)?;
    let inventory = inventory()?;
    let valid = records(&matrix, &inventory)?;
    let selected =
        valid.iter().position(|(_, _, bytes)| bytes.is_some()).context("nonempty partition")?;
    let mut missing = valid.clone();
    missing[selected].2 = None;
    assert!(aggregate_records("producer-digest", &matrix, &inventory, &missing).is_err());
    for bytes in [Vec::new(), b"not-json".to_vec(), events(["foreign".to_owned()], true)] {
        let mut changed = valid.clone();
        changed[selected].1["events_sha256"] = json!(bijux_dna_infra::hash_bytes_sha256(&bytes));
        changed[selected].2 = Some(bytes);
        assert!(aggregate_records("producer-digest", &matrix, &inventory, &changed).is_err());
    }
    let failed = events(selected_names(&matrix.include[selected].partition)?, false);
    let mut changed = valid.clone();
    changed[selected].1["events_sha256"] = json!(bijux_dna_infra::hash_bytes_sha256(&failed));
    changed[selected].2 = Some(failed);
    assert!(aggregate_records("producer-digest", &matrix, &inventory, &changed).is_err());
    let mut damaged = valid.clone();
    damaged[selected].2.as_mut().context("event bytes")?.push(b' ');
    assert!(aggregate_records("producer-digest", &matrix, &inventory, &damaged).is_err());
    let empty =
        valid.iter().position(|(_, _, bytes)| bytes.is_none()).context("empty partition")?;
    for (index, exit_code) in [(selected, json!(1)), (empty, json!(0))] {
        let mut contradictory = valid.clone();
        contradictory[index].1["native_exit_code"] = exit_code;
        assert!(aggregate_records("producer-digest", &matrix, &inventory, &contradictory).is_err());
    }
    for index in [selected, empty] {
        let mut missing_exit = valid.clone();
        missing_exit[index].1.as_object_mut().context("receipt object")?.remove("native_exit_code");
        assert!(aggregate_records("producer-digest", &matrix, &inventory, &missing_exit).is_err());
    }
    let mut invented = valid;
    invented[empty].2 = Some(Vec::new());
    assert!(aggregate_records("producer-digest", &matrix, &inventory, &invented).is_err());
    Ok(())
}

#[test]
fn distributed_union_refuses_valid_events_for_the_wrong_partition() -> Result<()> {
    let matrix = matrix(8)?;
    let inventory = inventory()?;
    let mut records = records(&matrix, &inventory)?;
    let nonempty = records
        .iter()
        .enumerate()
        .filter_map(|(index, (_, _, bytes))| bytes.as_ref().map(|_| index))
        .collect::<Vec<_>>();
    assert!(nonempty.len() >= 2, "fixture must exercise separate real hash partitions");
    let (first, second) = (nonempty[0], nonempty[1]);
    let first_bytes = records[first].2.take();
    records[first].2 = records[second].2.take();
    records[second].2 = first_bytes;
    for index in [first, second] {
        records[index].1["events_sha256"] = json!(bijux_dna_infra::hash_bytes_sha256(
            records[index].2.as_deref().context("swapped events")?
        ));
    }
    assert!(aggregate_records("producer-digest", &matrix, &inventory, &records).is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn distributed_import_seal_refuses_modified_extra_writable_and_redirected_files() -> Result<()> {
    use std::os::unix::fs::{symlink, PermissionsExt};

    let artifact_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../artifacts/tests/distributed-admission");
    std::fs::create_dir_all(&artifact_root)?;
    let temporary = tempfile::tempdir_in(&artifact_root)?;
    let root = temporary.path().canonicalize()?;
    let file = root.join("native");
    std::fs::write(&file, b"sealed native input")?;
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o555))?;
    let inputs = extracted_inputs(&root)?;
    imported_seal(&root, &inputs)?;
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755))?;
    assert!(imported_seal(&root, &inputs).is_err());
    let writable = extracted_inputs(&root)?;
    assert!(
        imported_seal(&root, &writable).is_err(),
        "a matching manifest does not authorize writable products"
    );
    std::fs::write(&file, b"changed native input")?;
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o555))?;
    assert!(imported_seal(&root, &inputs).is_err());
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755))?;
    std::fs::write(&file, b"sealed native input")?;
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o555))?;
    std::fs::write(root.join("undeclared"), b"extra")?;
    assert!(imported_seal(&root, &inputs).is_err());
    std::fs::remove_file(root.join("undeclared"))?;
    std::fs::remove_file(&file)?;
    assert!(imported_seal(&root, &inputs).is_err());
    symlink(root.join("absent"), &file)?;
    assert!(imported_seal(&root, &inputs).is_err());
    let mut escaped = inputs;
    let input = escaped.remove(Path::new("native")).context("file fixture")?;
    escaped.insert(PathBuf::from("../outside"), input);
    assert!(imported_seal(&root, &escaped).is_err());
    Ok(())
}
