//! Planning API for v1.
//!
//! Stability: v1 (stable).

pub use crate::explain::{explain_bundle, ExplainResponse, PlanExplainStageV1, PlanExplainV1};
pub use crate::request_args::{
    BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs, FastqCrossArgs, PlanRunRequest,
    PlanRunResult,
};
pub use crate::run::plan_run;
pub use crate::run::{select_pipeline, select_pipelines};
pub use crate::v1::bam::plan::plan_for_bam_stage_with_profile;
pub use bijux_dna_core::contract::ExecutionGraph;
pub use bijux_dna_pipelines::fastq::{
    validate_fastq_profile, FastqProfileValidationReport, FastqProfileViolation, FASTQ_INVARIANTS,
};
pub use bijux_dna_pipelines::registry::PipelineRegistry;
pub use bijux_dna_pipelines::{Domain, PipelineProfile};
pub use bijux_dna_planner_bam::{
    pipeline_id_catalog as bam_pipeline_id_catalog, plan_bam_to_bam__adna_capture__v1,
    plan_bam_to_bam__adna_shotgun__v1, BamPipelineInputs,
};
pub use bijux_dna_planner_fastq::{
    cross_fastq_to_bam_id_catalog, fastq_pipeline_id_catalog, plan_fastq_to_bam__default__v1,
    plan_fastq_to_fastq__default__v1, DefaultPipelineOptions, FastqPipelineInputs,
};

#[must_use]
pub fn explain(plan: &ExecutionGraph) -> PlanExplainV1 {
    PlanExplainV1::from_plan(plan)
}
