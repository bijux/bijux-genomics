//! Shared test helpers for bijux-engine.

#![allow(dead_code)]

mod execution_setup;
mod manifest_fixture;
mod plan_factory;
mod runner_stub;

pub use execution_setup::execution_setup;
pub use manifest_fixture::{layout_tree_text, write_manifest_hash};
pub use plan_factory::{build_graph, plan_for};
pub use runner_stub::{DeterministicRunner, FakeRunner, RecordingRunner};
