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

pub(crate) mod errors;
pub(crate) mod executor;
pub(crate) mod services;

#[cfg(test)]
mod runner_tests;

use anyhow::Result;
use bijux_core::contract::RunRecordV1;
use bijux_core::plan::execution_graph::ExecutionGraph;
use bijux_runtime::Runner;

pub fn validate(graph: &ExecutionGraph) -> Result<()> {
    graph.validate_strict()
}

pub fn execute(graph: &ExecutionGraph, services: &RuntimeServices<'_>) -> Result<RunRecordV1> {
    executor::execute_plan(
        graph,
        services.runner,
        &executor::ExecutionOptions::default(),
    )
}

pub struct RuntimeServices<'a> {
    pub runner: &'a dyn Runner,
}
