//! Public readiness coverage and command projection contracts share one link unit.

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[path = "contracts/benchmark/readiness/coverage/commands_argv.rs"]
mod commands_argv;
#[path = "contracts/benchmark/readiness/coverage/commands_report.rs"]
mod commands_report;
#[path = "contracts/benchmark/readiness/coverage/commands_shell.rs"]
mod commands_shell;
#[path = "contracts/benchmark/readiness/coverage/stage_tool_report.rs"]
mod stage_tool_report;
#[path = "contracts/benchmark/readiness/coverage/stage_tool_tsv.rs"]
mod stage_tool_tsv;
