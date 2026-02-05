//! BAM domain helpers for v1.

pub use bijux_planner_bam::stage_api::{bam_stage_completeness, BamStage};

pub use crate::args::{BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs};

pub(crate) mod plan;
pub mod support;

pub use support::downstream_enabled;
