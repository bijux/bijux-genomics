pub(crate) mod compilation_cache;
pub(crate) mod compiled_tests;
pub mod process;
pub(crate) mod source_snapshot;
pub mod workspace;
mod workspace_root;

#[allow(dead_code)]
const MODULE_SCOPE: &str = "runtime";
