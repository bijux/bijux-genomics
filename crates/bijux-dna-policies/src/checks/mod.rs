pub(crate) mod build_authority;
pub(crate) mod dependency_graph;
pub(crate) mod directory_layout;
pub(crate) mod failure_policy;
pub(crate) mod module_files;
pub(crate) mod no_wait;
pub(crate) mod path_match;
pub(crate) mod public_surface;
mod stable_surface;
pub(crate) mod stage_id_literals;

pub(crate) use stable_surface::*;

#[allow(dead_code)]
const CHECK_MODULE_REGISTRY: &str = "policy_check_rule_families";

pub(crate) mod workspace_dependencies;

pub(crate) mod workspace_source_audits;
