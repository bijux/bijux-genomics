//! Complete binary discovery partitions and admission of their native inventories.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};

use super::inventory::ExpectedTestInventory;

pub(crate) struct BinaryDiscoveryPlan {
    metadata: Value,
    pub(crate) groups: Vec<BinaryDiscoveryGroup>,
}

pub(crate) struct BinaryDiscoveryGroup {
    pub(crate) label: String,
    pub(crate) metadata: Vec<u8>,
    binaries: BTreeSet<String>,
}

impl BinaryDiscoveryPlan {
    pub(crate) fn new(bytes: &[u8], concurrency: usize) -> Result<Self> {
        ensure!(
            (1..=5).contains(&concurrency),
            "discovery concurrency must be between one and five"
        );
        let metadata: Value =
            serde_json::from_slice(bytes).context("parse compiled binary metadata")?;
        let binaries =
            metadata["rust-binaries"].as_object().context("compiled binary map is absent")?;
        ensure!(!binaries.is_empty(), "compiled binary inventory is empty");
        ensure!(metadata["rust-build-meta"].is_object(), "compiled build metadata is absent");
        for (id, binary) in binaries {
            ensure!(
                !id.is_empty() && binary["binary-id"].as_str() == Some(id),
                "compiled binary identity disagrees with map key"
            );
        }
        let mut selections = vec![BTreeSet::new(); concurrency.min(binaries.len())];
        let count = selections.len();
        for (index, id) in binaries.keys().enumerate() {
            selections[index % count].insert(id.clone());
        }
        let groups = selections
            .into_iter()
            .enumerate()
            .map(|(index, binaries)| {
                let mut selected = metadata.clone();
                selected["rust-binaries"]
                    .as_object_mut()
                    .context("compiled binary map is absent")?
                    .retain(|id, _| binaries.contains(id));
                Ok(BinaryDiscoveryGroup {
                    label: format!("test-inventory-group-{index}"),
                    metadata: serde_json::to_vec(&selected)?,
                    binaries,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { metadata, groups })
    }

    pub(crate) fn merge<'a>(
        &self,
        observations: impl IntoIterator<Item = (&'a str, &'a [u8])>,
    ) -> Result<Vec<u8>> {
        let expected_binaries =
            self.metadata["rust-binaries"].as_object().context("compiled binary map is absent")?;
        let mut seen_groups = BTreeSet::new();
        let mut merged = BTreeMap::new();
        let mut total = 0usize;
        for (label, bytes) in observations {
            let group = self
                .groups
                .iter()
                .find(|group| group.label == label)
                .context("unexpected discovery group")?;
            ensure!(seen_groups.insert(label), "duplicate discovery group");
            let value: Value =
                serde_json::from_slice(bytes).context("parse discovered inventory")?;
            ensure!(
                value["rust-build-meta"] == self.metadata["rust-build-meta"],
                "discovery build metadata changed"
            );
            let suites =
                value["rust-suites"].as_object().context("discovered suite map is absent")?;
            ensure!(
                suites.keys().eq(group.binaries.iter()),
                "discovery changed its assigned binary universe"
            );
            let mut count = 0usize;
            for (id, suite) in suites {
                let expected = &expected_binaries[id];
                for field in [
                    "binary-id",
                    "binary-name",
                    "package-id",
                    "kind",
                    "binary-path",
                    "build-platform",
                ] {
                    ensure!(
                        !expected[field].is_null() && suite[field] == expected[field],
                        "discovered binary {id} changed {field}"
                    );
                }
                let cases =
                    suite["testcases"].as_object().context("discovered cases are absent")?;
                ensure!(suite["status"] == "listed", "selected binary {id} was not discovered");
                for case in cases.values() {
                    ensure!(
                        case["filter-match"]["status"] == "matches",
                        "discovery filtered a selected case"
                    );
                    ensure!(case["ignored"].is_boolean(), "discovered ignored status is absent");
                }
                count = count.checked_add(cases.len()).context("discovery count overflow")?;
                ensure!(
                    merged.insert(id.clone(), suite.clone()).is_none(),
                    "duplicate discovered binary"
                );
            }
            ensure!(
                value["test-count"].as_u64() == Some(u64::try_from(count)?),
                "discovery test count disagrees with cases"
            );
            total = total.checked_add(count).context("total discovery count overflow")?;
        }
        ensure!(seen_groups.len() == self.groups.len(), "missing discovery group");
        ensure!(merged.keys().eq(expected_binaries.keys()), "discovery omitted compiled binaries");
        let bytes = serde_json::to_vec(
            &json!({"rust-build-meta":self.metadata["rust-build-meta"], "test-count":total, "rust-suites":merged}),
        )?;
        ExpectedTestInventory::parse(&bytes)?;
        Ok(bytes)
    }
}

#[cfg(test)]
#[path = "../../../tests/boundaries/ci/discovery.rs"]
mod tests;
