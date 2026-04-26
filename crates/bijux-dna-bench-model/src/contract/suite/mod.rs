//! Owner: bijux-dna-bench-model
//! Suite validation rule families.

mod analysis;
mod diversity;
mod edge_ports;
mod governance;
mod graph;
mod param_bindings;
mod validation;

pub(crate) use analysis::validate_suite_analysis_requirements;
pub(crate) use diversity::validate_suite_diversity;
pub(crate) use edge_ports::validate_edge_ports;
pub(crate) use governance::{
    ensure_supported_stage, planner_owned_graph_stage, validate_stage_tools,
};
pub(crate) use graph::{declared_graph_nodes, validate_suite_dag};
pub(crate) use param_bindings::validate_stage_param_bindings;
pub use validation::validate_suite;
