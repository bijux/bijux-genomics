//! Observation and summary contracts for domain-spanning benchmark readiness.
//! Acquisition and semantic admission live outside these persisted wire models.

pub(crate) mod dashboard;
pub(crate) mod expected_results;
pub(crate) mod full_report;
pub(crate) mod parser_collector;
pub(crate) mod rendered_commands;
pub(crate) mod result_collector;
pub(crate) mod smoke_subset;
pub(crate) mod stage_tool_table;

pub(crate) mod output_audit;

pub(crate) mod operational;

pub(crate) mod coverage;
