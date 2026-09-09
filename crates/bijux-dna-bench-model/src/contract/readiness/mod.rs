//! Owner: bijux-dna-bench-model
//! Readiness report admission over explicit catalog expectations.

mod stage_tool_table;

pub use stage_tool_table::{
    validate_all_domain_stage_tool_table_rows, AllDomainStageToolTableAdmissionContext,
};

mod rendered_commands;
pub use rendered_commands::validate_all_domain_rendered_command_rows;
