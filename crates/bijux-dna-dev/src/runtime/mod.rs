pub(crate) mod compilation_cache;
pub(crate) mod compiled_products;
pub(crate) mod crate_gate_inputs;
pub(crate) mod gate_receipt;
pub(crate) mod local_tests;
pub(crate) mod owned_artifacts;
pub mod process;
pub(crate) mod source_snapshot;
pub mod workspace;
mod workspace_root;

#[allow(dead_code)]
const MODULE_SCOPE: &str = "runtime";
