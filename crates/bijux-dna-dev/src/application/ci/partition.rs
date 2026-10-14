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
    #[serde(skip)]
    pub(super) expected: BTreeSet<TestIdentity>,
}

impl ShardDiscoveryPlan {
    pub(crate) fn selected_binary_ids(&self) -> BTreeSet<String> {
        self.expected.iter().map(|case| case.binary_id.clone()).collect()
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
    fn hash_matches_captured_pinned_executor_case_partitions() {
        for (name, hash, partition) in [
            ("ordinary", 5_165_241_260_839_206_075, 1),
            ("ignored", 3_808_214_212_173_381_903, 4),
            ("failure", 8_649_434_444_502_015_239, 5),
        ] {
            assert_eq!(XxHash64::oneshot(0, name.as_bytes()), hash);
            assert_eq!(hash % 5 + 1, partition);
        }
    }

    #[test]
    fn exact_matchers_escape_filterset_delimiters_and_control_characters() {
        assert_eq!(
            escape_matcher("owner::bin/path),\\\n\r\t"),
            "owner::bin\\/path\\)\\,\\\\\\n\\r\\t"
        );
    }
}
