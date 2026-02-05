//! Planning API for v1.
//!
//! Stability: v1 (stable).

pub use crate::args::{
    BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs, FastqCrossArgs, PlanRunRequest,
    PlanRunResult,
};
pub use crate::bam_plan::plan_for_bam_stage_with_profile;
pub use crate::fastq_router::fastq_preprocess_plan;
pub use crate::run::plan_run;
pub use crate::run::{select_pipeline, select_pipelines};
pub use bijux_pipelines::registry::PipelineRegistry;
pub use bijux_pipelines::{Domain, PipelineProfile};
pub use bijux_planner_bam::{
    explain_plan as explain_bam_plan, plan_bam_to_bam__adna_capture__v1,
    plan_bam_to_bam__adna_shotgun__v1, pipeline_stage_ids as bam_pipeline_stage_ids,
    BamPipelineInputs,
};
pub use bijux_planner_fastq::{
    explain_plan as explain_fastq_plan, plan_fastq_to_bam__default__v1,
    plan_fastq_to_fastq__default__v1, cross_fastq_to_bam_stage_ids,
    fastq_pipeline_stage_ids, DefaultPipelineOptions, FastqPipelineInputs,
};
