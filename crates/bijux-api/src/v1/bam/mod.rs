//! BAM domain helpers for v1.

pub use bijux_planner_bam::stage_api::{bam_stage_completeness, BamStage};

pub use crate::args::{BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs};

pub mod plan;
pub mod support;

pub use plan::{plan_for_bam_stage, plan_for_bam_stage_with_profile};
pub use support::downstream_enabled;
