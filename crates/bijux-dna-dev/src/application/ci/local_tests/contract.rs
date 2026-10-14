use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub(crate) const LANE_SCHEMA: &str = "bijux.ci.local_test_lane.v1";
pub(crate) const INPUT_SCHEMA: &str = "bijux.ci.local_test_inputs.v1";
pub(crate) const REQUIRED_LANES: [&str; 5] = [
    "rust-nextest",
    "rust-doctests",
    "python-ordinary",
    "python-workflow",
    "python-developer-doc-assets",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceIdentity {
    pub sha: String,
    pub tree: String,
    pub patch_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FileIdentity {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub mode: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ToolIdentity {
    pub name: String,
    pub path: String,
    pub resolved_path: String,
    pub sha256: String,
    pub version: String,
    pub components: Vec<FileIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneInputs {
    pub schema_version: String,
    pub lane: String,
    pub files: Vec<FileIdentity>,
    pub tools: Vec<ToolIdentity>,
    pub environment: BTreeMap<String, String>,
    pub harness: String,
    pub selections: Vec<String>,
    pub expected: Vec<String>,
    pub os: String,
    pub architecture: String,
    pub closure_qualified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArtifactReference {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaseOutcome {
    pub id: String,
    pub outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneReceipt {
    pub schema_version: String,
    pub lane: String,
    pub source: SourceIdentity,
    pub inputs: LaneInputs,
    pub state: String,
    pub first_attempt: bool,
    pub exit_code: i32,
    pub expected: Vec<String>,
    pub outcomes: Vec<CaseOutcome>,
    pub artifacts: Vec<ArtifactReference>,
    pub errors: Vec<String>,
    pub elapsed_milliseconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneDefinition {
    pub id: String,
    pub harness: String,
    pub discovery_roots: Vec<String>,
    pub explicit_modules: Vec<String>,
    pub excluded_modules: Vec<String>,
    pub input_files: Vec<String>,
    pub input_trees: Vec<String>,
    pub tools: Vec<String>,
    pub resource_units: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneCatalog {
    pub schema_version: String,
    pub lanes: Vec<LaneDefinition>,
}
