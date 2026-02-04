//! Planning API for v1.
//!
//! Stability: v1 (stable).

pub use crate::args::{
    BenchBamPipelineArgs, BenchBamStageArgs, BamRunArgs, FastqCrossArgs, PlanRunRequest,
    PlanRunResult,
};
pub use crate::bam_plan::plan_for_bam_stage_with_profile;
pub use crate::fastq_router::fastq_preprocess_plan;
pub use crate::run::plan_run;
pub use bijux_pipelines::registry::PipelineRegistry;
pub use bijux_pipelines::{Domain, PipelineProfile};
pub use crate::run::{select_pipeline, select_pipelines};
