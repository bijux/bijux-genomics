#![allow(
    clippy::assigning_clones,
    clippy::collapsible_if,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::ref_option,
    clippy::semicolon_if_nothing_returned,
    clippy::struct_field_names,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::unnecessary_wraps,
    clippy::uninlined_format_args
)]

mod coverage;
mod explain;
mod models;
mod params;
mod planner;
mod stage_catalog;

pub use explain::explain_vcf_plan;
pub use models::{
    ChunkPlanSettings, PlannerExplainStage, PlannerExplainV1, RegionChunkPlan, VcfPanelLock,
    VcfPipelineInputs,
};
pub use planner::{plan_vcf_minimal, plan_vcf_pipeline, plan_vcf_stage_plans};

pub const PLANNER_VERSION: &str = "bijux-dna-planner-vcf.v2";
