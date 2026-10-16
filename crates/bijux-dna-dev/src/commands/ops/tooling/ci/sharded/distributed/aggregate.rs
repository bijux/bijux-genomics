//! Exact producer-bound aggregation of the complete downloaded shard roster.

use std::path::Path;

use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};

use crate::application::ci::ExpectedTestInventory;
use crate::runtime::workspace::Workspace;

use super::protocol::aggregate_records;
use super::{load_producer, Session};

pub(super) fn aggregate(
    _workspace: &Workspace,
    root: &Path,
    session: &mut Session,
    receipt: &mut Value,
) -> Result<()> {
    let producer = load_producer(root, session, false)?;
    let producer_sha = bijux_dna_infra::hash_file_sha256(&root.join("build/producer.json"))?;
    receipt["producer_sha256"] = json!(producer_sha);
    let inventory =
        ExpectedTestInventory::parse(&std::fs::read(root.join("build/inventory.json"))?)?;
    let shard_root = root.join("shards");
    let mut records = Vec::new();
    for entry in std::fs::read_dir(&shard_root).context("downloaded shards are absent")? {
        let entry = entry?;
        ensure!(
            entry.file_type()?.is_dir() && entry.path().canonicalize()? == entry.path(),
            "downloaded shard is redirected or not a directory"
        );
        let id = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::anyhow!("shard id is not UTF-8"))?;
        let path = entry.path().join("receipt.json");
        let value: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        ensure!(
            value["source_condition"] == producer.source,
            "shard source condition differs from producer"
        );
        ensure!(value["transport_inputs_unchanged"] == true, "shard transported inputs changed");
        let events = entry.path().join("events.jsonl");
        let bytes = events.try_exists()?.then(|| std::fs::read(&events)).transpose()?;
        records.push((id, value, bytes));
    }
    let coverage = aggregate_records(&producer_sha, &producer.matrix, &inventory, &records)?;
    receipt["coverage"] = coverage;
    receipt["source_commit"] = json!(producer.source_commit);
    receipt["archive_sha256"] = json!(producer.archive_sha256);
    receipt["partitions"] = json!(producer.matrix.include);
    receipt["all_passed"] = json!(true);
    Ok(())
}
