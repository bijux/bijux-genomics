#![allow(dead_code)]

mod catalog;
mod graph;

pub use catalog::{
    canonical_amplicon_stage_order, canonical_stage_order, default_amplicon_preprocess_stage_order,
    default_shotgun_preprocess_stage_order, forbidden_transitions, optional_branches,
    stage_criticality, FastqPipelineMode, StageCriticality,
};
pub use graph::{
    preprocess_pipeline, preprocess_pipeline_for_mode, preprocess_pipeline_graph_for_stage_order,
};
