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

mod api;
mod chunk_plan;
mod coverage;
mod execution_graph;
mod explain;
mod explain_model;
mod input_policy;
mod params;
mod planner;
mod reference_context;
mod stage_io;
mod stage_plan;
mod stage_sequence;
mod tool_catalog;
mod tool_selection;
mod workspace_config;

pub use api::{ChunkPlanSettings, VcfPanelLock, VcfPipelineInputs};
pub use chunk_plan::RegionChunkPlan;
pub use explain::explain_vcf_plan;
pub use explain_model::{PlannerExplainStage, PlannerExplainV1};
pub use planner::{plan_vcf_minimal, plan_vcf_pipeline, plan_vcf_stage_plans};
pub use reference_context::{
    reference_context_report, resolve_reference_context_report, ReferenceContextReport,
};

pub const PLANNER_VERSION: &str = "bijux-dna-planner-vcf.v2";
