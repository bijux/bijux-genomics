//! Execution API for v1.
//!
//! Stability: v1 (stable).

pub use crate::args::{ExecuteRunRequest, ExecuteRunResult, RunRequest, RunResult};
pub use crate::cross_router::run_fastq_to_bam_profile;
pub use crate::run::{execute_run, run_pipeline, RunMode};
pub use bijux_environment::api::{load_image_catalog, load_platform, RunnerKind};
pub use bijux_infra::RUN_LAYOUT_CONTRACT;
pub use bijux_infra::{
    atomic_write_bytes, ensure_dir, normalize_run_base_dir, temp_dir, temp_dir_in, write_bytes,
};

pub use bijux_core::run_index::*;
pub use bijux_core::{
    load_manifests, load_profile, new_run_id, run_dir, FactsRowV1, PathSpec, Profile, RunSpec,
    StageId, StagePlanV1, ToolId, ToolRegistry, ToolRole,
};

pub use bijux_core::ExecutionManifest;
pub use bijux_infra::init_logging;
pub use bijux_runner::primitives::execute_stage_plan;
pub use bijux_runner::primitives::{build_tool_execution_spec, replay_run};
