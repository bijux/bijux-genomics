//! Planning API for v1.
//!
//! Stability: v1 (stable).

pub use crate::runtime::run::plan_run;
pub use crate::runtime::run::{
    explain_pipeline_profile, select_pipeline, select_pipelines, validate_pipeline_profile,
};
pub use crate::surface::explain::{
    explain_bundle, ExplainResponse, PlanExplainStageV1, PlanExplainV1,
};
pub use crate::surface::request_contracts::{
    BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs, FastqCrossArgs, PlanRunRequest,
    PlanRunResult,
};
pub use crate::v1::bam::plan::plan_for_bam_stage_with_profile;
pub use bijux_dna_core::contract::ExecutionGraph;
pub use bijux_dna_pipelines::bam::{
    validate_bam_profile, BamProfileValidationReport, BamProfileViolation, BAM_INVARIANTS,
};
pub use bijux_dna_pipelines::fastq::{
    validate_fastq_profile, FastqProfileValidationReport, FastqProfileViolation, FASTQ_INVARIANTS,
};
pub use bijux_dna_pipelines::registry::PipelineRegistry;
pub use bijux_dna_pipelines::vcf::{
    validate_vcf_profile, vcf_minimal_profile, VcfProfileValidationReport, VcfProfileViolation,
    VCF_INVARIANTS,
};
pub use bijux_dna_pipelines::{Domain, PipelineProfile};
pub use bijux_dna_pipelines::{cross::cross_workflow_template_by_id, cross::cross_workflow_templates};
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
