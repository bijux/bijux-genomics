//! Execution engine for Bijux.
//!
//! Owns: execution services, validation gates, and observability hooks.
//! Must NOT depend on: bijux-domain-* crates or domain semantics.

#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::implicit_hasher,
    clippy::must_use_candidate,
    clippy::new_without_default
)]

pub(crate) mod core;
pub(crate) mod runner;
pub(crate) mod services;

#[cfg(test)]
mod runner_tests;

use anyhow::Result;
use bijux_core::contract::RunRecordV1;
use bijux_core::plan::execution_plan::ExecutionPlan;
use bijux_runner::Runner;

pub fn validate(plan: &ExecutionPlan) -> Result<()> {
    let context = bijux_core::plan::execution_plan::PlanValidationContext {
        allowed_stage_ids: None,
        allowed_tool_ids: None,
    };
    plan.validate_strict(&context)
}

pub fn execute(
    plan: &ExecutionPlan,
    runner: &dyn Runner,
    _environment: &bijux_environment::api::PlatformSpec,
    _output_dir: &std::path::Path,
) -> Result<RunRecordV1> {
    runner::execute_plan(plan, runner, &runner::ExecutionOptions::default())
}
