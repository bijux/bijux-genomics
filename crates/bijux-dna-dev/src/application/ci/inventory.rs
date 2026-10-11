use std::collections::BTreeMap;

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(super) struct TestIdentity {
    pub binary_id: String,
    pub test_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub(super) struct SuiteIdentity {
    #[serde(rename = "crate")]
    pub package: String,
    pub test_binary: String,
    pub kind: String,
}

impl SuiteIdentity {
    pub fn wire_prefix(&self) -> String {
        format!("{}::{}", self.package, self.test_binary)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SuiteFacts {
    pub case_count: usize,
    pub ignored_count: usize,
}

pub(crate) struct ExpectedTestInventory {
    ignored_count: usize,
    pub(super) cases: BTreeMap<String, (TestIdentity, SuiteIdentity)>,
    pub(super) suites: BTreeMap<SuiteIdentity, SuiteFacts>,
}

#[derive(Deserialize)]
struct ListedInventory {
    #[serde(rename = "test-count")]
    test_count: usize,
    #[serde(rename = "rust-suites")]
    suites: BTreeMap<String, ListedSuite>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ListedSuite {
    package_name: String,
    binary_id: String,
    binary_name: String,
    kind: String,
    status: String,
    testcases: BTreeMap<String, ListedCase>,
}

#[derive(Deserialize)]
struct ListedCase {
    ignored: bool,
    #[serde(rename = "filter-match")]
    filter_match: FilterMatch,
}

#[derive(Deserialize)]
struct FilterMatch {
    status: String,
}

impl ExpectedTestInventory {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self> {
        let listed: ListedInventory =
            serde_json::from_slice(bytes).context("parse nextest inventory")?;
        let mut inventory =
            Self { ignored_count: 0, cases: BTreeMap::new(), suites: BTreeMap::new() };
        for (binary_id, suite) in listed.suites {
            ensure!(
                binary_id == suite.binary_id,
                "inventory binary identity disagrees with its map key"
            );
            ensure!(
                suite.status == "listed",
                "inventory contains an undiscovered binary: {binary_id}"
            );
            let identity = SuiteIdentity {
                package: suite.package_name,
                test_binary: suite.binary_name,
                kind: suite.kind,
            };
            let prefix = identity.wire_prefix();

            inventory.suites.insert(
                identity.clone(),
                SuiteFacts {
                    case_count: suite.testcases.len(),
                    ignored_count: suite.testcases.values().filter(|case| case.ignored).count(),
                },
            );
            for (name, case) in suite.testcases {
                ensure!(
                    case.filter_match.status == "matches",
                    "inventory filters out {binary_id}::{name}"
                );
                // The full execution contract runs both ordinary and ignored tests.
                inventory.ignored_count += usize::from(case.ignored);
                let wire_name = format!("{prefix}${name}");
                let test = TestIdentity { binary_id: binary_id.clone(), test_name: name };
                ensure!(
                    inventory.cases.insert(wire_name, (test, identity.clone())).is_none(),
                    "ambiguous nextest test identity"
                );
            }
        }
        ensure!(!inventory.cases.is_empty(), "full test inventory is empty");
        ensure!(
            listed.test_count == inventory.cases.len(),
            "inventory test count does not match declared cases"
        );
        Ok(inventory)
    }

    pub(crate) fn case_count(&self) -> usize {
        self.cases.len()
    }

    pub(crate) fn ignored_case_count(&self) -> usize {
        self.ignored_count
    }
}
