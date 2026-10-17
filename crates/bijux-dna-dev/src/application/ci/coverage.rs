use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::events::admit_shard_events;
use super::inventory::{ExpectedTestInventory, TestIdentity};
use super::partition::ShardDiscoveryPlan;

/// Exact reconciliation of a producer inventory and selected shard event streams.
#[derive(Serialize)]
pub(crate) struct CoverageAttestation {
    expected_count: usize,
    observed_count: usize,
    missing: Vec<TestIdentity>,
    duplicate: Vec<TestIdentity>,
    failed: Vec<TestIdentity>,
    invalid_shards: BTreeMap<String, String>,
    pub(crate) complete: bool,
    pub(crate) selected_shards_valid: bool,
}

struct ShardEvidence<'a> {
    id: &'a str,
    bytes: Option<&'a [u8]>,
    planned: Option<&'a ShardDiscoveryPlan>,
}

impl CoverageAttestation {
    #[cfg(test)]
    pub(crate) fn reconcile<'a>(
        inventory: &ExpectedTestInventory,
        shards: impl IntoIterator<Item = (&'a str, &'a [u8])>,
    ) -> Self {
        Self::reconcile_expected(
            inventory,
            shards.into_iter().map(|(id, bytes)| ShardEvidence {
                id,
                bytes: Some(bytes),
                planned: None,
            }),
        )
    }

    pub(crate) fn reconcile_planned<'a>(
        inventory: &ExpectedTestInventory,
        shards: impl IntoIterator<Item = (&'a str, &'a [u8], &'a ShardDiscoveryPlan)>,
    ) -> Self {
        Self::reconcile_terminals(
            inventory,
            shards.into_iter().map(|(id, bytes, plan)| (id, Some(bytes), plan)),
        )
    }

    /// None is an explicit no-process terminal; it never substitutes for native event bytes.
    pub(crate) fn reconcile_terminals<'a>(
        inventory: &ExpectedTestInventory,
        shards: impl IntoIterator<Item = (&'a str, Option<&'a [u8]>, &'a ShardDiscoveryPlan)>,
    ) -> Self {
        Self::reconcile_expected(
            inventory,
            shards.into_iter().map(|(id, bytes, plan)| ShardEvidence {
                id,
                bytes,
                planned: Some(plan),
            }),
        )
    }

    fn reconcile_expected<'a>(
        inventory: &ExpectedTestInventory,
        shards: impl IntoIterator<Item = ShardEvidence<'a>>,
    ) -> Self {
        let expected =
            inventory.cases.values().map(|(identity, _)| identity.clone()).collect::<BTreeSet<_>>();
        let mut observed = BTreeSet::new();
        let mut duplicate = BTreeSet::new();
        let mut failed = BTreeSet::new();
        let mut invalid_shards = BTreeMap::new();
        let mut shard_ids = BTreeSet::new();
        for ShardEvidence { id, bytes, planned } in shards {
            if !shard_ids.insert(id) {
                invalid_shards.insert(id.to_string(), "duplicate shard identity".to_string());
                continue;
            }
            let admitted = (|| -> anyhow::Result<_> {
                if let Some(plan) = planned {
                    plan.verify_inventory(inventory)?;
                }
                if let Some(bytes) = bytes {
                    admit_shard_events(inventory, bytes)
                } else {
                    anyhow::ensure!(
                        planned.is_some_and(|plan| plan.expected_case_count == 0),
                        "no-process terminal requires an exact expected-empty partition"
                    );
                    Ok(BTreeMap::new())
                }
            })();
            match admitted {
                Ok(cases) => {
                    if let Some(planned) = planned {
                        let actual = cases.keys().cloned().collect::<BTreeSet<_>>();
                        if actual != planned.expected {
                            invalid_shards.insert(
                                id.to_string(),
                                format!(
                                    "partition case mismatch: missing {:?}; unexpected {:?}",
                                    planned.expected.difference(&actual).collect::<Vec<_>>(),
                                    actual.difference(&planned.expected).collect::<Vec<_>>()
                                ),
                            );
                        }
                    }
                    for (case, passed) in cases {
                        if !observed.insert(case.clone()) {
                            duplicate.insert(case.clone());
                        }
                        if !passed {
                            failed.insert(case);
                        }
                    }
                }
                Err(error) => {
                    invalid_shards.insert(id.to_string(), format!("{error:#}"));
                }
            }
        }
        let missing = expected.difference(&observed).cloned().collect::<Vec<_>>();
        let selected_shards_valid = !shard_ids.is_empty()
            && invalid_shards.is_empty()
            && duplicate.is_empty()
            && failed.is_empty();
        Self {
            expected_count: expected.len(),
            observed_count: observed.len(),
            complete: selected_shards_valid && missing.is_empty(),
            selected_shards_valid,
            missing,
            duplicate: duplicate.into_iter().collect(),
            failed: failed.into_iter().collect(),
            invalid_shards,
        }
    }
}
