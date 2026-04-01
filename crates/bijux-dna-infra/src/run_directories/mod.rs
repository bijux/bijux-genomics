//! Run-directory contracts and operational helpers.

mod contract;
mod operations;

pub use crate::paths::{normalize_run_base_dir, pipeline_run_dir, run_layout_paths, run_stage_dir};
pub use contract::{
    RunLayoutContract, RunLayoutPaths, PIPELINE_RUN_DIR_TEMPLATE, RUN_LAYOUT_CONTRACT,
};
pub use operations::{lock_run, publish_run};
