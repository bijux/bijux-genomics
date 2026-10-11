//! Owner: bijux-dna-bench-model
//! Readiness report admission over explicit catalog expectations.

mod stage_tool_table;

pub use stage_tool_table::{
    validate_all_domain_stage_tool_table_rows, AllDomainStageToolTableAdmissionContext,
};

mod rendered_commands;
pub use rendered_commands::validate_all_domain_rendered_command_rows;

mod expected_results;
pub use expected_results::validate_all_domain_expected_result_rows;

pub(crate) use rendered_commands::validate_rendered_command_observations;
