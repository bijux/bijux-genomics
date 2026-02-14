#![allow(non_snake_case)]
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use bijux_dna_policies::GuardrailConfig;
use walkdir::WalkDir;

include!("workspace_rules/workspace_paths.rs");
include!("workspace_rules/layout_contracts.rs");
include!("workspace_rules/dependency_graph_contracts.rs");
include!("workspace_rules/boundary_enforcement.rs");
include!("workspace_rules/surface_safety_policies.rs");
