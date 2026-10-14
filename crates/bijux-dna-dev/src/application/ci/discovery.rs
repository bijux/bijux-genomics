//! Complete binary discovery partitions and admission of their native inventories.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

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
    pub(crate) binaries: BTreeSet<String>,
}

impl BinaryDiscoveryGroup {
    pub(crate) fn worker_demand(
        &self,
        case_threads: usize,
        native_worker_budget: usize,
    ) -> Result<usize> {
        let demand = self
            .binaries
            .len()
            .checked_mul(2)
            .context("native listing worker demand overflow")?
            .max(case_threads);
        ensure!(
            !self.binaries.is_empty() && demand <= native_worker_budget,
            "native listing/case demand exceeds allocation"
        );
        Ok(demand)
    }
}

impl BinaryDiscoveryPlan {
    pub(crate) fn new(bytes: &[u8], native_worker_budget: usize) -> Result<Self> {
        Self::selected(bytes, None, native_worker_budget)
    }

    /// Select whole binaries, keeping ignored and ordinary discovery in the same identity.
    pub(crate) fn selected(
        bytes: &[u8],
        selected: Option<&BTreeSet<String>>,
        native_worker_budget: usize,
    ) -> Result<Self> {
        ensure!(
            native_worker_budget >= 2,
            "pinned native discovery requires two worker slots per binary"
        );
        let mut metadata: Value =
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
        if let Some(selected) = selected {
            ensure!(
                !selected.is_empty() && selected.iter().all(|id| binaries.contains_key(id)),
                "selected native binary universe is empty or absent from producer"
            );
            metadata["rust-binaries"]
                .as_object_mut()
                .context("compiled binary map is absent")?
                .retain(|id, _| selected.contains(id));
        }
        let ids = metadata["rust-binaries"]
            .as_object()
            .context("compiled binary map is absent")?
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let groups = ids
            .chunks(native_worker_budget / 2)
            .map(|ids| {
                let binaries = ids.iter().cloned().collect::<BTreeSet<_>>();
                let digest = bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&binaries)?);
                let mut selected = metadata.clone();
                selected["rust-binaries"]
                    .as_object_mut()
                    .context("compiled binary map is absent")?
                    .retain(|id, _| binaries.contains(id));
                Ok(BinaryDiscoveryGroup {
                    label: format!("native-binaries-{digest}"),
                    metadata: serde_json::to_vec(&selected)?,
                    binaries,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { metadata, groups })
    }

    /// Expected observed paths follow the explicit metadata-only target remap. Input subset
    /// bytes remain producer-bound; all other metadata and executable identities remain exact.
    pub(crate) fn admit_target_remapping(mut self, target: &Path) -> Result<Self> {
        let producer = Path::new(
            self.metadata["rust-build-meta"]["target-directory"]
                .as_str()
                .context("producer target root is absent")?,
        )
        .to_owned();
        for root in [&producer, target] {
            ensure!(
                root.is_absolute()
                    && root
                        .components()
                        .all(|part| !matches!(part, Component::ParentDir | Component::CurDir)),
                "native discovery target is not normalized absolute authority"
            );
        }
        for binary in self.metadata["rust-binaries"]
            .as_object_mut()
            .context("compiled binary map absent")?
            .values_mut()
        {
            let path =
                Path::new(binary["binary-path"].as_str().context("producer binary path absent")?);
            let relative =
                path.strip_prefix(&producer).context("producer binary escapes admitted target")?;
            ensure!(
                relative.components().all(|part| matches!(part, Component::Normal(_))),
                "producer binary path contains traversal"
            );
            binary["binary-path"] =
                json!(target.join(relative).to_str().context("discovery path encoding")?);
        }
        self.metadata["rust-build-meta"]["target-directory"] =
            json!(target.to_str().context("discovery target encoding")?);
        Ok(self)
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
