use std::collections::BTreeSet;

use anyhow::{ensure, Context, Result};
use serde::Serialize;
use twox_hash::XxHash64;

use super::inventory::{ExpectedTestInventory, TestIdentity};

/// Binary discovery exclusions derived from the executor's exact case partition.
#[derive(Serialize)]
pub(crate) struct ShardDiscoveryPlan {
    partition: String,
    pub(crate) filterset: String,
    pub(crate) expected_case_count: usize,
    selected_binary_count: usize,
    case_universe_sha256: String,
    #[serde(skip)]
    pub(super) expected: BTreeSet<TestIdentity>,
}

impl ShardDiscoveryPlan {
    pub(crate) fn verify_inventory(&self, inventory: &ExpectedTestInventory) -> Result<()> {
        self.require_partition(inventory, &self.partition)
    }

    pub(crate) fn require_partition(
        &self,
        inventory: &ExpectedTestInventory,
        partition: &str,
    ) -> Result<()> {
        let expected = Self::from_inventory(inventory, partition, "0.9.100")?;
        ensure!(
            self.partition == expected.partition
                && self.case_universe_sha256 == expected.case_universe_sha256
                && self.expected == expected.expected
                && self.filterset == expected.filterset
                && self.expected_case_count == expected.expected_case_count
                && self.selected_binary_count == expected.selected_binary_count,
            "partition plan differs from admitted case inventory or assignment"
        );
        Ok(())
    }

    pub(crate) fn selected_binary_ids(&self) -> BTreeSet<String> {
        self.expected.iter().map(|case| case.binary_id.clone()).collect()
    }

    /// Selected case counts retain the already-admitted partition and binary identities.
    pub(crate) fn selected_binary_case_counts(&self) -> std::collections::BTreeMap<String, usize> {
        let mut counts = std::collections::BTreeMap::new();
        for case in &self.expected {
            *counts.entry(case.binary_id.clone()).or_default() += 1;
        }
        counts
    }

    pub(crate) fn from_inventory(
        inventory: &ExpectedTestInventory,
        partition: &str,
        executor_version: &str,
    ) -> Result<Self> {
        ensure!(
            executor_version == "0.9.100",
            "binary discovery planning requires qualified Nextest 0.9.100 hash semantics"
        );
        let suffix = partition.strip_prefix("hash:").context("expected a hash partition")?;
        let (selected, total) = suffix.split_once('/').context("expected hash:M/N")?;
        let selected = selected.parse::<u64>().context("invalid selected hash partition")?;
        let total = total.parse::<u64>().context("invalid hash partition count")?;
        ensure!((1..=total).contains(&selected), "hash partition must satisfy 1 <= M <= N");
        // Nextest 0.9 hashes the test name with xxHash64 and seed zero. Binary IDs do not
        // participate. Keep the native --partition argument as the execution authority.
        let expected = inventory
            .cases
            .values()
            .map(|(case, _)| case)
            .filter(|case| XxHash64::oneshot(0, case.test_name.as_bytes()) % total == selected - 1)
            .cloned()
            .collect::<BTreeSet<_>>();
        let binaries = expected.iter().map(|case| case.binary_id.as_str()).collect::<BTreeSet<_>>();
        let filterset = if binaries.is_empty() {
            "none()".to_string()
        } else {
            binaries
                .iter()
                .map(|id| format!("binary_id(={})", escape_matcher(id)))
                .collect::<Vec<_>>()
                .join(" | ")
        };
        Ok(Self {
            partition: partition.to_owned(),
            filterset,
            expected_case_count: expected.len(),
            selected_binary_count: binaries.len(),
            case_universe_sha256: bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(
                &inventory.cases.values().map(|(case, _)| case).collect::<Vec<_>>(),
            )?),
            expected,
        })
    }
}

pub(super) fn escape_matcher(value: &str) -> String {
    value.chars().fold(String::new(), |mut escaped, character| {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '/' | ')' | ',' => {
                escaped.push('\\');
                escaped.push(character);
            }
            _ => escaped.push(character),
        }
        escaped
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_matches_captured_pinned_executor_case_partitions() -> Result<()> {
        for (name, hash, partition) in [
            ("ordinary", 5_165_241_260_839_206_075, 1),
            ("ignored", 3_808_214_212_173_381_903, 4),
            ("failure", 8_649_434_444_502_015_239, 5),
        ] {
            assert_eq!(XxHash64::oneshot(0, name.as_bytes()), hash);
            assert_eq!(hash % 5 + 1, partition);
        }
        let suite = |id: &str, names: &[(&str, bool)]| {
            serde_json::json!({
                "package-name":"owner", "binary-id":id, "binary-name":id, "kind":"test", "status":"listed",
                "testcases":names.iter().map(|(name, ignored)| ((*name).to_owned(), serde_json::json!({
                    "ignored":ignored, "filter-match":{"status":"matches"}
                }))).collect::<std::collections::BTreeMap<_, _>>()
            })
        };
        let inventory = ExpectedTestInventory::parse(&serde_json::to_vec(&serde_json::json!({
            "test-count":3, "rust-suites":{
                "first":suite("first", &[("ordinary", false)]),
                "second":suite("second", &[("ordinary", false), ("ignored", true)])
            }
        }))?)?;
        let first = ShardDiscoveryPlan::from_inventory(&inventory, "hash:1/5", "0.9.100")?;
        assert_eq!(
            first.selected_binary_case_counts(),
            [("first".to_owned(), 1), ("second".to_owned(), 1)].into()
        );
        first.verify_inventory(&inventory)?;
        let ignored = ShardDiscoveryPlan::from_inventory(&inventory, "hash:4/5", "0.9.100")?;
        assert_eq!(ignored.selected_binary_case_counts(), [("second".to_owned(), 1)].into());
        assert_eq!(
            ignored.selected_binary_case_counts().values().sum::<usize>(),
            ignored.expected_case_count
        );
        let empty = ShardDiscoveryPlan::from_inventory(&inventory, "hash:5/5", "0.9.100")?;
        assert!(empty.selected_binary_case_counts().is_empty());
        assert!(!ignored.selected_binary_case_counts().contains_key("foreign"));
        Ok(())
    }

    #[test]
    fn exact_matchers_escape_filterset_delimiters_and_control_characters() {
        assert_eq!(
            escape_matcher("owner::bin/path),\\\n\r\t"),
            "owner::bin\\/path\\)\\,\\\\\\n\\r\\t"
        );
    }
}
